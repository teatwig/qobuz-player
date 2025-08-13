use crate::models::{Track, TrackStatus};
use client::Client;
use error::Error;
use futures::prelude::*;
use gstreamer::{Message, MessageView, prelude::*};
use models::Album;
use notification::{Notification, PlayNotification};
use rand::seq::SliceRandom;
use sink::{Sink, init_sink};
use std::{
    ops::{Add, Sub},
    sync::Arc,
    time::Duration,
};
use tokio::{
    select,
    sync::{
        RwLock,
        broadcast::{self, Receiver, Sender},
    },
};
use tracing::{debug, instrument};
use tracklist::{SingleTracklist, Tracklist, TracklistType};

pub use qobuz_player_client::client::{AlbumFeaturedType, AudioQuality, PlaylistFeaturedType};
pub mod client;
pub mod error;
pub mod models;
pub mod notification;
pub mod sink;
pub mod tracklist;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct Time(u64);

impl Time {
    pub fn from_mseconds(value: u64) -> Self {
        Self(value)
    }

    pub fn from_seconds(value: u64) -> Self {
        Self(value * 1000)
    }

    pub fn mseconds(&self) -> u64 {
        self.0
    }
}

impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Time::from_mseconds(self.0 + rhs.0)
    }
}

impl Sub for Time {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Time::from_mseconds(self.0 - rhs.0)
    }
}

#[derive(Debug)]
pub struct ReadOnly<T>(Arc<RwLock<T>>);

impl<T> Clone for ReadOnly<T> {
    fn clone(&self) -> Self {
        ReadOnly(self.0.clone())
    }
}

impl<T> ReadOnly<T> {
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, T> {
        self.0.read().await
    }
}

impl<T> From<Arc<RwLock<T>>> for ReadOnly<T> {
    fn from(arc: Arc<RwLock<T>>) -> Self {
        ReadOnly(arc)
    }
}

#[derive(Debug)]
pub struct Player {
    tracklist: Arc<RwLock<Tracklist>>,
    target_status: Arc<RwLock<tracklist::Status>>,
    last_updated_tracklist: chrono::DateTime<chrono::Utc>,
    client: Arc<Client>,
    broadcast: Arc<Broadcast>,
    sink: Arc<Sink>,
}

impl Player {
    pub fn start_player(
        tracklist: Arc<RwLock<Tracklist>>,
        client: Arc<Client>,
    ) -> (Arc<RwLock<tracklist::Status>>, Arc<Broadcast>, Arc<Sink>) {
        let target_status = Arc::new(RwLock::new(Default::default()));
        let (tx, rx) = broadcast::channel(20);
        let broadcast = Arc::new(Broadcast { tx, rx });
        let sink = Arc::new(init_sink(broadcast.clone()));

        let mut player = Self {
            tracklist,
            target_status: target_status.clone(),
            last_updated_tracklist: chrono::Utc::now(),
            client,
            broadcast: broadcast.clone(),
            sink: sink.clone(),
        };

        tokio::spawn(async move {
            match player.player_loop().await {
                Ok(_) => debug!("player loop exited successfully"),
                Err(error) => debug!("player loop error {error}"),
            }
        });

        (target_status, broadcast, sink)
    }

    async fn play_pause(&mut self) -> Result<()> {
        let target_status = *self.target_status.read().await;

        match target_status {
            tracklist::Status::Playing => self.pause().await,
            tracklist::Status::Paused | tracklist::Status::Stopped => self.play().await,
        }
    }

    async fn ready(&self) -> Result<()> {
        self.set_player_state(gstreamer::State::Ready)
    }

