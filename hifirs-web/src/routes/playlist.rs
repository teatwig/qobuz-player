use axum::{
    extract::Path,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use hifirs_player::service::{Playlist, Track};
use leptos::prelude::*;
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

async fn play(Path(id): Path<i64>) -> impl IntoResponse {
    _ = hifirs_player::play_playlist(id).await;
}

async fn set_favorite(Path(id): Path<String>) -> impl IntoResponse {
    hifirs_player::add_favorite_playlist(&id).await;
}

async fn unset_favorite(Path(id): Path<String>) -> impl IntoResponse {
    hifirs_player::remove_favorite_playlist(&id).await;
}

async fn index(Path(id): Path<i64>, headers: HeaderMap) -> impl IntoResponse {
    let (playlist, now_playing, favorites) = join!(
        hifirs_player::playlist(id),
        hifirs_player::current_track(),
        hifirs_player::user_playlists()
    );

    let now_playing_id = now_playing.map(|track| track.id);
    let is_favorite = favorites.iter().any(|playlist| playlist.id == id as u32);

    let inner = html! { <Playlist playlist=playlist is_favorite=is_favorite now_playing_id=now_playing_id /> };

    let hx_request = is_htmx_request(&headers);
    let html = match hx_request {
        true => inner.into_any(),
        false => html! { <Page active_page=Page::Search>{inner}</Page> }.into_any(),
    };

    render(html)
}

#[component]
fn playlist(playlist: Playlist, is_favorite: bool, now_playing_id: Option<u32>) -> impl IntoView {
    let tracks: Vec<Track> = playlist.tracks.into_iter().map(|x| x.1).collect();

    html! {
        <div
            class="flex flex-col justify-center items-center h-full landscape:flex-row"
            hx-trigger="sse:position"
            hx-swap="outerHTML"
            hx-get=""
        >
            {playlist
                .cover_art
                .map(|cover_art| {
                    html! {
                        <div class="flex justify-center p-8 landscape::max-w-[50%] portrait:max-h-[50%]">
                            <div class="max-h-full rounded-lg shadow-lg aspect-square overflow-clip">
                                <img
                                    src=cover_art
                                    alt=playlist.title.clone()
                                    class="object-contain"
                                />
                            </div>
                        </div>
                    }
                })}
            <div class="flex overflow-auto flex-col gap-4 items-center w-full h-full">
                <div class="flex flex-col gap-2 items-center w-full text-center">
                    <span class="text-lg">{playlist.title}</span>
                </div>

                <div class="flex gap-4">
                    <button
                        class="flex gap-2 items-center py-2 px-4 bg-blue-500 rounded"
                        hx-swap="none"
                        hx-put=format!("{}, play", playlist.id)
                    >

                        <span class="size-6">
                            <Play />
                        </span>
                        <span>Play</span>
                    </button>

                    <ToggleFavorite id=playlist.id.to_string() is_favorite=is_favorite />
                </div>

                <div class="w-full max-w-screen-sm">
                    <ListTracks
                        show_track_number=true
                        now_playing_id=now_playing_id
                        tracks=tracks
                        parent_id=playlist.id.to_string()
                    />
                </div>
            </div>
        </div>
    }
}
