use crate::qobuz_models::{Image, artist::OtherArtists};
use serde::{Deserialize, Serialize};

use super::release::{AudioInfo, Dates, Genre, Label, Rights};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumSuggestion {
    pub id: String,
    pub title: String,
    pub version: Option<String>,
    pub track_count: i64,
    pub artists: Option<Vec<OtherArtists>>,
    pub image: Image,
    pub label: Label,
    pub genre: Genre,
    pub release_type: Option<String>,
    pub release_tags: Option<Vec<String>>,
    pub duration: Option<i64>,
    pub dates: Dates,
    pub parental_warning: bool,
    pub audio_info: AudioInfo,
    pub rights: Rights,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumSuggestionResponse {
    pub algorithm: String,
    pub albums: AlbumSuggestions,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumSuggestions {
    pub limit: i64,
    pub items: Vec<AlbumSuggestion>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlbumOfTheWeekQuery {
    has_more: bool,
    pub items: Vec<AlbumSuggestion>,
}
