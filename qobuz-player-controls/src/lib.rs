use crate::models::{Artist, Favorites, Playlist, SearchResults, Track, TrackStatus};
use cached::{proc_macro::cached, Cached};
use error::Error;
use futures::prelude::*;
use gstreamer::{
    prelude::*, ClockTime, Element, Message, MessageView, SeekFlags, State as GstState,
    StateChangeSuccess, Structure,
};
use models::{Album, ArtistPage};
use notification::Notification;
use qobuz_player_client::client::Client;
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        LazyLock, OnceLock,
    },
    time::Duration,
};
use tokio::{
    select,
    sync::{
        broadcast::{self, Receiver, Sender},
        Mutex, RwLock,
    },
};
use tracing::{debug, instrument};
use tracklist::{TrackListType, Tracklist};

pub use qobuz_player_client::client::{AlbumFeaturedType, PlaylistFeaturedType};
pub mod error;
pub mod models;
pub mod notification;
pub mod tracklist;

pub type Result<T, E = Error> = std::result::Result<T, E>;

static PLAYBIN: LazyLock<Element> = LazyLock::new(|| {
    gstreamer::init().expect("error initializing gstreamer");

    let playbin = gstreamer::ElementFactory::make("playbin3")
        .build()
        .expect("error building playbin element");

    playbin.set_property_from_str("flags", "audio+buffering");

    if gstreamer::version().1 >= 22 {
        playbin.connect("element-setup", false, |value| {
            let element = &value[1].get::<gstreamer::Element>().unwrap();

            if element.name().contains("urisourcebin") {
                element.set_property("parse-streams", true);
            }

            None
        });
    }

    playbin.connect("source-setup", false, |value| {
        let element = &value[1].get::<gstreamer::Element>().unwrap();

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
        TRACK_ABOUT_TO_FINISH
            .tx
            .send(true)
            .expect("failed to send about to finish message");

        None
    });

    playbin
});

struct Broadcast {
    tx: Sender<Notification>,
    rx: Receiver<Notification>,
}

static BROADCAST_CHANNELS: LazyLock<Broadcast> = LazyLock::new(|| {
    let (tx, rx) = broadcast::channel(20);
    Broadcast { tx, rx }
});

struct TrackAboutToFinish {
    tx: Sender<bool>,
    rx: Receiver<bool>,
}

static TRACK_ABOUT_TO_FINISH: LazyLock<TrackAboutToFinish> = LazyLock::new(|| {
    let (tx, rx) = broadcast::channel(1);
    TrackAboutToFinish { tx, rx }
});

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);
static IS_LIVE: AtomicBool = AtomicBool::new(false);
static TARGET_STATUS: LazyLock<RwLock<gstreamer::State>> =
    LazyLock::new(|| RwLock::new(gstreamer::State::Null));
static TRACKLIST: LazyLock<RwLock<Tracklist>> = LazyLock::new(|| RwLock::new(Tracklist::new()));
static USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"
];

static CLIENT: OnceLock<Client> = OnceLock::new();
static CLIENT_INITIATED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static USERNAME: OnceLock<String> = OnceLock::new();
static PASSWORD: OnceLock<String> = OnceLock::new();

#[instrument]
async fn init_client() -> Client {
    tracing::info!("Logging in");
    let username = USERNAME.get().unwrap();
    let password = PASSWORD.get().unwrap();

    let client = qobuz_player_client::client::new(username, password)
        .await
        .expect("error making client");

    tracing::info!("Done");
    client
}

async fn get_client() -> &'static Client {
    if let Some(client) = CLIENT.get() {
        return client;
    }

    let mut inititiated = CLIENT_INITIATED.lock().await;

    if !*inititiated {
        let client = init_client().await;

        CLIENT.set(client).unwrap();
        *inititiated = true;
        drop(inititiated);
    }

    CLIENT.get().unwrap()
}

#[instrument]
/// Ready the player.
async fn ready() -> Result<()> {
    set_player_state(gstreamer::State::Ready).await
}

#[instrument]
/// Stop the player.
pub async fn stop() -> Result<()> {
    set_player_state(gstreamer::State::Null).await
}

