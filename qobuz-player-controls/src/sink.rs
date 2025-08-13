use crate::{Broadcast, Time, error::Error};
use gstreamer::{ClockTime, Element, SeekFlags, Structure, prelude::*};
use std::{str::FromStr, sync::Arc};

static USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
];

#[derive(Debug)]
pub struct Sink {
    playbin: Element,
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl Sink {
    pub fn seek(&self, time: Time) -> Result<()> {
        let clock_time = ClockTime::from_mseconds(time.mseconds());

        let flags = SeekFlags::FLUSH | SeekFlags::TRICKMODE_KEY_UNITS;
        self.playbin.seek_simple(flags, clock_time)?;
        Ok(())
    }

    pub fn position(&self) -> Option<Time> {
        self.playbin
            .query_position::<ClockTime>()
            .map(|clock_time| Time::from_mseconds(clock_time.mseconds()))
    }

    pub fn duration(&self) -> Option<Time> {
        self.playbin
            .query_duration::<ClockTime>()
            .map(|clock_time| Time::from_mseconds(clock_time.mseconds()))
    }

    pub fn set_state(&self, state: gstreamer::State) -> Result<()> {
        self.playbin.set_state(state)?;
        Ok(())
    }

    pub fn query_track_url(&self, track_url: &str) -> Result<()> {
        self.playbin.set_property("uri", track_url);
        self.set_state(gstreamer::State::Playing)?;
        Ok(())
    }

    pub fn state(&self) -> gstreamer::State {
        self.playbin.current_state()
    }

    pub fn set_volume(&self, volume: f64) {
        let volume_pow = volume.powi(3);
        self.playbin.set_property("volume", volume_pow);
    }

    pub fn volume(&self) -> f64 {
        self.playbin.property::<f64>("volume").powf(1.0 / 3.0)
    }

    pub(crate) fn bus(&self) -> gstreamer::bus::BusStream {
        self.playbin.bus().unwrap().stream()
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        if let Err(err) = self.playbin.set_state(gstreamer::State::Null) {
            tracing::warn!("Failed to set playbin to NULL state: {:?}", err);
        }
    }
}

pub(crate) fn init_sink(broadcast: Arc<Broadcast>) -> Sink {
    gstreamer::init().expect("error initializing gstreamer");

    let playbin = gstreamer::ElementFactory::make("playbin3")
        .build()
        .expect("error building playbin element");

    playbin.set_property_from_str("flags", "audio+buffering");

    if gstreamer::version().1 >= 22 {
        playbin.connect("element-setup", false, |value| {
            let element = &value[1].get::<gstreamer::Element>().unwrap();

            if element.name().contains("urisourcebin") {
                element.set_property("parse-streams", true);
            }

            None
        });
    }

    playbin.connect("source-setup", false, |value| {
        let element = &value[1].get::<gstreamer::Element>().unwrap();

        if element.name().contains("souphttpsrc") {
            tracing::debug!("new source, changing settings");
            let ua = if rand::random() {
                USER_AGENTS[0]
            } else {
                USER_AGENTS[1]
            };

            element.set_property("user-agent", ua);
            element.set_property("compress", true);
            element.set_property("retries", 10);
            element.set_property("timeout", 30_u32);
            element.set_property(
                "extra-headers",
                Structure::from_str("a-structure, DNT=1, Pragma=no-cache, Cache-Control=no-cache")
                    .expect("failed to make structure from string"),
            )
        }

        None
    });

    playbin.add_property_deep_notify_watch(Some("caps"), true);

    // Connects to the `about-to-finish` signal so the player
    // can setup the next track to play. Enables gapless playback.
    playbin.connect("about-to-finish", false, move |_| {
        tracing::debug!("about to finish");
        broadcast.track_about_to_finish();

        None
    });

    Sink { playbin }
}
