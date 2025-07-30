use database::{Database, LinkRequest};
use qobuz_player_controls::tracklist::Tracklist;
use tokio::sync::{Mutex, RwLock};

pub mod database;

pub struct State {
    pub rfid: bool,
    pub web_interface: String,
    pub web_secret: Option<String>,
    pub database: Database,
    pub link_request: Mutex<Option<LinkRequest>>,
    pub tracklist: RwLock<Tracklist>,
}

impl State {
    pub async fn new(
        rfid: bool,
        web_interface: String,
        web_secret: Option<String>,
        tracklist: Tracklist,
        database: Database,
    ) -> Self {
        let link_request = Mutex::new(None);
        let tracklist = RwLock::new(tracklist);

        Self {
            rfid,
            web_interface,
            web_secret,
            database,
            link_request,
            tracklist,
        }
    }
}