    async fn play(&mut self) -> Result<()> {
        if let Some(current_track_id) = self.tracklist.read().await.currently_playing() {
            let track_url = self.track_url(current_track_id).await?;
            self.query_track_url(&track_url).await?;
        }

        if chrono::Utc::now() - self.last_updated_tracklist > chrono::Duration::minutes(10) {
            let current_position = self.sink.position().unwrap_or_default();

            if let Some(track_id) = self.tracklist.read().await.currently_playing() {
                self.ready().await?;
                let track_url = self.track_url(track_id).await?;
                self.query_track_url(&track_url).await?;

                self.sink.seek(current_position)?;
            }
        }

        self.set_target_state(tracklist::Status::Playing).await;
        self.set_player_state(gstreamer::State::Playing)?;
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        self.set_target_state(tracklist::Status::Paused).await;
        self.set_player_state(gstreamer::State::Paused)?;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.set_target_state(tracklist::Status::Stopped).await;
        self.set_player_state(gstreamer::State::Null)
    }

    async fn set_target_state(&self, state: tracklist::Status) {
        let mut target_status = self.target_status.write().await;
        *target_status = state;

        self.broadcast
            .tx
            .send(Notification::Status { status: state })
            .unwrap();
    }

    fn set_player_state(&self, state: gstreamer::State) -> Result<()> {
        self.sink.set_state(state)?;
        Ok(())
    }

    async fn track_url(&self, track_id: u32) -> Result<String, qobuz_player_client::Error> {
        self.client.track_url(track_id).await
    }

    async fn query_track_url(&self, track_url: &str) -> Result<()> {
        self.sink.query_track_url(track_url)
    }

    async fn is_playing(&self) -> bool {
        *self.target_status.read().await == tracklist::Status::Playing
    }

    fn set_volume(&self, volume: f64) {
        self.sink.set_volume(volume);
    }

