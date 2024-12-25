use axum::{
    extract::Path,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use hifirs_player::service::{Album, Track};
use leptos::{component, prelude::*, IntoView};
use std::sync::Arc;
use tokio::join;

use crate::{
    components::{list::ListTracks, ToggleFavorite},
    html,
    icons::Play,
    is_htmx_request,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:id", get(index))
        .route("/:id/set-favorite", put(set_favorite))
        .route("/:id/unset-favorite", put(unset_favorite))
        .route("/:id/play", put(play))
        .route("/:id/play/:track_position", put(play_track))
}

async fn play_track(Path((id, track_position)): Path<(String, u32)>) -> impl IntoResponse {
    _ = hifirs_player::play_album(&id).await;
    _ = hifirs_player::skip(track_position, true).await;
}

async fn set_favorite(Path(id): Path<String>) -> impl IntoResponse {
    hifirs_player::add_favorite_album(&id).await;
    render(html! { <ToggleFavorite id=id is_favorite=true /> })
}

async fn unset_favorite(Path(id): Path<String>) -> impl IntoResponse {
    hifirs_player::remove_favorite_album(&id).await;
    render(html! { <ToggleFavorite id=id is_favorite=false /> })
}

async fn play(Path(id): Path<String>) -> impl IntoResponse {
    _ = hifirs_player::play_album(&id).await;
}

async fn index(Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    let (album, now_playing, favorites) = join!(
        hifirs_player::album(&id),
        hifirs_player::current_track(),
        hifirs_player::favorites()
    );

    let now_playing_id = now_playing.map(|track| track.id);
    let is_favorite = favorites.albums.iter().any(|album| album.id == id);

    let inner =
        html! { <Album album=album is_favorite=is_favorite now_playing_id=now_playing_id /> };

    let hx_request = is_htmx_request(&headers);
    let html = match hx_request {
        true => inner.into_any(),
        false => html! { <Page active_page=Page::Search>{inner}</Page> }.into_any(),
    };

    render(html)
}

#[component]
fn album(album: Album, is_favorite: bool, now_playing_id: Option<u32>) -> impl IntoView {
    let tracks: Vec<Track> = album.tracks.into_iter().map(|x| x.1).collect();

    html! {
        <div
            hx-get=""
            hx-trigger="sse:tracklist"
            hx-swap="outerHTML"
            class="flex flex-col justify-center items-center h-full landscape:flex-row"
        >
            <div class="flex justify-center p-8 landscape::max-w-[50%] portrait:max-h-[50%]">
                <div class="max-h-full rounded-lg shadow-lg aspect-square overflow-clip">
                    <img src=album.cover_art alt=album.title.clone() class="object-contain" />
                </div>
            </div>

            <div class="flex overflow-auto flex-col gap-4 items-center w-full h-full">
                <div class="flex flex-col gap-2 items-center w-full text-center">
                    <span class="w-full text-lg truncate">{album.title}</span>
                    <span class="text-gray-400">{album.release_year}</span>
                </div>

                <div class="flex gap-4">
                    <button
                        class="flex gap-2 items-center py-2 px-4 bg-blue-500 rounded"
                        hx-swap="none"
                        hx-put=format!("{}/play", album.id.clone())
                    >
                        <span class="size-6">
                            <Play />
                        </span>
                        <span>Play</span>
                    </button>

                    <ToggleFavorite id=album.id.clone() is_favorite=is_favorite />
                </div>

                <div class="w-full max-w-screen-sm">
                    <ListTracks
                        show_track_number=true
                        now_playing_id=now_playing_id
                        tracks=tracks
                        parent_id=album.id.clone()
                    />
                </div>
            </div>
        </div>
    }
}