async fn set_target_state(state: gstreamer::State) {
    let mut target_status = TARGET_STATUS.write().await;
    *target_status = state;
}

#[instrument]
/// Sets the player to a specific state.
async fn set_player_state(state: gstreamer::State) -> Result<()> {
    let ret = PLAYBIN.set_state(state)?;

    match ret {
        StateChangeSuccess::Success => {
            tracing::debug!("*** successful state change ***");
        }
        StateChangeSuccess::Async => {
            tracing::debug!("*** async state change ***");
        }
        StateChangeSuccess::NoPreroll => {
            tracing::debug!("*** stream is live ***");
            IS_LIVE.store(true, Ordering::Relaxed);
        }
    }

    Ok(())
}

async fn broadcast_track_list(list: &Tracklist) -> Result<()> {
    BROADCAST_CHANNELS
        .tx
        .send(Notification::CurrentTrackList { list: list.clone() })?;
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
    set_target_state(gstreamer::State::Playing).await;
    set_player_state(gstreamer::State::Playing).await?;
    Ok(())
}

#[instrument]
/// Pause the player.
pub async fn pause() -> Result<()> {
    set_target_state(gstreamer::State::Paused).await;
    set_player_state(gstreamer::State::Paused).await?;
    Ok(())
}

#[instrument]
/// Is the player paused?
pub fn is_paused() -> bool {
    PLAYBIN.current_state() != gstreamer::State::Playing
}

#[instrument]
/// Is the player playing?
pub fn is_playing() -> bool {
    PLAYBIN.current_state() == gstreamer::State::Playing
}