    async fn handle_message(&mut self, msg: &Message) -> Result<()> {
        match msg.view() {
            MessageView::Eos(_) => {
                tracing::debug!("END OF STREAM");
                self.ready().await?;
                self.set_target_state(tracklist::Status::Paused).await;

                let mut tracklist = self.tracklist.write().await;

                if let Some(last_track) = tracklist.queue.last_mut() {
                    last_track.status = TrackStatus::Played;
                }

                if let Some(first_track) = tracklist.queue.first_mut() {
                    first_track.status = TrackStatus::Playing;
                    let track_url = self.client.track_url(first_track.id).await?;
                    self.sink.query_track_url(&track_url)?;
                }

                self.broadcast_tracklist(tracklist.clone());
            }
            MessageView::StreamStart(_) => {
                tracing::debug!("STREAM START");

                let position = self.sink.position();

                if position.is_some_and(|position| position.mseconds() > 500) {
                    tracing::debug!("Starting next song");

                    self.skip_to_next_track().await;
                    self.broadcast_tracklist(self.tracklist.read().await.clone());
                }
            }
            MessageView::AsyncDone(msg) => {
                tracing::debug!("ASYNC DONE");

                let position = if let Some(p) = msg.running_time() {
                    Time::from_mseconds(p.mseconds())
                } else {
                    self.sink.position().unwrap_or_default()
                };

                self.broadcast
                    .tx
                    .send(Notification::Position { position })?;
            }
            MessageView::Buffering(buffering) => {
                if self.is_playing().await {
                    tracing::debug!("Playing, ignore buffering");
                    return Ok(());
                }

                tracing::debug!("Buffering");

                if buffering.percent() >= 100 {
                    tracing::info!("Done buffering");
                    self.play().await?;
                }
            }
            MessageView::StateChanged(_) => {}
            MessageView::ClockLost(_) => {
                tracing::warn!("clock lost, restarting playback");
                self.pause().await?;
                self.play().await?;
            }
            MessageView::Error(err) => {
                self.broadcast.tx.send(Notification::Message {
                    message: notification::Message::Error(err.to_string()),
                })?;

                self.ready().await?;
                self.pause().await?;
                self.play().await?;

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

    fn broadcast_tracklist(&self, tracklist: Tracklist) {
        self.broadcast
            .tx
            .send(Notification::CurrentTrackList { tracklist })
            .unwrap();
    }

    async fn skip_to_next_track(&mut self) {
        let mut tracklist = self.tracklist.write().await;

        let current_position = tracklist.current_position();
        let new_position = current_position + 1;
        tracklist.skip_to_track(new_position);
        self.broadcast_tracklist(tracklist.clone());
    }

    async fn jump_forward(&mut self) -> Result<()> {
        if let (Some(current_position), Some(duration)) =
            (self.sink.position(), self.sink.duration())
        {
            let ten_seconds = Time::from_seconds(10);
            let next_position = current_position + ten_seconds;

            if next_position < duration {
                self.sink.seek(next_position)?;
            } else {
                self.sink.seek(duration)?;
            }
        }

        Ok(())
    }

    async fn jump_backward(&mut self) -> Result<()> {
        if let Some(current_position) = self.sink.position() {
            if current_position.mseconds() < 10000 {
                self.sink.seek(Time::default())?;
            } else {
                let ten_seconds = Time::from_seconds(10);
                let seek_position = current_position - ten_seconds;

                self.sink.seek(seek_position)?;
            }
        }

        Ok(())
    }

    /// Skip to a specific track in the tracklist.
    async fn skip_to_position(&mut self, new_position: u32, force: bool) -> Result<()> {
        let mut tracklist = self.tracklist.write().await;
        let current_position = tracklist.current_position();

        if !force && new_position < current_position && current_position == 1 {
            self.sink.seek(Time::default())?;
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
            if let Some(current_player_position) = self.sink.position() {
                if current_player_position.mseconds() > 1000 {
                    self.sink.seek(Time::default())?;
                    return Ok(());
                }
            }
        }

        self.ready().await?;

        if let Some(next_track) = tracklist.skip_to_track(new_position) {
            let next_track_url = self.track_url(next_track.id).await?;
            self.query_track_url(&next_track_url).await?;
        } else if let Some(first_track) = tracklist.queue.first_mut() {
            first_track.status = TrackStatus::Playing;
            let first_track_url = self.track_url(first_track.id).await?;

            self.sink.query_track_url(&first_track_url)?;
        }

        self.broadcast_tracklist(tracklist.clone());

        Ok(())
    }

    async fn next(&mut self) -> Result<()> {
        let current_position = self.tracklist.read().await.current_position();

        self.skip_to_position(current_position + 1, true).await
    }

    async fn previous(&mut self) -> Result<()> {
        let current_position = self.tracklist.read().await.current_position();

        let next = if current_position == 0 {
            0
        } else {
            current_position - 1
        };

        self.skip_to_position(next, false).await
    }

    async fn play_track(&mut self, track_id: u32) -> Result<()> {
        self.ready().await?;

        let track_url = self.track_url(track_id).await?;
        self.query_track_url(&track_url).await?;

        let mut track: Track = self.client.track(track_id).await?;
        let mut tracklist = self.tracklist.write().await;

        tracklist.list_type = TracklistType::Track(SingleTracklist {
            track_title: track.title.clone(),
            album_id: track.album_id.clone(),
            image: track.image.clone(),
        });

        track.status = TrackStatus::Playing;

        tracklist.queue = vec![track];

        self.broadcast_tracklist(tracklist.clone());

        Ok(())
    }

    async fn play_album(&mut self, album_id: &str, index: u32) -> Result<()> {
        self.ready().await?;

        let album: Album = self.client.album(album_id).await?;

        let unstreambale_tracks_to_index = album
            .tracks
            .iter()
            .take(index as usize)
            .filter(|t| !t.available)
            .count() as u32;

        let mut tracklist = self.tracklist.write().await;
        tracklist.queue = album.tracks.into_iter().filter(|t| t.available).collect();

        if let Some(track) = tracklist.skip_to_track(index - unstreambale_tracks_to_index) {
            let track_url = self.track_url(track.id).await?;
            self.query_track_url(&track_url).await?;

            tracklist.list_type = TracklistType::Album(tracklist::AlbumTracklist {
                title: album.title,
                id: album.id,
                image: Some(album.image),
            });

            self.broadcast_tracklist(tracklist.clone());
        }

        Ok(())
    }

    async fn play_top_tracks(&mut self, artist_id: u32, index: u32) -> Result<()> {
        self.ready().await?;

        let artist = self.client.artist_page(artist_id).await?;

        let unstreambale_tracks_to_index = artist
            .top_tracks
            .iter()
            .take(index as usize)
            .filter(|t| !t.available)
            .count() as u32;

        let mut tracklist = self.tracklist.write().await;
        if let Some(track) = tracklist.skip_to_track(index - unstreambale_tracks_to_index) {
            let track_url = self.track_url(track.id).await?;
            self.query_track_url(&track_url).await?;

            tracklist.list_type = TracklistType::TopTracks(tracklist::TopTracklist {
                artist_name: artist.name,
                id: artist_id,
                image: artist.image,
            });

            self.broadcast_tracklist(tracklist.clone());
        }

        Ok(())
    }

    async fn play_playlist(&mut self, playlist_id: u32, index: u32, shuffle: bool) -> Result<()> {
        self.ready().await?;

        let playlist = self.client.playlist(playlist_id).await?;

        let unstreambale_tracks_to_index = playlist
            .tracks
            .iter()
            .take(index as usize)
            .filter(|t| !t.available)
            .count() as u32;

        let mut tracks: Vec<Track> = playlist
            .tracks
            .into_iter()
            .filter(|t| t.available)
            .collect();

        if shuffle {
            let mut rng = rand::rng();
            tracks.shuffle(&mut rng);
        }

        let mut tracklist = self.tracklist.write().await;
        tracklist.queue = tracks;

        if let Some(track) = tracklist.skip_to_track(index - unstreambale_tracks_to_index) {
            let track_url = self.track_url(track.id).await?;
            self.query_track_url(&track_url).await?;

            tracklist.list_type = TracklistType::Playlist(tracklist::PlaylistTracklist {
                title: playlist.title,
                id: playlist.id,
                image: playlist.image,
            });

            self.broadcast_tracklist(tracklist.clone());
        }

        Ok(())
    }

    #[instrument]
    /// In response to the about-to-finish signal,
    /// prepare the next track by downloading the stream url.
    async fn prep_next_track(&self) {
        tracing::info!("Prepping for next track");

        let tracklist = self.tracklist.read().await;
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
            .iter()
            .enumerate()
            .find(|t| t.0 as u32 == current_position + 1)
            .map(|t| t.1);

        if let Some(next_track) = next_track {
            if let Ok(url) = self.track_url(next_track.id).await {
                self.sink.query_track_url(&url).unwrap();
            }
        }
    }

    /// Handles messages from GStreamer, receives player actions from external controls
    /// receives the about-to-finish event and takes necessary action.
    pub async fn player_loop(&mut self) -> Result<()> {
        let mut messages = self.sink.bus();
        let mut receiver = self.broadcast.notify_receiver();

        let mut interval = tokio::time::interval(Duration::from_millis(500));
        let mut last_position = Time::default();

        loop {
            select! {
                _ = interval.tick() => {
                    let target_status = *self.target_status.read().await;
                    if target_status == tracklist::Status::Playing {
                        if let Some(position) = self.sink.position() {
                            if position.mseconds() != last_position.mseconds() {
                                last_position = position;

                                self.broadcast
                                    .tx
                                    .send(Notification::Position { position })
                                    .expect("failed to send notification");
                            }
                        }
                    }
                }
                Some(msg) = messages.next() => {
                    match self.handle_message(&msg).await {
                        Ok(_) => {},
                        Err(error) => tracing:: debug!(?error),
                    }
                }
                Ok(notification) = receiver.recv() => {
                    match notification {
                        Notification::Play(play) => {
                            match play {
                                PlayNotification::Album { id, index } => {
                                    self.play_album(&id, index).await.unwrap();
                                },
                                PlayNotification::Playlist { id, index, shuffle } => {
                                    self.play_playlist(id, index, shuffle).await.unwrap();
                                },
                                PlayNotification::ArtistTopTracks { artist_id, index } => {
                                    self.play_top_tracks(artist_id, index).await.unwrap();
                                },
                                PlayNotification::Track { id } => {
                                    self.play_track(id).await.unwrap();
                                },
                                PlayNotification::Next => {
                                    self.next().await.unwrap();
                                },
                                PlayNotification::Previous => {
                                    self.previous().await.unwrap();
                                },
                                PlayNotification::PlayPause => {
                                    self.play_pause().await.unwrap();
                                },
                                PlayNotification::Play => {
                                    self.play().await.unwrap();
                                },
                                PlayNotification::Pause => {
                                    self.pause().await.unwrap();
                                },
                                PlayNotification::Stop => {
                                    self.stop().await.unwrap();
                                },
                                PlayNotification::SkipToPosition {new_position, force} => {
                                    self.skip_to_position(new_position, force).await.unwrap();
                                },
                                PlayNotification::JumpForward => {
                                    self.jump_forward().await.unwrap();
                                },
                                PlayNotification::JumpBackward => {
                                    self.jump_backward().await.unwrap();
                                },
                                PlayNotification::Seek { time } => {
                                    self.sink.seek(time).unwrap();
                                },
                                PlayNotification::TrackAboutToFinish=> {
                                     self.prep_next_track().await;
                                },
                            }
                        },
                        Notification::Quit => {
                            break;
                        },
                        Notification::Status { status: _ } => (),
                        Notification::Position { position: _ } => (),
                        Notification::CurrentTrackList{ tracklist: _ } => {
                            self.last_updated_tracklist = chrono::Utc::now();
                        },
                        Notification::Message { message: _ } => (),
                        Notification::Volume { volume } => {
                            self.set_volume(volume);
                        },
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Broadcast {
    tx: Sender<Notification>,
    rx: Receiver<Notification>,
}

impl Broadcast {
    pub fn quit(&self) {
        self.tx.send(Notification::Quit).unwrap();
    }

    pub fn track_about_to_finish(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::TrackAboutToFinish))
            .unwrap();
    }

    pub fn next(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Next))
            .unwrap();
    }

    pub fn previous(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Previous))
            .unwrap();
    }

    pub fn play_pause(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::PlayPause))
            .unwrap();
    }

    pub fn play(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Play))
            .unwrap();
    }

    pub fn pause(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Pause))
            .unwrap();
    }

    pub fn play_album(&self, id: &str, index: u32) {
        self.tx
            .send(Notification::Play(PlayNotification::Album {
                id: id.to_string(),
                index,
            }))
            .unwrap();
    }

    pub fn play_playlist(&self, id: u32, index: u32, shuffle: bool) {
        self.tx
            .send(Notification::Play(PlayNotification::Playlist {
                id,
                index,
                shuffle,
            }))
            .unwrap();
    }

    pub fn play_track(&self, id: u32) {
        self.tx
            .send(Notification::Play(PlayNotification::Track { id }))
            .unwrap();
    }

    pub fn play_top_tracks(&self, artist_id: u32, index: u32) {
        self.tx
            .send(Notification::Play(PlayNotification::ArtistTopTracks {
                artist_id,
                index,
            }))
            .unwrap();
    }

    pub fn skip_to_position(&self, index: u32, force: bool) {
        self.tx
            .send(Notification::Play(PlayNotification::SkipToPosition {
                new_position: index,
                force,
            }))
            .unwrap();
    }

    pub fn stop(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Stop))
            .unwrap();
    }

    pub fn set_volume(&self, value: f64) {
        self.tx
            .send(Notification::Volume { volume: value })
            .unwrap();
    }

    pub fn seek(&self, time: Time) {
        self.tx
            .send(Notification::Play(PlayNotification::Seek { time }))
            .unwrap();
    }

    pub fn jump_forward(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::JumpForward))
            .unwrap();
    }

    pub fn jump_backward(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::JumpBackward))
            .unwrap();
    }

    pub fn notify_receiver(&self) -> Receiver<Notification> {
        self.rx.resubscribe()
    }

    pub fn send_message(&self, message: notification::Message) {
        self.tx.send(Notification::Message { message }).unwrap();
    }
}
