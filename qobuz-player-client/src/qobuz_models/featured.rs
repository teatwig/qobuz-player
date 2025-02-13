use serde::{Deserialize, Serialize};

use super::{album_suggestion::AlbumSuggestion, playlist::Playlist};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaturedAlbumResponse {
    total: u32,
    limit: u32,
    offset: u32,
    items: Vec<AlbumSuggestion>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeaturedPlaylistResponse {
    total: u32,
    limit: u32,
    offset: u32,
    items: Vec<Playlist>,
}
