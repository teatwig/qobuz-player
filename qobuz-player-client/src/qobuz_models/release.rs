use crate::qobuz_models::{artist::OtherArtists, Image};
use serde::{Deserialize, Serialize};

use super::artist_page::ArtistName;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseQuery {
    has_more: bool,
    pub items: Vec<Release>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub artist: Artist,
    pub image: Image,
    pub duration: Option<i64>,
    pub dates: Dates,
    pub parental_warning: bool,
    pub rights: Rights,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: u32,
    pub isrc: Option<String>,
    pub title: String,
    pub artist: Artist,
    pub artists: Vec<OtherArtists>,
    pub duration: i64,
    pub parental_warning: bool,
    pub audio_info: AudioInfo,
    pub rights: Rights,
    pub physical_support: PhysicalSupport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalSupport {
    pub media_number: u32,
    pub track_number: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genre {
    pub path: Vec<i64>,
    pub name: String,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    id: i64,
    name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tracks {
    pub items: Vec<Track>,
    has_more: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dates {
    download: String,
    pub original: String,
    stream: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rights {
    purchasable: bool,
    pub streamable: bool,
    downloadable: bool,
    pub hires_streamable: bool,
    hires_purchasable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artist {
    pub id: u32,
    pub name: ArtistName,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioInfo {
    pub maximum_bit_depth: u32,
    pub maximum_channel_count: f32,
    pub maximum_sampling_rate: f32,
}
