use gstreamer::State as GstState;
use tokio::sync::broadcast::Sender as BroadcastSender;

use crate::tracklist::Tracklist;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub status: GstState,
    pub target_status: GstState,
    pub quit_sender: BroadcastSender<bool>,
}
