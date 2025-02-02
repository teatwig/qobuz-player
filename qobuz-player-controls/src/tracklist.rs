use crate::models::{Album, Playlist, Track, TrackStatus};
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
    pub album: Option<Album>,
    pub playlist: Option<Playlist>,
    pub list_type: TrackListType,
}

impl Tracklist {
    pub fn total(&self) -> u32 {
        if let Some(album) = &self.album {
            album.total_tracks
        } else if let Some(list) = &self.playlist {
            list.tracks_count
        } else {
            self.queue.len() as u32
        }
    }

    #[instrument(skip(self))]
    pub fn get_album(&self) -> Option<&Album> {
        if let Some(c) = self.current_track() {
            if let Some(album) = &c.album {
                Some(album)
            } else {
                self.album.as_ref()
            }
        } else {
            self.album.as_ref()
        }
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
