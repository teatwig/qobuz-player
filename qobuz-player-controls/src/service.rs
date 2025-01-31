use qobuz_api::client::Image;
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum TrackStatus {
    Played,
    Playing,
    #[default]
    Unplayed,
    Unplayable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: u32,
    pub number: u32,
    pub title: String,
    pub album: Option<Album>,
    pub artist: Option<Artist>,
    pub duration_seconds: u32,
    pub explicit: bool,
    pub hires_available: bool,
    pub sampling_rate: f32,
    pub bit_depth: u32,
    pub status: TrackStatus,
    pub track_url: Option<String>,
    pub available: bool,
    pub cover_art: Option<String>,
    pub position: u32,
    pub media_number: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: Artist,
    pub release_year: u32,
    pub hires_available: bool,
    pub explicit: bool,
    pub total_tracks: u32,
    pub tracks: BTreeMap<u32, Track>,
    pub available: bool,
    pub cover_art: String,
    pub cover_art_small: String,
    pub duration_seconds: u32,
}

#[derive(Default, Debug, Clone)]
pub struct SearchResults {
    pub query: String,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

#[derive(Default, Debug, Clone)]
pub struct Favorites {
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Artist {
    pub id: u32,
    pub name: String,
    pub image: Option<Image>,
    pub albums: Option<Vec<Album>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Playlist {
    pub title: String,
    pub duration_seconds: u32,
    pub tracks_count: u32,
    pub id: u32,
    pub cover_art: Option<String>,
    pub tracks: BTreeMap<u32, Track>,
}
