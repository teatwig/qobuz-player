use gstreamer::State as GstState;
use qobuz_api::client::api::Client;
use tokio::sync::broadcast::Sender as BroadcastSender;

use crate::{service::Track, tracklist::TrackListValue};

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub service: Client,
    pub current_track: Option<Track>,
    pub tracklist: TrackListValue,
    pub status: GstState,
    pub target_status: GstState,
    pub quit_sender: BroadcastSender<bool>,
}
