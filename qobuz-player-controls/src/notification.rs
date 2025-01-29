use gstreamer::{ClockTime, State};

use crate::{error, tracklist::Tracklist};

#[derive(Debug, Clone, PartialEq)]
pub enum Notification {
    Status { status: State },
    Position { clock: ClockTime },
    CurrentTrackList { list: Tracklist },
    Quit,
    Error { error: error::Error },
    Volume { volume: f64 },
}
