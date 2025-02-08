use crate::models::{AlbumPage, Playlist, Track, TrackStatus};
use std::collections::BTreeMap;
use tracing::instrument;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum TrackListType {
    Album,
    Playlist,
    Track,
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Tracklist {
    pub queue: BTreeMap<u32, Track>,
    pub album: Option<AlbumPage>,
    pub playlist: Option<Playlist>,
    pub list_type: TrackListType,
}

impl Tracklist {
    pub fn total(&self) -> u32 {
        self.queue.len() as u32
    }

    #[instrument(skip(self))]
    pub fn get_album(&self) -> Option<&AlbumPage> {
        self.album.as_ref()
    }

    #[instrument(skip(self))]
    pub fn list_type(&self) -> &TrackListType {
        &self.list_type
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.queue
            .values()
            .find(|&track| track.status == TrackStatus::Playing)
    }
}
