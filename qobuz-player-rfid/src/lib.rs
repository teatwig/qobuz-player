use dialoguer::Input;
use qobuz_player_controls::tracklist;
use qobuz_player_state::{
    State,
    database::{LinkRequest, ReferenceType},
};
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

static SCAN_REQUEST: LazyLock<RwLock<Option<LinkRequest>>> = LazyLock::new(|| RwLock::new(None));

pub async fn init(state: Arc<State>) {
    loop {
        match Input::<String>::new()
            .with_prompt("Scan rfid")
            .interact_text()
        {
            Ok(res) => match &*SCAN_REQUEST.read().await {
                Some(request) => match request {
                    LinkRequest::Album(album) => submit_link_album(state.clone(), &res, album),
                    LinkRequest::Playlist(playlist) => {
                        submit_link_playlist(state.clone(), &res, *playlist)
                    }
                },
                None => handle_play_scan(&state, &res).await,
            },

            Err(_) => continue,
        };
    }
}

async fn handle_play_scan(state: &State, res: &str) {
    let reference = match state.database.get_reference(res).await {
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

fn submit_link_album(state: Arc<State>, rfid_id: &str, id: &str) {
    let rfid_id = rfid_id.to_owned();
    let reference = ReferenceType::Album(id.to_owned());

    tokio::spawn(async move {
        state.database.add_rfid_reference(rfid_id, reference).await;

        qobuz_player_controls::send_message(qobuz_player_controls::notification::Message::Success(
            "Link completed".to_string(),
        ));

        set_state(None).await;
    });
}

fn submit_link_playlist(state: Arc<State>, rfid_id: &str, id: u32) {
    let rfid_id = rfid_id.to_owned();
    let reference = ReferenceType::Playlist(id);

    tokio::spawn(async move {
        state.database.add_rfid_reference(rfid_id, reference).await;

        qobuz_player_controls::send_message(qobuz_player_controls::notification::Message::Success(
            "Link completed".to_string(),
        ));
        set_state(None).await;
    });
}
