use cached::proc_macro::cached;
use error::Error;
use flume::{Receiver, Sender};
use futures::prelude::*;
use gst::{
    prelude::*, ClockTime, Element, Message, MessageType, MessageView, SeekFlags,
    State as GstState, StateChangeSuccess, Structure,
};
use gstreamer as gst;
use notification::{BroadcastReceiver, BroadcastSender, Notification};
use qobuz_api::client::{self, api::Client, UrlType};
use queue::{controls::PlayerState, TrackListType, TrackListValue};
use service::{Album, Artist, Favorites, Playlist, SearchResults, Track, TrackStatus};
use std::{
    borrow::BorrowMut,
    collections::BTreeMap,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock, OnceLock,
    },
    time::Duration,
};
use tokio::{select, sync::RwLock};
use tracing::{debug, instrument};

pub mod error;
pub mod mpris;
pub mod notification;
pub mod qobuz;
pub mod queue;
pub mod service;
pub mod sql;

pub type Result<T, E = Error> = std::result::Result<T, E>;

static VERSION: LazyLock<(u32, u32, u32, u32)> = LazyLock::new(gstreamer::version);

static PLAYBIN: LazyLock<Element> = LazyLock::new(|| {
    gst::init().expect("error initializing gstreamer");

    let playbin = gst::ElementFactory::make("playbin3")
        .build()
        .expect("error building playbin element");

    playbin.set_property_from_str("flags", "audio+buffering");

    if VERSION.1 >= 22 {
        playbin.connect("element-setup", false, |value| {
            let element = &value[1].get::<gst::Element>().unwrap();

            if element.name().contains("urisourcebin") {
                element.set_property("parse-streams", true);
            }

            None
        });
    }

    playbin.connect("source-setup", false, |value| {
        let element = &value[1].get::<gst::Element>().unwrap();

        if element.name().contains("souphttpsrc") {
            debug!("new source, changing settings");
            let ua = if rand::random() {
                USER_AGENTS[0]
            } else {
                USER_AGENTS[1]
            };
            element.set_property("user-agent", ua);
            element.set_property("compress", true);
            element.set_property("retries", 10);
            element.set_property("timeout", 30_u32);
            element.set_property(
                "extra-headers",
                Structure::from_str("a-structure, DNT=1, Pragma=no-cache, Cache-Control=no-cache")
                    .expect("failed to make structure from string"),
            )
        }

        None
    });

    playbin.add_property_deep_notify_watch(Some("caps"), true);

    // Connects to the `about-to-finish` signal so the player
    // can setup the next track to play. Enables gapless playback.
    playbin.connect("about-to-finish", false, move |_| {
        debug!("about to finish");
        ABOUT_TO_FINISH
            .tx
            .send(true)
            .expect("failed to send about to finish message");

        None
    });

    playbin
});

struct Broadcast {
    tx: BroadcastSender,
    rx: BroadcastReceiver,
}

static BROADCAST_CHANNELS: LazyLock<Broadcast> = LazyLock::new(|| {
    let (mut tx, rx) = async_broadcast::broadcast(20);
    tx.set_overflow(true);

    Broadcast { rx, tx }
});

struct AboutToFinish {
    tx: Sender<bool>,
    rx: Receiver<bool>,
}

static ABOUT_TO_FINISH: LazyLock<AboutToFinish> = LazyLock::new(|| {
    let (tx, rx) = flume::bounded::<bool>(1);

    AboutToFinish { tx, rx }
});
static IS_BUFFERING: AtomicBool = AtomicBool::new(false);
static IS_LIVE: AtomicBool = AtomicBool::new(false);
static PLAYER_STATE: OnceLock<Arc<RwLock<PlayerState>>> = OnceLock::new();
static USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"
];

#[instrument]
pub async fn init(username: Option<&str>, password: Option<&str>) -> Result<()> {
    let client = qobuz::make_client(username, password)
        .await
        .expect("error making client");

    let tracklist = TrackListValue::new(None);
    let (quit_sender, _) = tokio::sync::broadcast::channel::<bool>(1);

    let state = PlayerState {
        current_track: None,
        service: client,
        tracklist,
        status: gstreamer::State::Null,
        target_status: gstreamer::State::Null,
        quit_sender,
    };

    let state = Arc::new(RwLock::new(state));
    let version = gstreamer::version();
    debug!(?version);

    PLAYER_STATE.set(state).expect("error setting player state");

    Ok(())
}

