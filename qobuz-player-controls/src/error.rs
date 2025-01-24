use gstreamer::{glib, StateChangeError};
use snafu::prelude::*;

use crate::notification::Notification;

#[derive(Snafu, Debug, Clone, PartialEq)]
pub enum Error {
    #[snafu(display("{message}"))]
    FailedToPlay {
        message: String,
    },
    #[snafu(display("failed to retrieve a track url"))]
    TrackURL,
    #[snafu(display("failed to seek"))]
    Seek,
    #[snafu(display("sorry, could not resume previous session"))]
    Resume,
    #[snafu(display("{message}"))]
    GStreamer {
        message: String,
    },
    #[snafu(display("{message}"))]
    Client {
        message: String,
    },
    Notification,
    App,
}

impl From<glib::Error> for Error {
    fn from(value: glib::Error) -> Self {
        Error::GStreamer {
            message: value.to_string(),
        }
    }
}

impl From<glib::BoolError> for Error {
    fn from(value: glib::BoolError) -> Self {
        Error::GStreamer {
            message: value.to_string(),
        }
    }
}

impl From<StateChangeError> for Error {
    fn from(value: StateChangeError) -> Self {
        Error::GStreamer {
            message: value.to_string(),
        }
    }
}

impl From<qobuz_api::Error> for Error {
    fn from(value: qobuz_api::Error) -> Self {
        Error::Client {
            message: value.to_string(),
        }
    }
}

impl From<tokio::sync::broadcast::error::SendError<Notification>> for Error {
    fn from(_value: tokio::sync::broadcast::error::SendError<Notification>) -> Self {
        Self::Notification
    }
}

impl From<&gstreamer::message::Error> for Error {
    fn from(value: &gstreamer::message::Error) -> Self {
        let error = format!(
            "Error from {:?}: {} ({:?})",
            value.src().map(|s| s.to_string()),
            value.error(),
            value.debug()
        );
        Error::GStreamer { message: error }
    }
}
