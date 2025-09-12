pub use qobuz_player_client::client::AudioQuality;

use crate::error::Error;
pub mod broadcast;
pub mod client;
pub mod error;
pub mod models;
pub mod notification;
pub mod player;
pub mod readonly;
pub(crate) mod simple_cache;
pub mod sink;
pub mod timer;
pub mod tracklist;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Playing,
    Buffering,
    #[default]
    Paused,
}
