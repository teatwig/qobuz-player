use dialoguer::Input;
use qobuz_player_controls::tracklist;
use std::sync::LazyLock;
use tokio::sync::RwLock;

static SCAN_REQUEST: LazyLock<RwLock<Option<LinkRequest>>> = LazyLock::new(|| RwLock::new(None));

pub async fn init() {
    loop {
        match Input::<String>::new()
            .with_prompt("Scan rfid")
            .interact_text()
        {
            Ok(res) => match &*SCAN_REQUEST.read().await {
                Some(request) => match request {
                    LinkRequest::Album(album) => submit_link_album(&res, album),
                    LinkRequest::Playlist(playlist) => submit_link_playlist(&res, *playlist),
                },
                None => handle_play_scan(&res).await,
            },

            Err(_) => continue,
        };
    }
}

async fn handle_play_scan(res: &str) {
    let reference = match get_reference(res).await {
        Some(reference) => reference,
        None => {
            return;
        }
    };

    let now_playing = qobuz_player_controls::current_tracklist().await.list_type;
    match reference {
        LinkRequest::Album(id) => {
            if let tracklist::TracklistType::Album(now_playing) = now_playing {
                if now_playing.id == id {
                    qobuz_player_controls::play_pause().await.unwrap();
                    return;
                }
            }

            qobuz_player_controls::play_album(&id, 0).await.unwrap()
        }
        LinkRequest::Playlist(id) => {
            if let tracklist::TracklistType::Playlist(now_playing) = now_playing {
                if now_playing.id == id {
                    qobuz_player_controls::play_pause().await.unwrap();
                    return;
                }
            }
            qobuz_player_controls::play_playlist(id, 0, false)
                .await
                .unwrap()
        }
    }
}

pub async fn link(request: LinkRequest) {
    set_state(Some(request.clone())).await;

    let type_string = match request {
        LinkRequest::Album(_) => "album",
        LinkRequest::Playlist(_) => "playlist",
    };

    qobuz_player_controls::send_message(qobuz_player_controls::notification::Message::Info(
        format!("Scan rfid to link {type_string}"),
    ));

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let request_ongoing = { SCAN_REQUEST.read().await.is_some() };

        if request_ongoing {
            qobuz_player_controls::send_message(
                qobuz_player_controls::notification::Message::Warning("Scan cancelled".to_string()),
            );
            set_state(None).await;
        }
    });
}

async fn set_state(request: Option<LinkRequest>) {
    let mut request_lock = SCAN_REQUEST.write().await;
    *request_lock = request;
}

fn submit_link_album(rfid_id: &str, id: &str) {
    let rfid_id = rfid_id.to_owned();
    let id = id.to_owned();

    tokio::spawn(async move {
        let mut conn = qobuz_player_database::get_pool().await;

        let reference_id = Some(id);
        sqlx::query_as!(
                    RFIDReference,
                    "INSERT INTO rfid_references (id, reference_type, album_id, playlist_id) VALUES ($1, $2, $3, $4) ON CONFLICT(id) DO UPDATE SET reference_type = excluded.reference_type, album_id = excluded.album_id, playlist_id = excluded.playlist_id RETURNING *;",
                    rfid_id,
                    1,
                    reference_id,
                    None::<u32>,
                ).fetch_one(&mut *conn).await.unwrap();

        qobuz_player_controls::send_message(qobuz_player_controls::notification::Message::Success(
            "Link completed".to_string(),
        ));

        set_state(None).await;
    });
}

fn submit_link_playlist(rfid_id: &str, id: u32) {
    let rfid_id = rfid_id.to_owned();

    tokio::spawn(async move {
        let mut conn = qobuz_player_database::get_pool().await;

        let reference_id = Some(id);
        sqlx::query_as!(
                    RFIDReference,
                    "INSERT INTO rfid_references (id, reference_type, album_id, playlist_id) VALUES ($1, $2, $3, $4) ON CONFLICT(id) DO UPDATE SET reference_type = excluded.reference_type, album_id = excluded.album_id, playlist_id = excluded.playlist_id RETURNING *;",
                    rfid_id,
                    2,
                    None::<String>,
                    reference_id,
                ).fetch_one(&mut *conn).await.unwrap();

        qobuz_player_controls::send_message(qobuz_player_controls::notification::Message::Success(
            "Link completed".to_string(),
        ));
        set_state(None).await;
    });
}

async fn get_reference(id: &str) -> Option<LinkRequest> {
    let mut conn = qobuz_player_database::get_pool().await;

    let db_reference = match sqlx::query_as!(
        RFIDReference,
        "SELECT * FROM rfid_references WHERE ID = $1;",
        id
    )
    .fetch_one(&mut *conn)
    .await
    {
        Ok(res) => res,
        Err(_) => return None,
    };

    match db_reference.reference_type {
        ReferenceType::Album => Some(LinkRequest::Album(db_reference.album_id.unwrap())),
        ReferenceType::Playlist => Some(LinkRequest::Playlist(
            db_reference.playlist_id.unwrap() as u32
        )),
    }
}

#[derive(sqlx::FromRow)]
struct RFIDReference {
    #[allow(dead_code)]
    id: String,
    reference_type: ReferenceType,
    album_id: Option<String>,
    playlist_id: Option<i64>,
}

enum ReferenceType {
    Album = 1,
    Playlist = 2,
}

impl From<i64> for ReferenceType {
    fn from(value: i64) -> Self {
        match value {
            1 => ReferenceType::Album,
            2 => ReferenceType::Playlist,
            _ => panic!("Unable to parse reference type!"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LinkRequest {
    Album(String),
    Playlist(u32),
}
