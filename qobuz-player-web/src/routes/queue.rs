use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use leptos::{component, prelude::*, IntoView};
use qobuz_player_controls::{
    models::TrackStatus,
    tracklist::{TrackListType, Tracklist},
};

use crate::{
    components::list::{List, ListItem},
    html,
    page::Page,
    view::render,
};

pub fn routes() -> Router {
    Router::new()
        .route("/queue", get(index))
        .route("/queue/list", get(queue_partial))
        .route("/queue/skip-to/{track_number}", put(skip_to))
}

async fn skip_to(Path(track_number): Path<u32>) -> impl IntoResponse {
    qobuz_player_controls::skip_to_position(track_number, true)
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    let current_tracklist = qobuz_player_controls::current_tracklist().await;

    render(html! {
        <Page active_page=Page::Queue>
            <Queue current_tracklist=current_tracklist />
        </Page>
    })
}

#[component]
fn queue(current_tracklist: Tracklist) -> impl IntoView {
    let entity_title = match current_tracklist.list_type() {
        TrackListType::Album(album) => Some(album.title.clone()),
        TrackListType::Playlist(playlist) => Some(playlist.title.clone()),
        TrackListType::Track => None,
    };

    html! {
        <div
            hx-get="/queue/list"
            hx-trigger="sse:tracklist"
            hx-target="#queue-list"
            class="flex flex-col flex-grow gap-4 max-h-full"
        >
            <div class="p-4 text-center">
                <p class="text-lg">{entity_title}</p>
            </div>

            <div id="queue-list">
                <QueueList current_tracklist=current_tracklist />
            </div>
        </div>
    }
}

async fn queue_partial() -> impl IntoResponse {
    let current_tracklist = qobuz_player_controls::current_tracklist().await;

    render(html! { <QueueList current_tracklist=current_tracklist /> })
}

#[component]
fn queue_list(current_tracklist: Tracklist) -> impl IntoView {
    html! {
        <List>
            {current_tracklist
                .queue
                .into_iter()
                .enumerate()
                .map(|(position, track)| {
                    html! {
                        <ListItem>
                            <button
                                hx-swap="none"
                                hx-put=format!("/queue/skip-to/{}", position)
                                class=format!(
                                    "cursor-pointer flex w-full items-center flex-row gap-4 text-left {}",
                                    if track.status == TrackStatus::Playing {
                                        "bg-blue-800"
                                    } else if track.status == TrackStatus::Played {
                                        "text-gray-500"
                                    } else {
                                        ""
                                    },
                                )
                            >
                                <span class="w-5 text-center">
                                    <span class="text-gray-400">{position + 1}</span>
                                </span>

                                <span class="truncate">{track.title.clone()}</span>
                            </button>
                        </ListItem>
                    }
                })
                .collect::<Vec<_>>()}
        </List>
    }
}