#[instrument]
/// Ready the player.
async fn ready() -> Result<()> {
    set_player_state(gst::State::Ready).await?;
    Ok(())
}

#[instrument]
/// Stop the player.
pub async fn stop() -> Result<()> {
    set_player_state(gst::State::Null).await?;
    Ok(())
}

#[instrument]
/// Sets the player to a specific state.
pub async fn set_player_state(state: gst::State) -> Result<()> {
    let ret = PLAYBIN.set_state(state)?;

    match ret {
        StateChangeSuccess::Success => {
            tracing::debug!("*** successful state change ***");
        }
        StateChangeSuccess::Async => {
            tracing::debug!("*** async state change ***");

            BROADCAST_CHANNELS
                .tx
                .broadcast(Notification::Loading {
                    is_loading: true,
                    target_state: state,
                })
                .await?;
        }
        StateChangeSuccess::NoPreroll => {
            tracing::debug!("*** stream is live ***");
            IS_LIVE.store(true, Ordering::Relaxed);
        }
    }

    Ok(())
}
async fn broadcast_track_list<'a>(list: &TrackListValue) -> Result<()> {
    BROADCAST_CHANNELS
        .tx
        .broadcast(Notification::CurrentTrackList { list: list.clone() })
        .await?;
    Ok(())
}

#[instrument]
/// Toggle play and pause.
pub async fn play_pause() -> Result<()> {
    if is_playing() {
        pause().await?;
    } else if is_paused() || is_ready() {
        play().await?;
    }

    Ok(())
}

#[instrument]
/// Play the player.
pub async fn play() -> Result<()> {
    if let Some(queue) = PLAYER_STATE.get() {
        let mut state = queue.write().await;
        state.target_status = GstState::Playing;
    }

    set_player_state(gst::State::Playing).await?;
    Ok(())
}

#[instrument]
/// Pause the player.
pub async fn pause() -> Result<()> {
    if let Some(queue) = PLAYER_STATE.get() {
        let mut state = queue.write().await;
        state.target_status = GstState::Paused;
    }

    set_player_state(gst::State::Paused).await?;
    Ok(())
}

#[instrument]
/// Is the player paused?
pub fn is_paused() -> bool {
    PLAYBIN.current_state() == gst::State::Paused
}

#[instrument]
/// Is the player playing?
pub fn is_playing() -> bool {
    PLAYBIN.current_state() == gst::State::Playing
}

#[instrument]
/// Is the player ready?
pub fn is_ready() -> bool {
    PLAYBIN.current_state() == gst::State::Ready
}

#[instrument]
/// Current player state
pub fn current_state() -> GstState {
    PLAYBIN.current_state()
}

#[instrument]
/// Current track position.
pub fn position() -> Option<ClockTime> {
    PLAYBIN.query_position::<ClockTime>()
}

#[instrument]
/// Current track duration.
pub fn duration() -> Option<ClockTime> {
    PLAYBIN.query_duration::<ClockTime>()
}

#[instrument]
/// Current volume
pub fn volume() -> f64 {
    PLAYBIN.property::<f64>("volume")
}

#[instrument]
/// Set volume
pub fn set_volume(value: f64) {
    PLAYBIN.set_property("volume", value);

    tokio::task::spawn(async move {
        _ = BROADCAST_CHANNELS
            .tx
            .broadcast(Notification::Volume { volume: value })
            .await;
    });
}

#[instrument]
/// Seek to a specified time in the current track.
pub async fn seek(time: ClockTime, flags: Option<SeekFlags>) -> Result<()> {
    let flags = flags.unwrap_or(SeekFlags::FLUSH | SeekFlags::TRICKMODE_KEY_UNITS);

    PLAYBIN.seek_simple(flags, time)?;
    Ok(())
}

#[instrument]
/// Jump forward in the currently playing track +10 seconds.
pub async fn jump_forward() -> Result<()> {
    if let (Some(current_position), Some(duration)) = (
        PLAYBIN.query_position::<ClockTime>(),
        PLAYBIN.query_duration::<ClockTime>(),
    ) {
        let ten_seconds = ClockTime::from_seconds(10);
        let next_position = current_position + ten_seconds;

        if next_position < duration {
            seek(next_position, None).await?;
        } else {
            seek(duration, None).await?;
        }
    }

    Ok(())
}

