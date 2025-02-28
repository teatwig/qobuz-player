use crate::models::{Track, TrackStatus};
use tracing::instrument;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AlbumTracklist {
    pub title: String,
    pub id: String,
    pub image: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PlaylistTracklist {
    pub title: String,
    pub id: u32,
    pub image: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TopTracklist {
    pub artist_name: String,
    pub id: u32,
    pub image: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SingleTracklist {
    pub track_title: String,
    pub album_id: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum TracklistType {
    Album(AlbumTracklist),
    Playlist(PlaylistTracklist),
    TopTracks(TopTracklist),
    Track(SingleTracklist),
    #[default]
    None,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Tracklist {
    pub queue: Vec<Track>,
    pub list_type: TracklistType,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Playing,
    Paused,
    #[default]
    Stopped,
}

impl Tracklist {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn total(&self) -> u32 {
        self.queue.len() as u32
    }

    pub fn currently_playing(&self) -> Option<u32> {
        self.queue
            .iter()
            .find(|t| t.status == TrackStatus::Playing)
            .map(|x| x.id)
    }

    pub fn current_position(&self) -> u32 {
        self.queue
            .iter()
            .enumerate()
            .find(|t| t.1.status == TrackStatus::Playing)
            .map(|x| x.0 as u32)
            .unwrap_or(0)
    }

    #[instrument(skip(self))]
    pub fn list_type(&self) -> &TracklistType {
        &self.list_type
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.queue.iter().find(|t| t.status == TrackStatus::Playing)
    }
}
