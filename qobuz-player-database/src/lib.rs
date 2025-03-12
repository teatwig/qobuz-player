use qobuz_player_controls::AudioQuality;
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use std::{path::PathBuf, sync::OnceLock};
use tracing::debug;

macro_rules! acquire {
    () => {
        POOL.get().unwrap().acquire().await
    };
}

macro_rules! query {
    ($query:expr, $conn:ident, $value:ident) => {
        sqlx::query!($query, $value)
            .execute(&mut *$conn)
            .await
            .expect("database failure")
    };
}

macro_rules! get_one {
    ($query:expr, $return_type:ident, $conn:ident) => {
        sqlx::query_as!($return_type, $query)
            .fetch_one(&mut *$conn)
            .await
    };
}

static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

pub async fn get_pool() -> sqlx::pool::PoolConnection<Sqlite> {
    acquire!().unwrap()
}

pub struct DatabaseCredentials {
    pub username: Option<String>,
    pub password: Option<String>,
}

pub struct DatabaseConfiguration {
    pub max_audio_quality: i64,
}

pub async fn init() {
    let database_url = if let Ok(url) = std::env::var("DATABASE_URL") {
        PathBuf::from(url.replace("sqlite://", ""))
    } else {
        let mut url = dirs::data_local_dir().unwrap();
        url.push("qobuz-player");

        if !url.exists() {
            std::fs::create_dir_all(&url).expect("failed to create database directory");
        }

        url.push("data.db");

        url
    };

    debug!("DATABASE_URL: {}", database_url.to_string_lossy());

    let options = SqliteConnectOptions::new()
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .filename(database_url)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .expect("failed to open database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migration failed");

    POOL.set(pool).expect("error setting static pool");

    create_credentials_row().await;
    create_configuration().await;
}

pub async fn set_username(username: String) {
    let mut conn = acquire!().unwrap();
    query!(
        r#"
            UPDATE credentials
            SET username=?1
            WHERE ROWID = 1
            "#,
        conn,
        username
    );
}

pub async fn set_password(password: String) {
    let md5_pw = format!("{:x}", md5::compute(password));
    let mut conn = acquire!().unwrap();
    query!(
        r#"
            UPDATE credentials
            SET password=?1
            WHERE ROWID = 1
            "#,
        conn,
        md5_pw
    );
}

pub async fn set_max_audio_quality(quality: AudioQuality) {
    let mut conn = acquire!().unwrap();
    let quality_id = quality as i32;

    query!(
        r#"
            UPDATE configuration
            SET max_audio_quality=?1
            WHERE ROWID = 1
            "#,
        conn,
        quality_id
    );
}

async fn create_credentials_row() {
    let mut conn = acquire!().unwrap();
    let rowid = 1;
    query!(
        r#"
            INSERT OR IGNORE INTO credentials (ROWID) VALUES (?1);
            "#,
        conn,
        rowid
    );
}

async fn create_configuration() {
    let mut conn = acquire!().unwrap();
    let rowid = 1;
    query!(
        r#"
            INSERT OR IGNORE INTO configuration (ROWID) VALUES (?1);
            "#,
        conn,
        rowid
    );
}

pub async fn get_credentials() -> DatabaseCredentials {
    let mut conn = acquire!().unwrap();

    get_one!(
        r#"
            SELECT * FROM credentials
            WHERE ROWID = 1;
            "#,
        DatabaseCredentials,
        conn
    )
    .unwrap()
}

pub async fn get_configuration() -> DatabaseConfiguration {
    let mut conn = acquire!().unwrap();

    get_one!(
        r#"
            SELECT * FROM configuration
            WHERE ROWID = 1;
            "#,
        DatabaseConfiguration,
        conn
    )
    .unwrap()
}
