use crate::Result;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Debug, Clone, PartialEq)]
pub enum Notification {
    Error(String),
    Warning(String),
    Success(String),
    Info(String),
}

#[derive(Debug)]
pub struct NotificationBroadcast {
    tx: Sender<Notification>,
    rx: Receiver<Notification>,
}

impl NotificationBroadcast {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(20);
        Self { tx, rx }
    }
    pub fn send(&self, notification: Notification) -> Result<()> {
        self.tx.send(notification)?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<Notification> {
        self.rx.resubscribe()
    }

    pub fn send_message(&self, message: Notification) {
        self.tx.send(message).unwrap();
    }
}

impl Default for NotificationBroadcast {
    fn default() -> Self {
        Self::new()
    }
}