#[instrument]
/// Is the player ready?
pub fn is_ready() -> bool {
    PLAYBIN.current_state() == gstreamer::State::Ready
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
        BROADCAST_CHANNELS
            .tx
            .send(Notification::Volume { volume: value })
            .unwrap();
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
/// Skip to a specific track in the tracklist.
pub async fn skip_to_position(new_position: u32, force: bool) -> Result<()> {
    let mut tracklist = TRACKLIST.write().await;
    let current_position = tracklist.current_position();

    if !force && new_position < current_position && current_position == 1 {
        seek(ClockTime::default(), None).await?;
        return Ok(());
    }

    let total_tracks = tracklist.total();

    // Typical previous skip functionality where if,
    // the track is greater than 1 second into playing,
    // then it goes to the beginning. If triggered again
    // within a second after playing, it will skip to the previous track.
    // Ignore if going from the last track to the first (EOS).
    if !force
        && new_position < current_position
        && total_tracks != current_position
        && new_position != 0
    {
        if let Some(current_player_position) = position() {
            if current_player_position.seconds() > 1 {
                seek(ClockTime::default(), None).await?;
                return Ok(());
            }
        }
    }

    ready().await?;

    let client = get_client().await;

    if let Some(next_track) = skip_to_track(&mut tracklist, new_position) {
        let next_track_url = client.track_url(next_track.id).await?;
        PLAYBIN.set_property("uri", next_track_url);
        play().await?;
    } else if let Some(first_track) = tracklist.queue.first_mut() {
        first_track.status = TrackStatus::Playing;
        let first_track_url = client.track_url(first_track.id).await?;

        PLAYBIN.set_property("uri", first_track_url);
    };

    broadcast_track_list(&tracklist).await?;

    Ok(())
}

#[instrument]
pub async fn next() -> Result<()> {
    let current_position = {
        let lock = TRACKLIST.read().await;
        lock.current_position()
    };

    skip_to_position(current_position + 1, true).await
}

#[instrument]
pub async fn previous() -> Result<()> {
    let current_position = {
        let lock = TRACKLIST.read().await;
        lock.current_position()
    };

    skip_to_position(current_position - 1, false).await
}

fn skip_to_track(tracklist: &mut Tracklist, new_position: u32) -> Option<&tracklist::Track> {
    let mut new_track: Option<&tracklist::Track> = None;
    for queue_item in tracklist.queue.iter_mut().enumerate() {
        let queue_item_position = queue_item.0 as u32;
        match queue_item_position.cmp(&new_position) {
            std::cmp::Ordering::Less => {
                queue_item.1.status = TrackStatus::Played;
            }
            std::cmp::Ordering::Equal => {
                queue_item.1.status = TrackStatus::Playing;
                new_track = Some(queue_item.1)
            }
            std::cmp::Ordering::Greater => {
                queue_item.1.status = TrackStatus::Unplayed;
            }
        }
    }

    new_track
}

fn skip_to_next_track(tracklist: &mut Tracklist) {
    let current_position = tracklist.current_position();
    let new_position = current_position + 1;

    for queue_item in tracklist.queue.iter_mut().enumerate() {
        let queue_item_position = queue_item.0 as u32;
        match queue_item_position.cmp(&new_position) {
            std::cmp::Ordering::Less => {
                queue_item.1.status = TrackStatus::Played;
            }
            std::cmp::Ordering::Equal => {
                queue_item.1.status = TrackStatus::Playing;
            }
            std::cmp::Ordering::Greater => {
                queue_item.1.status = TrackStatus::Unplayed;
            }
        }
    }
}

#[instrument]
/// Plays a single track.
pub async fn play_track(track_id: u32) -> Result<()> {
    ready().await?;

    let client = get_client().await;
    let track_url = client.track_url(track_id).await?;
    PLAYBIN.set_property("uri", track_url);
    play().await?;

    let mut tracklist = TRACKLIST.write().await;

    let full_track_info: Track = client.track(track_id).await?.into();
    let track = tracklist::Track {
        id: full_track_info.id,
        title: full_track_info.title,
        status: TrackStatus::Unplayed,
    };

    tracklist.queue = vec![track];
    tracklist.list_type = TrackListType::Track;

    broadcast_track_list(&tracklist).await?;

    Ok(())
}

#[instrument]
/// Plays a full album.
pub async fn play_album(album_id: &str, index: u32) -> Result<()> {
    ready().await?;

    let client = get_client().await;
    let mut tracklist = TRACKLIST.write().await;

    let album = client.album(album_id).await?;
    tracklist.queue = album
        .tracks
        .unwrap_or_default()
        .items
        .into_iter()
        .filter(|t| t.streamable)
        .map(|t| tracklist::Track {
            id: t.id,
            title: t.title,
            status: TrackStatus::Unplayed,
        })
        .collect();

    if let Some(track) = skip_to_track(&mut tracklist, index) {
        let track_url = client.track_url(track.id).await?;
        PLAYBIN.set_property("uri", track_url);
        play().await?;

        tracklist.list_type = TrackListType::Album(tracklist::AlbumTracklist {
            title: album.title,
            id: album.id,
        });

        broadcast_track_list(&tracklist).await?;
    };

    Ok(())
}

#[instrument]
/// Plays top tracks from artist starting from index.
pub async fn play_top_tracks(artist_id: u32, index: u32) -> Result<()> {
    ready().await?;

    let client = get_client().await;
    let mut tracklist = TRACKLIST.write().await;

    tracklist.queue = client
        .artist(artist_id)
        .await?
        .top_tracks
        .into_iter()
        .map(|t| tracklist::Track {
            id: t.id,
            title: t.title,
            status: TrackStatus::Unplayed,
        })
        .collect();

    if let Some(track) = skip_to_track(&mut tracklist, index) {
        let track_url = client.track_url(track.id).await?;
        PLAYBIN.set_property("uri", track_url);
        play().await?;

        tracklist.list_type = TrackListType::Track;

        broadcast_track_list(&tracklist).await?;
    };

    Ok(())
}

#[instrument]
/// Plays all tracks in a playlist.
pub async fn play_playlist(playlist_id: i64, index: u32) -> Result<()> {
    ready().await?;

    let client = get_client().await;
    let mut tracklist = TRACKLIST.write().await;

    let playlist = client.playlist(playlist_id).await?;

    tracklist.queue = playlist
        .tracks
        .unwrap_or_default()
        .items
        .into_iter()
        .filter(|t| t.streamable)
        .map(|t| tracklist::Track {
            id: t.id,
            title: t.title,
            status: TrackStatus::Unplayed,
        })
        .collect();

    if let Some(track) = skip_to_track(&mut tracklist, index) {
        let track_url = client.track_url(track.id).await?;
        PLAYBIN.set_property("uri", track_url);
        play().await?;

        tracklist.list_type = TrackListType::Playlist(tracklist::PlaylistTracklist {
            title: playlist.name,
            id: playlist.id,
        });

        broadcast_track_list(&tracklist).await?;
    };

    Ok(())
}

#[instrument]
/// In response to the about-to-finish signal,
/// prepare the next track by downloading the stream url.
async fn prep_next_track() -> Result<()> {
    tracing::info!("Prepping for next track");

    let client = get_client().await;
    let mut tracklist = TRACKLIST.write().await;

    let total_tracks = tracklist.total();
    let current_position = tracklist.current_position();

    tracing::info!(
        "Total tracks: {}, current position: {}",
        total_tracks,
        current_position
    );

    if total_tracks == current_position {
        tracing::info!("No more tracks left");
    }

    let next_track = tracklist
        .queue
        .iter_mut()
        .enumerate()
        .find(|t| t.0 as u32 == current_position + 1)
        .map(|t| t.1);

    if let Some(next_track) = next_track {
        if let Ok(url) = client.track_url(next_track.id).await {
            PLAYBIN.set_property("uri", url);
        };
    };

    Ok(())
}

#[instrument]
/// Get a notification channel receiver
pub fn notify_receiver() -> Receiver<Notification> {
    BROADCAST_CHANNELS.rx.resubscribe()
}

#[instrument]
/// Returns the current track list loaded in the player.
pub async fn current_tracklist() -> Tracklist {
    TRACKLIST.read().await.clone()
}

#[instrument]
/// Returns the current track loaded in the player.
pub async fn current_track() -> Result<Option<Track>> {
    let track_id = TRACKLIST.read().await.current_track().map(|t| t.id);

    match track_id {
        Some(id) => {
            let client = get_client().await;
            Ok(Some(client.track(id).await?.into()))
        }
        None => Ok(None),
    }
}

#[instrument]
pub async fn search(query: &str) -> Result<SearchResults> {
    let client = get_client().await;
    let user_id = client.get_user_id();

    let results = client.search_all(query, 20).await?;
    Ok(models::parse_search_results(results, user_id))
}

#[instrument]
/// Get artist page
pub async fn artist_page(artist_id: u32) -> Result<ArtistPage> {
    let client = get_client().await;
    let artist = client.artist(artist_id).await?;
    Ok(artist.into())
}

#[instrument]
/// Get similar artists
pub async fn similar_artists(artist_id: u32) -> Result<Vec<Artist>> {
    let client = get_client().await;
    let similar_artists = client.similar_artists(artist_id, None).await?;

    Ok(similar_artists
        .items
        .into_iter()
        .map(|s_a| s_a.into())
        .collect())
}

#[instrument]
/// Get album
pub async fn album(id: &str) -> Result<Album> {
    let client = get_client().await;
    let album = client.album(id).await?;
    Ok(album.into())
}

#[instrument]
/// Get track
pub async fn track(id: u32) -> Result<Track> {
    let client = get_client().await;
    Ok(client.track(id).await?.into())
}

#[instrument]
/// Get suggested albums
pub async fn suggested_albums(album_id: &str) -> Result<Vec<Album>> {
    let client = get_client().await;
    let suggested_albums = client.suggested_albums(album_id).await?;

    Ok(suggested_albums
        .albums
        .items
        .into_iter()
        .map(|x| x.into())
        .collect())
}

#[instrument]
#[cached(size = 1, time = 600)]
/// Get featured albums
pub async fn featured_albums(featured_type: AlbumFeaturedType) -> Result<Vec<Album>> {
    let client = get_client().await;
    let featured = client.featured_albums(featured_type).await?;

    Ok(featured
        .albums
        .items
        .into_iter()
        .map(|value| {
            let year = chrono::NaiveDate::from_str(&value.release_date_original)
                .expect("failed to parse date")
                .format("%Y")
                .to_string()
                .parse::<u32>()
                .expect("error converting year");

            Album {
                id: value.id,
                title: value.title,
                artist: value.artist.into(),
                release_year: year,
                hires_available: value.hires_streamable,
                explicit: value.parental_warning,
                total_tracks: value.tracks_count,
                tracks: vec![],
                available: value.streamable,
                cover_art: value.image.large,
                cover_art_small: value.image.small,
                duration_seconds: value.duration,
            }
        })
        .collect())
}

#[instrument]
#[cached(size = 1, time = 600)]
/// Get featured albums
pub async fn featured_playlists(featured_type: PlaylistFeaturedType) -> Result<Vec<Playlist>> {
    let client = get_client().await;
    let user_id = client.get_user_id();
    let featured = client.featured_playlists(featured_type).await?;

    Ok(featured
        .playlists
        .items
        .into_iter()
        .map(|playlist| models::parse_playlist(playlist, user_id))
        .collect())
}

#[instrument]
/// Get playlist
pub async fn playlist(id: i64) -> Result<Playlist> {
    let client = get_client().await;
    let user_id = client.get_user_id();
    let playlist = client.playlist(id).await?;

    Ok(models::parse_playlist(playlist, user_id))
}

#[instrument]
#[cached(size = 10, time = 600)]
/// Fetch the albums for a specific artist.
pub async fn artist_albums(artist_id: u32) -> Result<Vec<Album>> {
    let client = get_client().await;
    let albums = client.artist_releases(artist_id, None).await?;

    Ok(albums.into_iter().map(|release| release.into()).collect())
}

#[instrument]
/// Add album to favorites
pub async fn add_favorite_album(id: &str) -> Result<()> {
    let client = get_client().await;
    client.add_favorite_album(id).await?;

    FAVORITES.lock().await.cache_clear();
    Ok(())
}

#[instrument]
/// Remove album from favorites
pub async fn remove_favorite_album(id: &str) -> Result<()> {
    let client = get_client().await;
    client.remove_favorite_album(id).await?;

    FAVORITES.lock().await.cache_clear();
    Ok(())
}

#[instrument]
/// Add artist to favorites
pub async fn add_favorite_artist(id: &str) -> Result<()> {
    let client = get_client().await;
    client.add_favorite_artist(id).await?;

    FAVORITES.lock().await.cache_clear();
    Ok(())
}

#[instrument]
/// Remove artist from favorites
pub async fn remove_favorite_artist(id: &str) -> Result<()> {
    let client = get_client().await;
    client.remove_favorite_artist(id).await?;

    FAVORITES.lock().await.cache_clear();
    Ok(())
}

#[instrument]
/// Add playlist to favorites
pub async fn add_favorite_playlist(id: &str) -> Result<()> {
    let client = get_client().await;
    client.add_favorite_playlist(id).await?;

    FAVORITES.lock().await.cache_clear();
    Ok(())
}

#[instrument]
/// Remove playlist from favorites
pub async fn remove_favorite_playlist(id: &str) -> Result<()> {
    let client = get_client().await;
    client.remove_favorite_playlist(id).await?;

    FAVORITES.lock().await.cache_clear();
    Ok(())
}

#[instrument]
/// Fetch the current user's list of playlists.
async fn user_playlists(client: &Client) -> Result<Vec<Playlist>> {
    let user_id = client.get_user_id();
    let playlists = client.user_playlists().await?;

    Ok(playlists
        .playlists
        .items
        .into_iter()
        .map(|playlist| models::parse_playlist(playlist, user_id))
        .collect())
}

#[instrument]
#[cached(size = 1, time = 600)]
/// Get favorites
pub async fn favorites() -> Result<Favorites> {
    let client = get_client().await;
    let (favorites, favorite_playlists) =
        tokio::join!(client.favorites(1000), user_playlists(client));

    let qobuz_player_client::qobuz_models::favorites::Favorites {
        albums,
        tracks: _,
        artists,
    } = favorites?;
    let albums = albums.items;
    let artists = artists.items;

    Ok(Favorites {
        albums: albums.into_iter().map(|x| x.into()).collect(),
        artists: artists.into_iter().map(|x| x.into()).collect(),
        playlists: favorite_playlists?,
    })
}

#[instrument]
/// Inserts the most recent position into the state at a set interval.
async fn clock_loop() {
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
                        .send(Notification::Position { clock: position })
                        .expect("failed to send notification");
                }
            }
        }
    }
}

