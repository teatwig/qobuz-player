use crate::models::{self, TrackStatus};
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
pub enum TrackListType {
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
    pub list_type: TrackListType,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Track {
    pub id: u32,
    pub title: String,
    pub number: u32,
    pub explicit: bool,
    pub hires_available: bool,
    pub status: TrackStatus,
    pub image: Option<String>,
    pub duration_seconds: u32,
    pub artist_name: Option<String>,
    pub artist_id: Option<u32>,
    pub album_title: Option<String>,
    pub album_id: Option<String>,
}

impl From<models::Track> for Track {
    fn from(value: models::Track) -> Self {
        let (artist_name, artist_id) = value
            .artist
            .map_or((None, None), |a| (Some(a.name), Some(a.id)));

        let (album_title, album_id) = value
            .album
            .map_or((None, None), |a| (Some(a.title), Some(a.id)));

        Self {
            id: value.id,
            title: value.title,
            number: value.number,
            explicit: value.explicit,
            hires_available: value.hires_available,
            status: TrackStatus::Unplayed,
            image: value.cover_art,
            duration_seconds: value.duration_seconds,
            artist_name,
            artist_id,
            album_title,
            album_id,
        }
    }
}

impl From<qobuz_player_client::qobuz_models::track::Track> for Track {
    fn from(value: qobuz_player_client::qobuz_models::track::Track) -> Self {
        let internal_model: models::Track = value.into();
        internal_model.into()
    }
}

impl From<qobuz_player_client::qobuz_models::artist_page::Track> for Track {
    fn from(value: qobuz_player_client::qobuz_models::artist_page::Track) -> Self {
        let internal_model: models::Track = value.into();
        internal_model.into()
    }
}

impl Tracklist {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
            list_type: Default::default(),
        }
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
    pub fn list_type(&self) -> &TrackListType {
        &self.list_type
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.queue.iter().find(|t| t.status == TrackStatus::Playing)
    }
}