#[instrument]
/// Jump forward in the currently playing track -10 seconds.
pub async fn jump_backward() -> Result<()> {
    if let Some(current_position) = PLAYBIN.query_position::<ClockTime>() {
        if current_position.seconds() < 10 {
            seek(ClockTime::default(), None).await?;
        } else {
            let ten_seconds = ClockTime::from_seconds(10);
            let seek_position = current_position - ten_seconds;

            seek(seek_position, None).await?;
        }
    }

    Ok(())
}

#[instrument]
/// Skip to a specific track in the playlist.
pub async fn skip(new_position: u32, force: bool) -> Result<()> {
    let mut state = PLAYER_STATE.get().unwrap().write().await;
    let current_position = current_track_position(&state);
    let total_tracks = state.tracklist.total();

    // Typical previous skip functionality where if,
    // the track is greater than 1 second into playing,
    // then it goes to the beginning. If triggered again
    // within a second after playing, it will skip to the previous track.
    // Ignore if going from the last track to the first (EOS).
    if !force
        && new_position < current_position
        && total_tracks != current_position
        && new_position != 1
    {
        if let Some(current_player_position) = position() {
            if current_player_position.seconds() > 1 {
                debug!("current track position >1s, seeking to start of track");

                let zero_clock = ClockTime::default();

                seek(zero_clock, None).await?;

                return Ok(());
            }
        }
    }

    ready().await?;

    if let Some(next_track_to_play) = skip_track(state.borrow_mut(), new_position).await {
        let target_status = state.target_status;

        broadcast_track_list(&state.tracklist).await?;
        drop(state);

        BROADCAST_CHANNELS
            .tx
            .broadcast(Notification::Position {
                clock: ClockTime::default(),
            })
            .await?;

        debug!("skipping to next track");

        PLAYBIN.set_property("uri", next_track_to_play);
        set_player_state(target_status).await?;
    }

    Ok(())
}

pub async fn next() -> Result<()> {
    let state = PLAYER_STATE.get().unwrap().read().await;

    let current_position = current_track_position(&state);
    drop(state);
    skip(current_position + 1, true).await?;

    Ok(())
}

pub async fn previous() -> Result<()> {
    let state = PLAYER_STATE.get().unwrap().read().await;

    let current_position = current_track_position(&state);
    drop(state);
    skip(current_position - 1, false).await?;

    Ok(())
}

async fn attach_track_url(client: &Client, track: &mut Track) {
    if let Ok(track_url) = client.track_url(track.id as i32, None).await {
        debug!("attaching url information to track");
        track.track_url = Some(track_url.url);
    }
}

fn current_track_position(state: &PlayerState) -> u32 {
    if let Some(track) = &state.current_track {
        track.position
    } else {
        0
    }
}

async fn skip_track(state: &mut PlayerState, index: u32) -> Option<String> {
    for t in state.tracklist.queue.values_mut() {
        match t.position.cmp(&index) {
            std::cmp::Ordering::Less => {
                t.status = TrackStatus::Played;
            }
            std::cmp::Ordering::Equal => {
                if let Ok(url) = state.service.track_url(t.id as i32, None).await {
                    t.status = TrackStatus::Playing;
                    t.track_url = Some(url.url.clone());
                    state.current_track = Some(t.clone());
                    return Some(url.url);
                } else {
                    t.status = TrackStatus::Unplayable;
                }
            }
            std::cmp::Ordering::Greater => {
                t.status = TrackStatus::Unplayed;
            }
        }
    }

    None
}

#[instrument]
/// Plays a single track.
pub async fn play_track(track_id: i32) -> Result<()> {
    ready().await?;

    let mut state = PLAYER_STATE.get().unwrap().write().await;
    if let Ok(track) = state.service.track(track_id).await {
        let mut track: Track = track.into();
        track.status = TrackStatus::Playing;
        track.number = 1;

        let mut queue = BTreeMap::new();
        queue.entry(track.position).or_insert_with(|| track.clone());

        let mut tracklist = TrackListValue::new(Some(&queue));
        tracklist.set_list_type(TrackListType::Track);

        state.tracklist = tracklist;

        attach_track_url(&state.service, &mut track).await;
        state.current_track = Some(track.clone());

        state.status = GstState::Playing;

        broadcast_track_list(&state.tracklist).await?;

        drop(state);

        PLAYBIN.set_property("uri", track.track_url.clone());

        play().await?;
    }

    Ok(())
}

