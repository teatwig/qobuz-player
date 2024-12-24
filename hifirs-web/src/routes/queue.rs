use axum::{
    extract::Path,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use hifirs_player::{
    queue::{TrackListType, TrackListValue},
    service::TrackStatus,
};
use leptos::{component, prelude::*, IntoView};
use std::sync::Arc;

use crate::{
    components::{
        list::{List, ListItem},
        Info,
    },
    html, is_htmx_request,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index))
        .route("/skip-to/:track_number", put(skip_to))
}

async fn skip_to(Path(track_number): Path<u32>) -> impl IntoResponse {
    _ = hifirs_player::skip(track_number, true).await;
}

async fn index(headers: HeaderMap) -> impl IntoResponse {
    let current_tracklist = hifirs_player::current_tracklist().await;

    let inner = html! { <Queue current_tracklist=current_tracklist /> };

    let hx_request = is_htmx_request(&headers);
    let html = match hx_request {
        true => inner.into_any(),
        false => html! { <Page active_page=Page::Queue>{inner}</Page> }.into_any(),
    };

    render(html)
}

#[component]
pub fn queue(current_tracklist: TrackListValue) -> impl IntoView {
    let list_type = current_tracklist.list_type.clone();

    let album = current_tracklist.get_album();
    let entity_title = match current_tracklist.list_type() {
        TrackListType::Album => album.map(|album| album.title.clone()),
        TrackListType::Playlist => current_tracklist.playlist.map(|playlist| playlist.title),
        TrackListType::Track => album.map(|album| album.title.clone()),
        TrackListType::Unknown => None,
    };

    html! {
        <div
            hx-get=""
            hx-trigger="sse:tracklist"
            hx-swap="outerHTML"
            class="flex flex-col flex-grow gap-4 max-h-full"
        >
            <div class="p-4 text-center">
                <p class="text-lg">{entity_title}</p>
            </div>

            <List>
                {current_tracklist
                    .queue
                    .into_iter()
                    .map(|x| x.1)
                    .map(|track| {
                        let list_type = list_type.clone();
                        html! {
                            <ListItem>
                                <button
                                    hx-swap="none"
                                    hx-put=format!("/queue/skip-to/{}", track.position)
                                    class=format!(
                                        "flex w-full flex-row gap-4 text-left {}",
                                        if track.status == TrackStatus::Playing {
                                            "bg-blue-800"
                                        } else if track.status == TrackStatus::Played {
                                            "text-gray-800"
                                        } else {
                                            ""
                                        },
                                    )
                                >
                                    <span class="w-5 text-center">

                                        {if list_type == TrackListType::Album
                                            || list_type == TrackListType::Track
                                        {
                                            Some(
                                                html! { <span class="text-gray-400">{track.number}</span> },
                                            )
                                        } else if list_type == TrackListType::Playlist {
                                            Some(
                                                html! {
                                                    <span class="text-gray-400">{track.position}</span>
                                                },
                                            )
                                        } else {
                                            None
                                        }}

                                    </span>

                                    <div class="flex overflow-hidden flex-grow justify-between items-center">
                                        <span class="truncate">{track.title.clone()}</span>
                                        <Info
                                            explicit=track.explicit
                                            hires_available=track.hires_available
                                        />
                                    </div>
                                </button>
                            </ListItem>
                        }
                    })
                    .collect::<Vec<_>>()}
            </List>
        </div>
    }
}
