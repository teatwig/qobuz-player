use std::sync::Arc;

use database::{Database, LinkRequest};
use qobuz_player_controls::{
    ReadOnly,
    tracklist::{self, Tracklist},
};
use tokio::sync::{Mutex, RwLock};

pub mod database;

pub struct State {
    pub rfid: bool,
    pub web_interface: String,
    pub web_secret: Option<String>,
    pub database: Database,
    pub link_request: Mutex<Option<LinkRequest>>,
    pub tracklist: ReadOnly<Tracklist>,
    pub target_status: ReadOnly<tracklist::Status>,
}

impl State {
    pub async fn new(
        rfid: bool,
        web_interface: String,
        web_secret: Option<String>,
        tracklist: Arc<RwLock<Tracklist>>,
        database: Database,
        target_status: ReadOnly<tracklist::Status>,
    ) -> Self {
        let link_request = Mutex::new(None);

        Self {
            rfid,
            web_interface,
            web_secret,
            database,
            link_request,
            tracklist: tracklist.into(),
            target_status,
        }
    }
}