#[instrument]
/// Plays a full album.
pub async fn play_album(album_id: &str) -> Result<()> {
    ready().await?;

    let mut state = PLAYER_STATE.get().unwrap().write().await;

    if let Ok(album) = state.service.album(album_id).await {
        let album: Album = album.into();
        let mut tracklist = TrackListValue::new(Some(&album.tracks));
        tracklist.set_album(album);
        tracklist.set_list_type(TrackListType::Album);
        tracklist.set_track_status(1, TrackStatus::Playing);

        state.tracklist = tracklist.clone();

        if let Some(mut entry) = tracklist.queue.first_entry() {
            let first_track = entry.get_mut();

            attach_track_url(&state.service, first_track).await;
            state.current_track = Some(first_track.clone());
            state.status = GstState::Playing;
            broadcast_track_list(&state.tracklist).await?;

            drop(state);

            PLAYBIN.set_property("uri", first_track.track_url.clone());

            match play().await {
                Ok(_) => (),
                Err(err) => {
                    tracing::error!("Error playing album: Not able to play: {}", err);
                    return Err(err);
                }
            };
        }
    };

    Ok(())
}

#[instrument]
/// Plays all tracks in a playlist.
pub async fn play_playlist(playlist_id: i64) -> Result<()> {
    ready().await?;

    let mut state = PLAYER_STATE.get().unwrap().write().await;

    if let Ok(playlist) = state.service.playlist(playlist_id).await {
        let playlist: Playlist = playlist.into();
        let mut tracklist = TrackListValue::new(Some(&playlist.tracks));

        tracklist.set_playlist(playlist);
        tracklist.set_list_type(TrackListType::Playlist);
        tracklist.set_track_status(1, TrackStatus::Playing);

        state.tracklist = tracklist.clone();

        if let Some(mut entry) = tracklist.queue.first_entry() {
            let first_track = entry.get_mut();

            attach_track_url(&state.service, first_track).await;
            state.current_track = Some(first_track.clone());

            state.status = GstState::Playing;
            broadcast_track_list(&state.tracklist).await?;

            drop(state);

            PLAYBIN.set_property("uri", first_track.track_url.clone());

            play().await?;
        };
    }

    Ok(())
}

#[instrument]
/// Play an item from Qobuz web uri
pub async fn play_uri(uri: &str) -> Result<()> {
    match client::parse_url(uri) {
        Ok(url) => match url {
            UrlType::Album { id } => {
                play_album(&id).await?;
            }
            UrlType::Playlist { id } => {
                play_playlist(id).await?;
            }
            UrlType::Track { id } => {
                play_track(id).await?;
            }
        },
        Err(err) => {
            return Err(Error::FailedToPlay {
                message: format!("Failed to play item. {err}"),
            })
        }
    }

    Ok(())
}

#[instrument]
/// In response to the about-to-finish signal,
/// prepare the next track by downloading the stream url.
async fn prep_next_track() -> Result<()> {
    let mut state = PLAYER_STATE.get().unwrap().write().await;

    let total_tracks = state.tracklist.total();
    let current_position = current_track_position(&state);

    if total_tracks == current_position {
        debug!("no more tracks left");
    } else if let Some(next_track_url) = skip_track(state.borrow_mut(), current_position + 1).await
    {
        drop(state);

        PLAYBIN.set_property("uri", next_track_url);
    }

    Ok(())
}

#[instrument]
/// Get a notification channel receiver
pub fn notify_receiver() -> BroadcastReceiver {
    BROADCAST_CHANNELS.rx.clone()
}

#[instrument]
/// Returns the current track list loaded in the player.
pub async fn current_tracklist() -> TrackListValue {
    PLAYER_STATE.get().unwrap().read().await.tracklist.clone()
}

#[instrument]
/// Returns the current track loaded in the player.
pub async fn current_track() -> Option<Track> {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .tracklist
        .current_track()
        .cloned()
}

#[instrument]
/// Search the service.
pub async fn search(query: &str) -> SearchResults {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .search_all(query, 20)
        .await
        .ok()
        .map(|x| x.into())
        .unwrap_or_default()
}

#[instrument]
#[cached(size = 1, time = 600)]
/// Get favorites
pub async fn favorites() -> Favorites {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .favorites(1000)
        .await
        .ok()
        .map(|x| x.into())
        .unwrap_or_default()
}

