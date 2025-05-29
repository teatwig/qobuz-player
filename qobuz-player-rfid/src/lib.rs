use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};

use cursive::{
    Cursive,
    reexports::crossbeam_channel::Sender,
    view::Nameable,
    views::{Dialog, EditView},
};
use qobuz_player_controls::tracklist;

static INITIATED: AtomicBool = AtomicBool::new(false);
static AWAITING_SCAN: AtomicBool = AtomicBool::new(false);
static SINK: OnceLock<CursiveSender> = OnceLock::new();
type CursiveSender = Sender<Box<dyn FnOnce(&mut Cursive) + Send>>;

pub fn is_initiated() -> bool {
    INITIATED.load(Ordering::Relaxed)
}

pub async fn init() {
    INITIATED.store(true, Ordering::Relaxed);

    let mut siv = cursive::default();

    SINK.set(siv.cb_sink().clone()).expect("error setting sink");

    siv.add_global_callback(cursive::event::Event::CtrlChar('c'), Cursive::quit);

    siv.add_layer(scan_dialog());

    siv.run();
}

fn scan_dialog() -> Dialog {
    Dialog::new()
        .title("Scan RFID")
        .padding_lrtb(1, 1, 1, 0)
        .content(EditView::new().on_submit(submit_scan).with_name("id"))
}

fn submit_scan(s: &mut Cursive, rfid_id: &str) {
    if rfid_id.is_empty() {
        s.add_layer(Dialog::info("Please scan RFID!"));
    } else {
        let rfid_id = rfid_id.to_owned();
        tokio::spawn(async move {
            let reference = match get_reference(&rfid_id).await {
                Some(reference) => reference,
                None => return,
            };
            let now_playing = qobuz_player_controls::current_tracklist().await.list_type;
            match reference {
                Reference::Album(id) => {
                    if let tracklist::TracklistType::Album(now_playing) = now_playing {
                        if now_playing.id == id {
                            qobuz_player_controls::play_pause().await.unwrap();
                            return;
                        }
                    }

                    qobuz_player_controls::play_album(&id, 0).await.unwrap()
                }
                Reference::Playlist(id) => {
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
        });
        s.pop_layer();
        s.add_layer(scan_dialog());
    }
}

#[derive(Debug, Clone)]
pub enum LinkRequest {
    Album(String),
    Playlist(u32),
}

pub async fn link(request: LinkRequest) {
    AWAITING_SCAN.store(true, Ordering::Relaxed);

    let type_string = match request {
        LinkRequest::Album(_) => "album",
        LinkRequest::Playlist(_) => "playlist",
    };

    qobuz_player_controls::send_message(qobuz_player_controls::notification::Message::Info(
        format!("Scan rfid to link {type_string}"),
    ));
    let sink = SINK.get().unwrap();

    sink.send(Box::new(move |s| {
        s.pop_layer();
        s.add_layer(
            Dialog::new()
                .title("Link album to RFID")
                .padding_lrtb(1, 1, 1, 0)
                .content(
                    EditView::new()
                        .on_submit(move |s, rfid_id| match request.clone() {
                            LinkRequest::Album(id) => submit_link_album(s, rfid_id, id.clone()),
                            LinkRequest::Playlist(id) => submit_link_playlist(s, rfid_id, id),
                        })
                        .with_name("id"),
                ),
        )
    }))
    .unwrap();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        if AWAITING_SCAN.load(Ordering::Relaxed) {
            let sink = SINK.get().unwrap();
            sink.send(Box::new(move |s| {
                s.pop_layer();
                s.add_layer(scan_dialog());
            }))
            .unwrap();

            AWAITING_SCAN.store(false, Ordering::Relaxed);
            qobuz_player_controls::send_message(
                qobuz_player_controls::notification::Message::Warning("Scan cancelled".to_string()),
            );
        }
    });
}

fn submit_link_album(s: &mut Cursive, rfid_id: &str, id: String) {
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
        AWAITING_SCAN.store(false, Ordering::Relaxed);
    });

    s.pop_layer();
    s.add_layer(scan_dialog());
}

fn submit_link_playlist(s: &mut Cursive, rfid_id: &str, id: u32) {
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
        AWAITING_SCAN.store(false, Ordering::Relaxed);
    });

    s.pop_layer();
    s.add_layer(scan_dialog());
}

async fn get_reference(id: &str) -> Option<Reference> {
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
