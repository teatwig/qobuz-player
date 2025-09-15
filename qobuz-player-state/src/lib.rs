use std::sync::Arc;

use database::{Database, LinkRequest};
use qobuz_player_controls::{broadcast::Broadcast, client::Client};
use tokio::sync::Mutex;

pub mod database;

pub struct State {
    pub client: Arc<Client>,
    pub rfid: bool,
    pub web_interface: String,
    pub web_secret: Option<String>,
    pub database: Database,
    pub link_request: Mutex<Option<LinkRequest>>,
    pub broadcast: Arc<Broadcast>,
}

impl State {
    pub async fn new(
        client: Arc<Client>,
        rfid: bool,
        web_interface: String,
        web_secret: Option<String>,
        database: Database,
        broadcast: Arc<Broadcast>,
    ) -> Self {
        let link_request = Mutex::new(None);

        Self {
            client,
            rfid,
            web_interface,
            web_secret,
            database,
            link_request,
            broadcast,
        }
    }
}