#[instrument]
/// Get artist
pub async fn artist(artist_id: i32) -> Artist {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .artist(artist_id, None)
        .await
        .ok()
        .map(|x| x.into())
        .unwrap_or_default()
}

#[instrument]
/// Get similar artists
pub async fn similar_artists(artist_id: i32) -> Vec<Artist> {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .similar_artists(artist_id, None)
        .await
        .ok()
        .map_or(vec![], |result| {
            result.items.into_iter().map(|x| x.into()).collect()
        })
}

#[instrument]
/// Get album
pub async fn album(id: &str) -> Album {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .album(id)
        .await
        .ok()
        .map(|x| x.into())
        .unwrap()
}

#[instrument]
/// Get suggested albums
pub async fn suggested_albums(album_id: &str) -> Vec<Album> {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .suggested_albums(album_id)
        .await
        .ok()
        .map_or(vec![], |result| {
            result.albums.items.into_iter().map(|x| x.into()).collect()
        })
}

#[instrument]
/// Get playlist
pub async fn playlist(id: i64) -> Playlist {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .playlist(id)
        .await
        .ok()
        .map(|x| x.into())
        .unwrap_or_default()
}

#[instrument]
#[cached(size = 10, time = 600)]
/// Fetch the albums for a specific artist.
pub async fn artist_albums(artist_id: i32) -> Vec<Album> {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .artist_releases(artist_id, None)
        .await
        .ok()
        .map_or(vec![], |result| {
            result.into_iter().map(|release| release.into()).collect()
        })
}

#[instrument]
/// Add album to favorites
pub async fn add_favorite_album(id: &str) {
    _ = PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .add_favorite_album(id)
        .await;
}

#[instrument]
/// Remove album from favorites
pub async fn remove_favorite_album(id: &str) {
    _ = PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .remove_favorite_album(id)
        .await;
}

#[instrument]
/// Add artist to favorites
pub async fn add_favorite_artist(id: &str) {
    _ = PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .add_favorite_artist(id)
        .await;
}

#[instrument]
/// Remove artist from favorites
pub async fn remove_favorite_artist(id: &str) {
    _ = PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .remove_favorite_artist(id)
        .await;
}

#[instrument]
/// Add playlist to favorites
pub async fn add_favorite_playlist(id: &str) {
    _ = PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .add_favorite_playlist(id)
        .await;
}

#[instrument]
/// Remove playlist from favorites
pub async fn remove_favorite_playlist(id: &str) {
    _ = PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .remove_favorite_playlist(id)
        .await;
}

#[instrument]
#[cached(size = 10, time = 600)]
/// Fetch the tracks for a specific playlist.
pub async fn playlist_tracks(playlist_id: i64) -> Vec<Track> {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .playlist(playlist_id)
        .await
        .ok()
        .map_or(vec![], |result| {
            result.tracks.map_or(vec![], |tracks| {
                tracks.items.into_iter().map(|track| track.into()).collect()
            })
        })
}

#[instrument]
#[cached(size = 1, time = 600)]
/// Fetch the current user's list of playlists.
pub async fn user_playlists() -> Vec<Playlist> {
    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .service
        .user_playlists()
        .await
        .ok()
        .map_or(vec![], |x| {
            x.playlists
                .items
                .into_iter()
                .map(|playlist| playlist.into())
                .collect()
        })
}

/// Inserts the most recent position into the state at a set interval.
#[instrument]
pub async fn clock_loop() {
    debug!("starting clock loop");

    let mut interval = tokio::time::interval(Duration::from_millis(250));
    let mut last_position = ClockTime::default();

    loop {
        interval.tick().await;

        if current_state() == GstState::Playing {
            if let Some(position) = position() {
                if position.seconds() != last_position.seconds() {
                    last_position = position;

                    BROADCAST_CHANNELS
                        .tx
                        .broadcast(Notification::Position { clock: position })
                        .await
                        .expect("failed to send notification");
                }
            }
        }
    }
}

pub async fn quit() -> Result<()> {
    debug!("stopping player");

    PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .quit_sender
        .send(true)
        .unwrap();

    if is_playing() {
        debug!("pausing player");
        pause().await?;
    }

    if is_paused() {
        debug!("readying player");
        ready().await?;
    }

    if is_ready() {
        debug!("stopping player");
        stop().await?;
    }

    BROADCAST_CHANNELS
        .tx
        .broadcast(Notification::Quit)
        .await
        .expect("error sending broadcast");

    Ok(())
}

