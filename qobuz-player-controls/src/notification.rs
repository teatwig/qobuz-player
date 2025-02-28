use gstreamer::ClockTime;

use crate::{
    error,
    tracklist::{self, Tracklist},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Notification {
    Status { status: tracklist::Status },
    Position { clock: ClockTime },
    CurrentTrackList { list: Tracklist },
    Quit,
    Error { error: error::Error },
    Volume { volume: f64 },
}
