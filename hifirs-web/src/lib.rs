use assets::static_handler;
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use hifirs_player::notification::Notification;
use leptos::html::*;
use leptos::*;
use routes::{album, artist, favorites, now_playing, playlist, queue, search};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

mod assets;
mod components;
mod icons;
mod page;
mod routes;
mod view;

pub fn is_htmx_request(headers: &axum::http::HeaderMap) -> bool {
    headers.get("HX-Request").is_some() && headers.get("HX-Boosted").is_none()
}

pub async fn init(address: String) {
    println!("Lisening on {address}");
    let router = create_router().await;
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let mut broadcast_receiver = hifirs_player::notify_receiver();

            loop {
                if let Some(message) = broadcast_receiver.next().await {
                    if message == Notification::Quit {
                        break;
                    }
                }
            }
        })
        .await
        .unwrap();
}

async fn create_router() -> Router {
    let (tx, _rx) = broadcast::channel::<ServerSentEvent>(100);
    let shared_state = Arc::new(AppState { tx: tx.clone() });
    tokio::spawn(background_task(tx));

    let router = axum::Router::new()
        .nest("/", now_playing::routes())
        .nest("/search", search::routes())
        .nest("/album", album::routes())
        .nest("/artist", artist::routes())
        .nest("/playlist", playlist::routes())
        .nest("/favorites", favorites::routes())
        .nest("/queue", queue::routes())
        .route("/sse", get(sse_handler))
        .route("/assets/*file", get(static_handler));

    router.with_state(shared_state)
}

async fn background_task(tx: Sender<ServerSentEvent>) {
    let mut receiver = hifirs_player::notify_receiver();

    loop {
        if let Ok(notification) = receiver.recv().await {
            if let Notification::Position { clock } = &notification {
                let event = ServerSentEvent {
                    event_name: "position".into(),
                    event_data: clock.seconds().to_string(),
                };
                _ = tx.send(event);
            }

            if let Notification::CurrentTrackList { list } = &notification {
                let serialized = serde_json::to_string(list).unwrap_or("".into());

                let event = ServerSentEvent {
                    event_name: "tracklist".into(),
                    event_data: serialized,
                };
                _ = tx.send(event);
            }
        }
    }
}

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => Some(Ok(Event::default()
            .event(event.event_name)
            .data(event.event_data))),
        Err(_) => None,
    });

    Sse::new(stream)
}

pub struct AppState {
    pub tx: Sender<ServerSentEvent>,
}

#[derive(Clone)]
pub struct ServerSentEvent {
    event_name: String,
    event_data: String,
}