/// Handles messages from GStreamer, receives player actions from external controls
/// receives the about-to-finish event and takes necessary action.
#[instrument]
pub async fn player_loop() -> Result<()> {
    let mut messages = PLAYBIN.bus().unwrap().stream();
    let mut about_to_finish = ABOUT_TO_FINISH.rx.stream();

    let mut quitter = PLAYER_STATE
        .get()
        .unwrap()
        .read()
        .await
        .quit_sender
        .subscribe();

    let clock_handle = tokio::spawn(async { clock_loop().await });

    loop {
        select! {
            Ok(should_quit)= quitter.recv() => {
                if should_quit {
                    clock_handle.abort();
                    break;
                }
            }
            Some(almost_done) = about_to_finish.next() => {
                if almost_done {
                    tokio::spawn(async { prep_next_track().await });
                }
            }
            Some(msg) = messages.next() => {
                if msg.type_() == MessageType::Buffering {
                    match handle_message(&msg).await {
                        Ok(_) => {},
                        Err(error) => debug!(?error),
                    };
                } else {
                    tokio::spawn(async move { match handle_message(&msg).await {
                            Ok(()) => {}
                            Err(error) => {debug!(?error);}
                        }
                    });
                }
            }
        }
    }

    Ok(())
}

async fn handle_message(msg: &Message) -> Result<()> {
    match msg.view() {
        MessageView::Eos(_) => {
            debug!("END OF STREAM");
            let mut state = PLAYER_STATE.get().unwrap().write().await;
            state.target_status = GstState::Paused;
            drop(state);

            skip(1, true).await?;
        }
        MessageView::StreamStart(_) => {
            if is_playing() {
                broadcast_track_list(&PLAYER_STATE.get().unwrap().read().await.tracklist).await?;
            }
        }
        MessageView::AsyncDone(msg) => {
            debug!("ASYNC DONE");
            BROADCAST_CHANNELS
                .tx
                .broadcast(Notification::Loading {
                    is_loading: false,
                    target_state: PLAYER_STATE.get().unwrap().read().await.target_status,
                })
                .await?;

            let position = if let Some(p) = msg.running_time() {
                p
            } else {
                position().unwrap_or_default()
            };

            BROADCAST_CHANNELS
                .tx
                .broadcast(Notification::Position { clock: position })
                .await?;
        }
        MessageView::Buffering(buffering) => {
            if IS_LIVE.load(Ordering::Relaxed) {
                debug!("stream is live, ignore buffering");
                return Ok(());
            }
            let percent = buffering.percent();

            let target_status = PLAYER_STATE.get().unwrap().read().await.target_status;

            if percent < 100 && !is_paused() && !IS_BUFFERING.load(Ordering::Relaxed) {
                pause().await?;
                IS_BUFFERING.store(true, Ordering::Relaxed);
            } else if percent > 99 && IS_BUFFERING.load(Ordering::Relaxed) && is_paused() {
                play().await?;
                IS_BUFFERING.store(false, Ordering::Relaxed);
            }

            if percent.rem_euclid(10) == 0 {
                debug!("buffering {}%", percent);
                BROADCAST_CHANNELS
                    .tx
                    .broadcast(Notification::Buffering {
                        is_buffering: percent < 99,
                        target_state: target_status,
                        percent: percent as u32,
                    })
                    .await?;
            }
        }
        MessageView::StateChanged(state_changed) => {
            let current_state = state_changed
                .current()
                .to_value()
                .get::<GstState>()
                .unwrap();

            let mut state = PLAYER_STATE.get().unwrap().write().await;

            if state.status != current_state && state.target_status == current_state {
                debug!("player state changed {:?}", current_state);
                state.status = current_state;
                drop(state);

                BROADCAST_CHANNELS
                    .tx
                    .broadcast(Notification::Status {
                        status: current_state,
                    })
                    .await?;
            }
        }
        MessageView::ClockLost(_) => {
            debug!("clock lost, restarting playback");
            pause().await?;
            play().await?;
        }
        MessageView::Error(err) => {
            BROADCAST_CHANNELS
                .tx
                .broadcast(Notification::Error { error: err.into() })
                .await?;

            ready().await?;
            pause().await?;
            play().await?;

            debug!(
                "Error from {:?}: {} ({:?})",
                err.src().map(|s| s.path_string()),
                err.error(),
                err.debug()
            );
        }
        _ => (),
    }

    Ok(())
}
