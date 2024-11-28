use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Action {
    Play,
    Pause,
    PlayPause,
    Next,
    Previous,
    Stop,
    Quit,
    SkipTo { num: u32 },
    JumpForward,
    JumpBackward,
    PlayAlbum { album_id: String },
    PlayTrack { track_id: i32 },
    PlayUri { uri: String },
    PlayPlaylist { playlist_id: i64 },
    Search { query: String },
    FetchArtistAlbums { artist_id: i32 },
    FetchPlaylistTracks { playlist_id: i64 },
    FetchUserPlaylists,
}
