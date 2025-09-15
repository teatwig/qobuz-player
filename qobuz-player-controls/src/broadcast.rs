use std::time::Duration;

use crate::Result;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::notification::{self, Notification, PlayNotification};

#[derive(Debug, Clone)]
pub struct Controls {
    tx: tokio::sync::mpsc::UnboundedSender<PlayNotification>,
}

impl Controls {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<PlayNotification>) -> Self {
        Self { tx }
    }

    pub fn next(&self) {
        self.tx.send(PlayNotification::Next).unwrap();
    }

    pub fn previous(&self) {
        self.tx.send(PlayNotification::Previous).unwrap();
    }

    pub fn play_pause(&self) {
        self.tx.send(PlayNotification::PlayPause).unwrap();
    }

    pub fn play(&self) {
        self.tx.send(PlayNotification::Play).unwrap();
    }

    pub fn pause(&self) {
        self.tx.send(PlayNotification::Pause).unwrap();
    }

    pub fn play_album(&self, id: &str, index: u32) {
        self.tx
            .send(PlayNotification::Album {
                id: id.to_string(),
                index,
            })
            .unwrap();
    }

    pub fn play_playlist(&self, id: u32, index: u32, shuffle: bool) {
        self.tx
            .send(PlayNotification::Playlist { id, index, shuffle })
            .unwrap();
    }

    pub fn play_track(&self, id: u32) {
        self.tx.send(PlayNotification::Track { id }).unwrap();
    }

    pub fn play_top_tracks(&self, artist_id: u32, index: u32) {
        self.tx
            .send(PlayNotification::ArtistTopTracks { artist_id, index })
            .unwrap();
    }

    pub fn skip_to_position(&self, index: u32, force: bool) {
        self.tx
            .send(PlayNotification::SkipToPosition {
                new_position: index,
                force,
            })
            .unwrap();
    }

    pub fn set_volume(&self, volume: f32) {
        self.tx
            .send(PlayNotification::SetVolume { volume })
            .unwrap();
    }

    pub fn seek(&self, time: Duration) {
        self.tx.send(PlayNotification::Seek { time }).unwrap();
    }

    pub fn jump_forward(&self) {
        self.tx.send(PlayNotification::JumpForward).unwrap();
    }

    pub fn jump_backward(&self) {
        self.tx.send(PlayNotification::JumpBackward).unwrap();
    }
}

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
