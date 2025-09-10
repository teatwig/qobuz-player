use rand::seq::SliceRandom;
use tokio::{select, sync::RwLock};

use crate::{
    Result, Status,
    models::{Album, Track, TrackStatus},
    notification::{Notification, PlayNotification},
    tracklist::{SingleTracklist, TracklistType},
};
use std::{sync::Arc, time::Duration};

use crate::{
    broadcast::Broadcast,
    client::Client,
    readonly::ReadOnly,
    sink::Sink,
    tracklist::{self, Tracklist},
};

pub struct Player {
    tracklist: Arc<RwLock<Tracklist>>,
    target_status: Arc<RwLock<Status>>,
    client: Arc<Client>,
    broadcast: Arc<Broadcast>,
    sink: Sink,
    volume: Arc<RwLock<f32>>,
    position: Arc<RwLock<Duration>>,
    next_track_is_queried: bool,
    first_track_queried: bool,
}

impl Player {
    pub fn new(tracklist: Arc<RwLock<Tracklist>>, client: Arc<Client>, volume: f32) -> Self {
        let broadcast = Arc::new(Broadcast::new());
        let sink = Sink::new(broadcast.clone(), volume).unwrap();
        let volume = Arc::new(RwLock::new(volume));

        Self {
            tracklist,
            target_status: Default::default(),
            client,
            broadcast,
            sink,
            volume,
            position: Default::default(),
            next_track_is_queried: false,
            first_track_queried: false,
        }
    }

    pub fn status(&self) -> ReadOnly<Status> {
        self.target_status.clone().into()
    }

    pub fn broadcast(&self) -> Arc<Broadcast> {
        self.broadcast.clone()
    }

    pub fn volume(&self) -> ReadOnly<f32> {
        self.volume.clone().into()
    }

    pub fn position(&self) -> ReadOnly<Duration> {
        self.position.clone().into()
    }

    async fn play_pause(&mut self) -> Result<()> {
        let target_status = *self.target_status.read().await;

        match target_status {
            Status::Playing | Status::Buffering => self.pause().await,
            Status::Paused => self.play().await?,
        }

        Ok(())
    }

    async fn play(&mut self) -> Result<()> {
        if !self.first_track_queried
            && let Some(current_track) = self.tracklist.read().await.current_track()
        {
            let track_url = self.track_url(current_track.id).await?;
            self.query_track_url(&track_url).await?;
            self.first_track_queried = true;
        }

        self.set_target_status(Status::Playing).await;
        self.sink.play();
        Ok(())
    }

    async fn pause(&mut self) {
        self.set_target_status(Status::Paused).await;
        self.sink.pause();
    }

    async fn set_target_status(&self, status: Status) {
        self.broadcast
            .send(Notification::Status { status })
            .unwrap();
    }

    async fn track_url(&self, track_id: u32) -> Result<String> {
        let track_url = self.client.track_url(track_id).await?;
        Ok(track_url)
    }

    async fn query_track_url(&self, track_url: &str) -> Result<()> {
        self.sink.query_track_url(track_url).await
    }

