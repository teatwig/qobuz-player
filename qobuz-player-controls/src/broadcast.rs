use std::time::Duration;

use crate::Result;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::notification::{self, Notification, PlayNotification};

#[derive(Debug)]
pub struct Broadcast {
    tx: Sender<Notification>,
    rx: Receiver<Notification>,
}

impl Broadcast {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(20);
        Self { tx, rx }
    }
    pub fn send(&self, notification: Notification) -> Result<()> {
        self.tx.send(notification)?;
        Ok(())
    }

    pub fn quit(&self) {
        self.tx.send(Notification::Quit).unwrap();
    }

    pub fn track_finished(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::TrackFinished))
            .unwrap();
    }

    pub fn next(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Next))
            .unwrap();
    }

    pub fn previous(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Previous))
            .unwrap();
    }

    pub fn play_pause(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::PlayPause))
            .unwrap();
    }

    pub fn play(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Play))
            .unwrap();
    }

    pub fn pause(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::Pause))
            .unwrap();
    }

    pub fn play_album(&self, id: &str, index: u32) {
        self.tx
            .send(Notification::Play(PlayNotification::Album {
                id: id.to_string(),
                index,
            }))
            .unwrap();
    }

    pub fn play_playlist(&self, id: u32, index: u32, shuffle: bool) {
        self.tx
            .send(Notification::Play(PlayNotification::Playlist {
                id,
                index,
                shuffle,
            }))
            .unwrap();
    }

    pub fn play_track(&self, id: u32) {
        self.tx
            .send(Notification::Play(PlayNotification::Track { id }))
            .unwrap();
    }

    pub fn play_top_tracks(&self, artist_id: u32, index: u32) {
        self.tx
            .send(Notification::Play(PlayNotification::ArtistTopTracks {
                artist_id,
                index,
            }))
            .unwrap();
    }

    pub fn skip_to_position(&self, index: u32, force: bool) {
        self.tx
            .send(Notification::Play(PlayNotification::SkipToPosition {
                new_position: index,
                force,
            }))
            .unwrap();
    }

    pub fn set_volume(&self, volume: f32) {
        self.tx.send(Notification::Volume { volume }).unwrap();
    }

    pub fn seek(&self, time: Duration) {
        self.tx
            .send(Notification::Play(PlayNotification::Seek { time }))
            .unwrap();
    }

    pub fn jump_forward(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::JumpForward))
            .unwrap();
    }

    pub fn jump_backward(&self) {
        self.tx
            .send(Notification::Play(PlayNotification::JumpBackward))
            .unwrap();
    }

    pub fn notify_receiver(&self) -> Receiver<Notification> {
        self.rx.resubscribe()
    }

    pub fn send_message(&self, message: notification::Message) {
        self.tx.send(Notification::Message { message }).unwrap();
    }
}

impl Default for Broadcast {
    fn default() -> Self {
        Self::new()
    }
}
