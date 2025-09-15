use assets::static_handler;
use axum::{
    Router,
    extract::State,
    response::{Sse, sse::Event},
    routing::get,
};
use futures::stream::Stream;
use leptos::*;
use leptos::{html::*, prelude::RenderHtml};
use qobuz_player_controls::{
    PositionReceiver, Status, StatusReceiver, TracklistReceiver, VolumeReceiver,
    broadcast::Broadcast,
    models::{Album, AlbumSimple, Favorites, Playlist},
    notification::Notification,
};
use routes::{
    album, artist, auth, controls, discover, favorites, now_playing, playlist, queue, search,
};
use std::{convert::Infallible, sync::Arc};
use tokio::{
    join,
    sync::broadcast::{self, Sender},
};
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

mod assets;
mod components;
mod icons;
mod page;
mod routes;
mod view;

pub async fn init(
    state: Arc<qobuz_player_state::State>,
    position_receiver: PositionReceiver,
    tracklist_receiver: TracklistReceiver,
    volume_receiver: VolumeReceiver,
    status_receiver: StatusReceiver,
) {
    let listener = tokio::net::TcpListener::bind(&state.web_interface)
        .await
        .unwrap();

    let router = create_router(
        state.clone(),
        position_receiver,
        tracklist_receiver,
        volume_receiver,
        status_receiver,
    )
    .await;

    axum::serve(listener, router).await.unwrap();
}

async fn create_router(
    state: Arc<qobuz_player_state::State>,
    position_receiver: PositionReceiver,
    tracklist_receiver: TracklistReceiver,
    volume_receiver: VolumeReceiver,
    status_receiver: StatusReceiver,
) -> Router {
    let (tx, _rx) = broadcast::channel::<ServerSentEvent>(100);
    let shared_state = Arc::new(AppState {
        tx: tx.clone(),
        player_state: state.clone(),
        position_receiver: position_receiver.clone(),
        tracklist_receiver: tracklist_receiver.clone(),
        volume_receiver: volume_receiver.clone(),
        status_receiver: status_receiver.clone(),
    });
    tokio::spawn(background_task(
        tx,
        state.broadcast.clone(),
        position_receiver,
        tracklist_receiver,
        volume_receiver,
        status_receiver,
    ));

    axum::Router::new()
        .route("/sse", get(sse_handler))
        .merge(now_playing::routes())
        .merge(search::routes())
        .merge(album::routes())
        .merge(artist::routes())
        .merge(playlist::routes())
        .merge(favorites::routes())
        .merge(queue::routes())
        .merge(discover::routes())
        .merge(controls::routes())
        .layer(axum::middleware::from_fn_with_state(
            shared_state.clone(),
            auth::auth_middleware,
        ))
        .route("/assets/{*file}", get(static_handler))
        .merge(auth::routes())
        .with_state(shared_state.clone())
}

async fn background_task(
    tx: Sender<ServerSentEvent>,
    receiver: Arc<Broadcast>,
    mut position: PositionReceiver,
    mut tracklist: TracklistReceiver,
    mut volume: VolumeReceiver,
    mut status: StatusReceiver,
) {
    let mut receiver = receiver.notify_receiver();

    loop {
        tokio::select! {
            Ok(_) = position.changed() => {
                let position_duration = position.borrow_and_update();
                let event = ServerSentEvent {
                    event_name: "position".into(),
                    event_data: position_duration.as_millis().to_string(),
                };

                _ = tx.send(event);
            },
            Ok(_) = tracklist.changed() => {
                _ = tracklist.borrow_and_update();
                let event = ServerSentEvent {
                    event_name: "tracklist".into(),
                    event_data: Default::default(),
                };
                _ = tx.send(event);
            },
            Ok(_) = volume.changed() => {
                let volume = *volume.borrow_and_update();
                let volume = (volume * 100.0) as u32;
                let event = ServerSentEvent {
                    event_name: "volume".into(),
                    event_data: volume.to_string(),
                };
                _ = tx.send(event);
            }
            Ok(_) = status.changed() => {
                let status = status.borrow_and_update();
                let message_data = match *status {
                    Status::Paused => "pause",
                    Status::Playing => "play",
                    Status::Buffering => "buffering",
                };

                let event = ServerSentEvent {
                    event_name: "status".into(),
                    event_data: message_data.into(),
                };
                _ = tx.send(event);
            }
            notification = receiver.recv() => {
                if let Ok(notification) = notification {
                    match notification {
                        Notification::Message { message } => {
                            let toast = components::toast(message.clone()).to_html();

                            let event = match message {
                                qobuz_player_controls::notification::Message::Error(_) => ServerSentEvent {
                                    event_name: "error".into(),
                                    event_data: toast,
                                },
                                qobuz_player_controls::notification::Message::Warning(_) => {
                                    ServerSentEvent {
                                        event_name: "warn".into(),
                                        event_data: toast,
                                    }
                                }
                                qobuz_player_controls::notification::Message::Success(_) => {
                                    ServerSentEvent {
                                        event_name: "success".into(),
                                        event_data: toast,
                                    }
                                }
                                qobuz_player_controls::notification::Message::Info(_) => ServerSentEvent {
                                    event_name: "info".into(),
                                    event_data: toast,
                                },
                            };
                            _ = tx.send(event);
                        }
                        Notification::Play(_) => (),
                    };
                }
            }
        }
    }
}

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> (
    axum::http::HeaderMap,
    Sse<impl Stream<Item = Result<Event, Infallible>>>,
) {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => Some(Ok(Event::default()
            .event(event.event_name)
            .data(event.event_data))),
        Err(_) => None,
    });

    let mut headers = axum::http::HeaderMap::new();
    headers.insert("X-Accel-Buffering", "no".parse().unwrap());

    (headers, Sse::new(stream))
}

pub(crate) struct AppState {
    tx: Sender<ServerSentEvent>,
    pub(crate) player_state: Arc<qobuz_player_state::State>,
    pub(crate) position_receiver: PositionReceiver,
    pub(crate) tracklist_receiver: TracklistReceiver,
    pub(crate) status_receiver: StatusReceiver,
    pub(crate) volume_receiver: VolumeReceiver,
}

impl AppState {
    pub async fn get_favorites(&self) -> Favorites {
        self.player_state.client.favorites().await.unwrap()
    }

    pub async fn get_album(&self, id: &str) -> AlbumData {
        let (album, suggested_albums) = join!(
            self.player_state.client.album(id),
            self.player_state.client.suggested_albums(id),
        );

        let album = album.unwrap();
        let suggested_albums = suggested_albums.unwrap();

        AlbumData {
            album,
            suggested_albums,
        }
    }

    pub async fn is_album_favorite(&self, id: &str) -> bool {
        let favorites = self.get_favorites().await;
        favorites.albums.iter().any(|album| album.id == id)
    }
}

#[derive(Clone)]
pub(crate) struct AlbumData {
    pub album: Album,
    pub suggested_albums: Vec<AlbumSimple>,
}

#[derive(Clone)]
pub(crate) struct ServerSentEvent {
    event_name: String,
    event_data: String,
}

#[derive(Clone)]
pub(crate) struct Discover {
    pub albums: Vec<(String, Vec<AlbumSimple>)>,
    pub playlists: Vec<(String, Vec<Playlist>)>,
}