    async fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
        *self.volume.write().await = volume;
    }

    fn broadcast_tracklist(&self, tracklist: Tracklist) {
        self.broadcast
            .send(Notification::CurrentTrackList { tracklist })
            .unwrap();
    }

    async fn jump_forward(&mut self) -> Result<()> {
        let duration = self
            .tracklist
            .read()
            .await
            .current_track()
            .map(|x| Duration::from_secs(x.duration_seconds as u64));

        if let Some(duration) = duration {
            let ten_seconds = Duration::from_secs(10);
            let next_position = self.sink.position().await + ten_seconds;

            if next_position < duration {
                self.sink.seek(next_position).await?;
            } else {
                self.sink.seek(duration).await?;
            }
        }

        Ok(())
    }

    async fn jump_backward(&mut self) -> Result<()> {
        let current_position = self.sink.position().await;

        if current_position.as_millis() < 10000 {
            self.sink.seek(Duration::default()).await?;
        } else {
            let ten_seconds = Duration::from_secs(10);
            let seek_position = current_position - ten_seconds;

            self.sink.seek(seek_position).await?;
        }
        Ok(())
    }

    /// Skip to a specific track in the tracklist.
    async fn skip_to_position(&mut self, new_position: u32, force: bool) -> Result<()> {
        let mut tracklist = self.tracklist.read().await.clone();
        let current_position = tracklist.current_position();
        self.set_target_status(Status::Buffering).await;
        *self.position.write().await = Default::default();

        if !force && new_position < current_position && current_position == 0 {
            self.sink.seek(Duration::default()).await?;
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
            && self.sink.position().await.as_millis() > 1000
        {
            self.sink.seek(Duration::default()).await?;
            return Ok(());
        }

        if let Some(next_track) = tracklist.skip_to_track(new_position) {
            let next_track_url = self.track_url(next_track.id).await?;
            self.sink.clear().await?;
            self.next_track_is_queried = false;
            self.query_track_url(&next_track_url).await?;
            self.first_track_queried = true;
        } else {
            tracklist.reset();
            self.sink.clear().await?;
            self.next_track_is_queried = false;
            self.first_track_queried = false;
            self.set_target_status(Status::Paused).await;
            self.sink.pause();
            *self.position.write().await = Default::default();
        }

        self.broadcast_tracklist(tracklist);

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

    async fn new_queue(&mut self, tracklist: Tracklist) -> Result<()> {
        self.sink.clear().await?;
        self.next_track_is_queried = false;
        self.set_target_status(Status::Buffering).await;

        if let Some(first_track) = tracklist.current_track() {
            let track_url = self.track_url(first_track.id).await?;
            self.query_track_url(&track_url).await?;
            self.first_track_queried = true;
        }

        self.broadcast_tracklist(tracklist);

        Ok(())
    }

    async fn play_track(&mut self, track_id: u32) -> Result<()> {
        let mut track: Track = self.client.track(track_id).await?;
        track.status = TrackStatus::Playing;

        let tracklist = Tracklist {
            list_type: TracklistType::Track(SingleTracklist {
                track_title: track.title.clone(),
                album_id: track.album_id.clone(),
                image: track.image.clone(),
            }),
            queue: vec![track],
        };

        self.new_queue(tracklist).await
    }

    async fn play_album(&mut self, album_id: &str, index: u32) -> Result<()> {
        let album: Album = self.client.album(album_id).await?;

        let unstreambale_tracks_to_index = album
            .tracks
            .iter()
            .take(index as usize)
            .filter(|t| !t.available)
            .count() as u32;

        let mut tracklist = Tracklist {
            queue: album.tracks.into_iter().filter(|t| t.available).collect(),
            list_type: TracklistType::Album(tracklist::AlbumTracklist {
                title: album.title,
                id: album.id,
                image: Some(album.image),
            }),
        };

        tracklist.skip_to_track(index - unstreambale_tracks_to_index);
        self.new_queue(tracklist).await
    }

    async fn play_top_tracks(&mut self, artist_id: u32, index: u32) -> Result<()> {
        let artist = self.client.artist_page(artist_id).await?;
        let tracks = artist.top_tracks;
        let unstreambale_tracks_to_index = tracks
            .iter()
            .take(index as usize)
            .filter(|t| !t.available)
            .count() as u32;

        let mut tracklist = Tracklist {
            queue: tracks.into_iter().filter(|t| t.available).collect(),
            list_type: TracklistType::TopTracks(tracklist::TopTracklist {
                artist_name: artist.name,
                id: artist_id,
                image: artist.image,
            }),
        };

        tracklist.skip_to_track(index - unstreambale_tracks_to_index);
        self.new_queue(tracklist).await
    }

    async fn play_playlist(&mut self, playlist_id: u32, index: u32, shuffle: bool) -> Result<()> {
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
            tracks.shuffle(&mut rand::rng());
        }

        let mut tracklist = Tracklist {
            queue: tracks,
            list_type: TracklistType::Playlist(tracklist::PlaylistTracklist {
                title: playlist.title,
                id: playlist.id,
                image: playlist.image,
            }),
        };

        tracklist.skip_to_track(index - unstreambale_tracks_to_index);
        self.new_queue(tracklist).await
    }

    async fn tick(&mut self) -> Result<()> {
        let target_status = *self.target_status.read().await;
        if target_status != Status::Playing {
            return Ok(());
        }

        let position = self.sink.position().await;

        self.broadcast.send(Notification::Position { position })?;

        let duration = self
            .tracklist
            .read()
            .await
            .current_track()
            .map(|x| x.duration_seconds);

        if let Some(duration) = duration {
            let position = position.as_secs();
            let track_about_to_finish = (duration as i16 - position as i16) < 10;

            if track_about_to_finish && !self.next_track_is_queried {
                let tracklist = self.tracklist.read().await;

                if let Some(next_track) = tracklist.next_track() {
                    let next_track_url = self.track_url(next_track.id).await?;
                    self.query_track_url(&next_track_url).await?;
                    self.first_track_queried = true;
                    self.next_track_is_queried = true;
                }
            }
        }

        Ok(())
    }

    async fn handle_message(&mut self, notification: Notification) -> bool {
        match notification {
            Notification::Play(play) => match play {
                PlayNotification::Album { id, index } => {
                    self.play_album(&id, index).await.unwrap();
                    false
                }
                PlayNotification::Playlist { id, index, shuffle } => {
                    self.play_playlist(id, index, shuffle).await.unwrap();
                    false
                }
                PlayNotification::ArtistTopTracks { artist_id, index } => {
                    self.play_top_tracks(artist_id, index).await.unwrap();
                    false
                }
                PlayNotification::Track { id } => {
                    self.play_track(id).await.unwrap();
                    false
                }
                PlayNotification::Next => {
                    self.next().await.unwrap();
                    false
                }
                PlayNotification::Previous => {
                    self.previous().await.unwrap();
                    false
                }
                PlayNotification::PlayPause => {
                    self.play_pause().await.unwrap();
                    false
                }
                PlayNotification::Play => {
                    self.play().await.unwrap();
                    false
                }
                PlayNotification::DoneBuffering => {
                    self.set_target_status(Status::Playing).await;
                    false
                }
                PlayNotification::Pause => {
                    self.pause().await;
                    false
                }
                PlayNotification::SkipToPosition {
                    new_position,
                    force,
                } => {
                    self.skip_to_position(new_position, force).await.unwrap();
                    false
                }
                PlayNotification::JumpForward => {
                    self.jump_forward().await.unwrap();
                    false
                }
                PlayNotification::JumpBackward => {
                    self.jump_backward().await.unwrap();
                    false
                }
                PlayNotification::Seek { time } => {
                    self.sink.seek(time).await.unwrap();
                    false
                }
                PlayNotification::TrackFinished => {
                    let mut tracklist = self.tracklist.read().await.clone();

                    let current_position = tracklist.current_position();
                    let new_position = current_position + 1;
                    if tracklist.skip_to_track(new_position).is_none() {
                        tracklist.reset();
                        self.set_target_status(Status::Paused).await;
                        self.sink.pause();
                    };
                    self.next_track_is_queried = false;
                    self.first_track_queried = false;
                    self.broadcast_tracklist(tracklist);
                    false
                }
            },
            Notification::Quit => true,
            Notification::Status { status } => {
                *self.target_status.write().await = status;
                false
            }
            Notification::Position { position } => {
                *self.position.write().await = position;
                false
            }
            Notification::CurrentTrackList { tracklist } => {
                *self.tracklist.write().await = tracklist;
                false
            }
            Notification::Message { message: _ } => false,
            Notification::Volume { volume } => {
                self.set_volume(volume).await;
                false
            }
        }
    }

    pub async fn player_loop(&mut self) -> Result<()> {
        let mut receiver = self.broadcast.notify_receiver();
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            select! {
                _ = interval.tick() => {
                    self.tick().await?;
                }

                Ok(notification) = receiver.recv() => {
                    let break_received = self.handle_message(notification).await;
                    if break_received {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
