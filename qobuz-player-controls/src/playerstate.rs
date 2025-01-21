use gstreamer::State as GstState;
use tokio::sync::broadcast::Sender as BroadcastSender;

use crate::tracklist::TrackListValue;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub tracklist: TrackListValue,
    pub status: GstState,
    pub target_status: GstState,
    pub quit_sender: BroadcastSender<bool>,
}
