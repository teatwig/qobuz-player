use gstreamer::{ClockTime, State};

use crate::{error, tracklist::Tracklist};

#[derive(Debug, Clone, PartialEq)]
pub enum Notification {
    Buffering {
        is_buffering: bool,
        percent: u32,
        target_state: State,
    },
    Status {
        status: State,
    },
    Position {
        clock: ClockTime,
    },
    CurrentTrackList {
        list: Tracklist,
    },
    Quit,
    Loading {
        is_loading: bool,
        target_state: State,
    },
    Error {
        error: error::Error,
    },
    Volume {
        volume: f64,
    },
}
