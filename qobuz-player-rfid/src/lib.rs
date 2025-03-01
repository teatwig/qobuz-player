use dialoguer::Input;

pub async fn init() {
    tokio::spawn(async { receive_notifications().await });
}

pub async fn link_album(id: String) {
    let rfid_id = match Input::<String>::new().interact_text() {
        Ok(res) => res,
        Err(_) => return,
    };

    let mut conn = qobuz_player_database::get_pool().await;

    let reference_id = Some(id);

    sqlx::query_as!(
        RFIDReference,
        "INSERT INTO rfid_references (id, reference_type, album_id, playlist_id) VALUES ($1, $2, $3, $4) ON CONFLICT(id) DO UPDATE SET reference_type = excluded.reference_type, album_id = excluded.album_id, playlist_id = excluded.playlist_id RETURNING *;",
        rfid_id,
        1,
        reference_id,
        None::<String>,
    ).fetch_one(&mut *conn).await.unwrap();
}

pub async fn link_playlist(id: u32) {
    let rfid_id = match Input::<String>::new().interact_text() {
        Ok(res) => res,
        Err(_) => return,
    };

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
}

async fn receive_notifications() {
    loop {
        match Input::<String>::new()
            .with_prompt("Scan rfid")
            .interact_text()
        {
            Ok(res) => {
                let reference = match get_reference(res).await {
                    Some(reference) => reference,
                    None => continue,
                };

                match reference {
                    Reference::Album(id) => {
                        qobuz_player_controls::play_album(&id, 0).await.unwrap()
                    }
                    Reference::Playlist(id) => qobuz_player_controls::play_playlist(id, 0, false)
                        .await
                        .unwrap(),
                }
            }
            Err(_) => return,
        };
    }
}

async fn get_reference(id: String) -> Option<Reference> {
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
        ReferenceType::Album => Some(Reference::Album(db_reference.album_id.unwrap())),
        ReferenceType::Playlist => {
            Some(Reference::Playlist(db_reference.playlist_id.unwrap() as u32))
        }
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

#[derive(Debug)]
enum Reference {
    Album(String),
    Playlist(u32),
}
