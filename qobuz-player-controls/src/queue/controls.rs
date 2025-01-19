use gstreamer::State as GstState;
use qobuz_api::client::api::Client;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{
    broadcast::{Receiver as BroadcastReceiver, Sender as BroadcastSender},
    RwLock,
};
use tracing::debug;

use crate::{
    qobuz,
    service::{Album, Artist, Favorites, Playlist, SearchResults, Track, TrackStatus},
};

use super::{TrackListType, TrackListValue};

#[derive(Debug, Clone)]
pub struct PlayerState {
    service: Arc<Client>,
    current_track: Option<Track>,
    tracklist: TrackListValue,
    status: GstState,
    target_status: GstState,
    quit_sender: BroadcastSender<bool>,
}

pub type SafePlayerState = Arc<RwLock<PlayerState>>;

impl PlayerState {
    pub async fn play_album(&mut self, album_id: &str) -> Option<String> {
        debug!("setting up album to play");

        if let Ok(album) = self.service.album(album_id).await {
            let album: Album = album.into();
            let mut tracklist = TrackListValue::new(Some(&album.tracks));
            tracklist.set_album(album);
            tracklist.set_list_type(TrackListType::Album);
            tracklist.set_track_status(1, TrackStatus::Playing);

            self.replace_list(tracklist.clone());

            if let Some(mut entry) = tracklist.queue.first_entry() {
                let first_track = entry.get_mut();

                self.attach_track_url(first_track).await;
                self.set_current_track(first_track.clone());
                self.set_target_status(GstState::Playing);

                first_track.track_url.clone()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn play_track(&mut self, track_id: i32) -> Option<String> {
        debug!("setting up track to play");

        if let Ok(track) = self.service.track(track_id).await {
            let mut track: Track = track.into();
            track.status = TrackStatus::Playing;
            track.number = 1;

            let mut queue = BTreeMap::new();
            queue.entry(track.position).or_insert_with(|| track.clone());

            let mut tracklist = TrackListValue::new(Some(&queue));
            tracklist.set_list_type(TrackListType::Track);

            self.replace_list(tracklist.clone());

            self.attach_track_url(&mut track).await;
            self.set_current_track(track.clone());
            self.set_target_status(GstState::Playing);

            track.track_url.clone()
        } else {
            None
        }
    }

    pub async fn play_playlist(&mut self, playlist_id: i64) -> Option<String> {
        debug!("setting up playlist to play");

        if let Ok(playlist) = self.service.playlist(playlist_id).await {
            let playlist: Playlist = playlist.into();
            let mut tracklist = TrackListValue::new(Some(&playlist.tracks));

            tracklist.set_playlist(playlist);
            tracklist.set_list_type(TrackListType::Playlist);
            tracklist.set_track_status(1, TrackStatus::Playing);

            self.replace_list(tracklist.clone());

            if let Some(mut entry) = tracklist.queue.first_entry() {
                let first_track = entry.get_mut();

                self.attach_track_url(first_track).await;
                self.set_current_track(first_track.clone());
                self.set_target_status(GstState::Playing);

                first_track.track_url.clone()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_status(&mut self, status: GstState) {
        self.status = status;
    }

    pub fn status(&self) -> GstState {
        self.status
    }

    fn set_current_track(&mut self, track: Track) {
        self.current_track = Some(track);
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.tracklist.current_track()
    }

    pub fn current_track_position(&self) -> u32 {
        if let Some(track) = &self.current_track {
            track.position
        } else {
            0
        }
    }

    fn replace_list(&mut self, tracklist: TrackListValue) {
        debug!("replacing tracklist");
        self.tracklist = tracklist;
    }

    pub fn track_list(&self) -> TrackListValue {
        self.tracklist.clone()
    }

    pub fn target_status(&self) -> GstState {
        self.target_status
    }

    pub fn set_target_status(&mut self, target: GstState) {
        self.target_status = target;
    }

    /// Attach a `TrackURL` to the given track.
    async fn attach_track_url(&mut self, track: &mut Track) {
        debug!("fetching track url");
        if let Ok(track_url) = self.service.track_url(track.id as i32, None).await {
            debug!("attaching url information to track");
            track.track_url = Some(track_url.url);
        }
    }

    pub async fn skip_track(&mut self, index: u32) -> Option<String> {
        for t in self.tracklist.queue.values_mut() {
            match t.position.cmp(&index) {
                std::cmp::Ordering::Less => {
                    t.status = TrackStatus::Played;
                }
                std::cmp::Ordering::Equal => {
                    if let Ok(url) = self.service.track_url(t.id as i32, None).await {
                        t.status = TrackStatus::Playing;
                        t.track_url = Some(url.url.clone());
                        self.current_track = Some(t.clone());
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

    pub async fn search_all(&self, query: &str) -> Option<SearchResults> {
        let results = self.service.search_all(query, 20).await.ok();
        results.map(|x| x.into())
    }

    pub async fn favorites(&self) -> Option<Favorites> {
        let results = self.service.favorites(1000).await.ok();
        results.map(|x| x.into())
    }

    pub async fn add_favorite_album(&self, id: &str) {
        _ = self.service.add_favorite_album(id).await;
    }

    pub async fn remove_favorite_album(&self, id: &str) {
        _ = self.service.remove_favorite_album(id).await;
    }

    pub async fn add_favorite_artist(&self, id: &str) {
        _ = self.service.add_favorite_artist(id).await;
    }

    pub async fn remove_favorite_artist(&self, id: &str) {
        _ = self.service.remove_favorite_artist(id).await;
    }

    pub async fn add_favorite_playlist(&self, id: &str) {
        _ = self.service.add_favorite_playlist(id).await;
    }

    pub async fn remove_favorite_playlist(&self, id: &str) {
        _ = self.service.remove_favorite_playlist(id).await;
    }

    pub async fn artist(&self, artist_id: i32) -> Option<Artist> {
        let result = self.service.artist(artist_id, None).await.ok();
        result.map(|x| x.into())
    }

    pub async fn get_album(&self, id: &str) -> Option<Album> {
        let result = self.service.album(id).await.ok();
        result.map(|x| x.into())
    }

    pub async fn get_suggested_albums(&self, id: &str) -> Vec<Album> {
        let result = self.service.suggested_albums(id).await.ok();
        result.map_or(vec![], |result| {
            result.albums.items.into_iter().map(|x| x.into()).collect()
        })
    }

    pub async fn get_similar_artists(&self, id: i32) -> Vec<Artist> {
        let result = self.service.similar_artists(id, None).await.ok();
        result.map_or(vec![], |result| {
            result.items.into_iter().map(|x| x.into()).collect()
        })
    }

    pub async fn get_playlist(&self, playlist_id: i64) -> Option<Playlist> {
        let result = self.service.playlist(playlist_id).await.ok();
        result.map(|x| x.into())
    }

    pub async fn fetch_artist_albums(&self, artist_id: i32) -> Vec<Album> {
        let result = self.service.artist_releases(artist_id, None).await.ok();
        result.map_or(vec![], |result| {
            result.into_iter().map(|release| release.into()).collect()
        })
    }

    pub async fn fetch_playlist_tracks(&self, playlist_id: i64) -> Vec<Track> {
        let result = self.service.playlist(playlist_id).await.ok();
        result.map_or(vec![], |result| {
            result.tracks.map_or(vec![], |tracks| {
                tracks.items.into_iter().map(|track| track.into()).collect()
            })
        })
    }

    pub async fn fetch_user_playlists(&self) -> Vec<Playlist> {
        let result = self.service.user_playlists().await.ok();
        result.map_or(vec![], |x| {
            x.playlists
                .items
                .into_iter()
                .map(|playlist| playlist.into())
                .collect()
        })
    }

    pub fn quitter(&self) -> BroadcastReceiver<bool> {
        self.quit_sender.subscribe()
    }

    pub fn quit(&self) {
        self.quit_sender
            .send(true)
            .expect("failed to send quit message");
    }

    pub async fn new(username: Option<&str>, password: Option<&str>) -> Self {
        let client = Arc::new(
            qobuz::make_client(username, password)
                .await
                .expect("error making client"),
        );

        let tracklist = TrackListValue::new(None);
        let (quit_sender, _) = tokio::sync::broadcast::channel::<bool>(1);

        Self {
            current_track: None,
            service: client,
            tracklist,
            status: gstreamer::State::Null,
            target_status: gstreamer::State::Null,
            quit_sender,
        }
    }
}
