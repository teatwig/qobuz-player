use database::{Database, LinkRequest};
use tokio::sync::Mutex;

pub mod database;

pub struct State {
    pub rfid: bool,
    pub web_interface: String,
    pub web_secret: Option<String>,
    pub database: Database,
    pub link_request: Mutex<Option<LinkRequest>>,
}

impl State {
    pub async fn new(rfid: bool, web_interface: String, web_secret: Option<String>) -> Self {
        let database = Database::new().await;
        let link_request = Mutex::new(None);

        Self {
            rfid,
            web_interface,
            web_secret,
            database,
            link_request,
        }
    }
}
