use qobuz_player_controls::{AudioQuality, tracklist::Tracklist};
use serde_json::to_string;
use sqlx::types::Json;
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use std::path::PathBuf;
use tracing::debug;

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new() -> Self {
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

        create_credentials_row(&pool).await;
        create_configuration(&pool).await;

        Self { pool }
    }

    pub async fn set_username(&self, username: String) {
        sqlx::query!(
            r#"
            UPDATE credentials
            SET username=?1
            WHERE ROWID = 1
            "#,
            username
        )
        .execute(&self.pool)
        .await
        .expect("database failure");
    }

    pub async fn set_password(&self, password: String) {
        let md5_pw = format!("{:x}", md5::compute(password));
        sqlx::query!(
            r#"
            UPDATE credentials
            SET password=?1
            WHERE ROWID = 1
            "#,
            md5_pw
        )
        .execute(&self.pool)
        .await
        .expect("database failure");
    }

    pub async fn set_tracklist(&self, tracklist: &Tracklist) {
        let serialized = to_string(&tracklist).unwrap();

        sqlx::query!(
            r#"
           delete from tracklist
        "#
        )
        .execute(&self.pool)
        .await
        .expect("database failure");

        sqlx::query!(
            r#"
            INSERT INTO tracklist (tracklist) VALUES (?1);
        "#,
            serialized
        )
        .execute(&self.pool)
        .await
        .expect("failed to insert new tracklist");
    }

    pub async fn get_tracklist(&self) -> Option<Tracklist> {
        let row = sqlx::query_as!(
            TracklistDb,
            r#"
            SELECT tracklist as "tracklist: Json<Tracklist>" FROM tracklist
        "#
        )
        .fetch_one(&self.pool)
        .await;

        row.ok().map(|x| x.tracklist.0)
    }

    pub async fn set_volume(&self, volume: f32) {
        sqlx::query!(
            r#"
           delete from volume
        "#
        )
        .execute(&self.pool)
        .await
        .expect("database failure");

        sqlx::query!(
            r#"
            INSERT INTO volume (volume) VALUES (?1);
        "#,
            volume
        )
        .execute(&self.pool)
        .await
        .expect("failed to insert new volume");
    }

    pub async fn get_volume(&self) -> Option<f32> {
        let row = sqlx::query_as!(
            VolumeDb,
            r#"
            SELECT volume FROM volume
        "#
        )
        .fetch_one(&self.pool)
        .await;

        row.ok().map(|x| x.volume as f32)
    }

    pub async fn set_max_audio_quality(&self, quality: AudioQuality) {
        let quality_id = quality as i32;

        sqlx::query!(
            r#"
            UPDATE configuration
            SET max_audio_quality=?1
            WHERE ROWID = 1
            "#,
            quality_id
        )
        .execute(&self.pool)
        .await
        .expect("database failure");
    }

    pub async fn get_credentials(&self) -> DatabaseCredentials {
        sqlx::query_as!(
            DatabaseCredentials,
            r#"
            SELECT * FROM credentials
            WHERE ROWID = 1;
            "#
        )
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    pub async fn get_configuration(&self) -> DatabaseConfiguration {
        sqlx::query_as!(
            DatabaseConfiguration,
            r#"
            SELECT * FROM configuration
            WHERE ROWID = 1;
            "#
        )
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    pub async fn add_rfid_reference(&self, rfid_id: String, reference: ReferenceType) {
        match reference {
            ReferenceType::Album(id) => {
                let id = Some(id);

                sqlx::query!(
                    "INSERT INTO rfid_references (id, reference_type, album_id, playlist_id) VALUES ($1, $2, $3, $4) ON CONFLICT(id) DO UPDATE SET reference_type = excluded.reference_type, album_id = excluded.album_id, playlist_id = excluded.playlist_id RETURNING *;",
                    rfid_id,
                    1,
                    id,
                    None::<u32>,
                ).fetch_one(&self.pool).await.unwrap();
            }
            ReferenceType::Playlist(id) => {
                let id = Some(id);

                sqlx::query!(
                    "INSERT INTO rfid_references (id, reference_type, album_id, playlist_id) VALUES ($1, $2, $3, $4) ON CONFLICT(id) DO UPDATE SET reference_type = excluded.reference_type, album_id = excluded.album_id, playlist_id = excluded.playlist_id RETURNING *;",
                    rfid_id,
                    2,
                    None::<String>,
                    id,
                ).fetch_one(&self.pool).await.unwrap();
            }
        }
    }

    pub async fn get_reference(&self, id: &str) -> Option<LinkRequest> {
        let db_reference = match sqlx::query_as!(
            RFIDReference,
            "SELECT * FROM rfid_references WHERE ID = $1;",
            id
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(res) => res,
            Err(_) => return None,
        };

        match db_reference.reference_type {
            ReferenceTypeDatabase::Album => {
                Some(LinkRequest::Album(db_reference.album_id.unwrap()))
            }
            ReferenceTypeDatabase::Playlist => Some(LinkRequest::Playlist(
                db_reference.playlist_id.unwrap() as u32,
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LinkRequest {
    Album(String),
    Playlist(u32),
}

pub enum ReferenceType {
    Album(String),
    Playlist(u32),
}

#[derive(sqlx::FromRow)]
struct RFIDReference {
    #[allow(dead_code)]
    id: String,
    reference_type: ReferenceTypeDatabase,
    album_id: Option<String>,
    playlist_id: Option<i64>,
}

enum ReferenceTypeDatabase {
    Album = 1,
    Playlist = 2,
}

impl From<i64> for ReferenceTypeDatabase {
    fn from(value: i64) -> Self {
        match value {
            1 => ReferenceTypeDatabase::Album,
            2 => ReferenceTypeDatabase::Playlist,
            _ => panic!("Unable to parse reference type!"),
        }
    }
}

pub struct DatabaseCredentials {
    pub username: Option<String>,
    pub password: Option<String>,
}

pub struct DatabaseConfiguration {
    pub max_audio_quality: i64,
}

#[derive(Debug, sqlx::FromRow, serde::Deserialize)]
struct TracklistDb {
    tracklist: Json<Tracklist>,
}

#[derive(Debug, sqlx::FromRow, serde::Deserialize)]
struct VolumeDb {
    volume: f64,
}

async fn create_credentials_row(pool: &Pool<Sqlite>) {
    let rowid = 1;

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO credentials (ROWID) VALUES (?1);
            "#,
        rowid
    )
    .execute(pool)
    .await
    .expect("database failure");
}

async fn create_configuration(pool: &Pool<Sqlite>) {
    let rowid = 1;
    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO configuration (ROWID) VALUES (?1);
            "#,
        rowid
    )
    .execute(pool)
    .await
    .expect("database failure");
}