pub async fn quit() -> Result<()> {
    debug!("stopping player");

    SHOULD_QUIT.store(true, Ordering::Relaxed);

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
        .send(Notification::Quit)
        .expect("error sending broadcast");

    Ok(())
}

#[instrument]
/// Handles messages from GStreamer, receives player actions from external controls
/// receives the about-to-finish event and takes necessary action.
pub async fn player_loop(username: String, password: String) -> Result<()> {
    USERNAME.set(username).unwrap();
    PASSWORD.set(password).unwrap();

    let mut messages = PLAYBIN.bus().unwrap().stream();
    let mut about_to_finish = TRACK_ABOUT_TO_FINISH.rx.resubscribe();

    let clock_loop = tokio::spawn(async { clock_loop().await });

    loop {
        if SHOULD_QUIT.load(Ordering::Relaxed) {
            clock_loop.abort();
            break;
        }

        select! {
             Ok(almost_done) = about_to_finish.recv()=> {
                if almost_done {
                     prep_next_track().await.unwrap();
                }
            }
            Some(msg) = messages.next() => {
                    match handle_message(&msg).await {
                        Ok(_) => {},
                        Err(error) => debug!(?error),
                    };
            }
        }
    }
    Ok(())
}

async fn handle_message(msg: &Message) -> Result<()> {
    match msg.view() {
        MessageView::Eos(_) => {
            tracing::debug!("END OF STREAM");
            let mut tracklist = TRACKLIST.write().await;

            if let Some(last_track) = tracklist.queue.last_mut() {
                last_track.status = TrackStatus::Played;
            };

            if let Some(first_track) = tracklist.queue.first_mut() {
                first_track.status = TrackStatus::Playing;
                let client = get_client().await;
                let track_url = client.track_url(first_track.id).await?;
                PLAYBIN.set_property("uri", track_url);
            };

            ready().await?;
            broadcast_track_list(&tracklist).await?;
        }
        MessageView::StreamStart(_) => {
            tracing::debug!("STREAM START");
            if is_playing() {
                tracing::debug!("Starting next song");

                let mut tracklist = TRACKLIST.write().await;
                skip_to_next_track(&mut tracklist);
                broadcast_track_list(&tracklist).await?;
            }
        }
        MessageView::AsyncDone(msg) => {
            tracing::debug!("ASYNC DONE");

            let position = if let Some(p) = msg.running_time() {
                p
            } else {
                position().unwrap_or_default()
            };

            BROADCAST_CHANNELS
                .tx
                .send(Notification::Position { clock: position })?;
        }
        MessageView::Buffering(buffering) => {
            if IS_LIVE.load(Ordering::Relaxed) {
                debug!("stream is live, ignore buffering");
                return Ok(());
            }
            let percent = buffering.percent();

            if percent < 100 && !is_paused() {
                tracing::info!("Buffering");
                pause().await?;
            } else if percent >= 100 && is_paused() {
                tracing::info!("Done buffering");
                play().await?;
            }
        }
        MessageView::StateChanged(state_changed) => {
            let current_player_state = state_changed
                .current()
                .to_value()
                .get::<GstState>()
                .unwrap();

            let target_status = TARGET_STATUS.read().await;

            if *target_status == current_player_state {
                debug!("player state changed {:?}", current_player_state);

                BROADCAST_CHANNELS.tx.send(Notification::Status {
                    status: current_player_state,
                })?;
            }
        }
        MessageView::ClockLost(_) => {
            tracing::warn!("clock lost, restarting playback");
            pause().await?;
            play().await?;
        }
        MessageView::Error(err) => {
            BROADCAST_CHANNELS
                .tx
                .send(Notification::Error { error: err.into() })?;

            ready().await?;
            pause().await?;
            play().await?;

            tracing::error!(
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
