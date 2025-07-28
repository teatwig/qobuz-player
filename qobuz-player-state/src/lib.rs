use database::Database;

pub mod database;

pub struct State {
    pub rfid: bool,
    pub web_interface: String,
    pub web_secret: Option<String>,
    pub database: Database,
}

impl State {
    pub async fn new(rfid: bool, web_interface: String, web_secret: Option<String>) -> Self {
        let database = Database::new().await;

        Self {
            rfid,
            web_interface,
            web_secret,
            database,
        }
    }
}
