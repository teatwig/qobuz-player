use axum::{
    extract::Path,
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
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/playlist/{id}", get(index))
        .route("/playlist/{id}/tracks", get(tracks_partial))
        .route("/playlist/{id}/set-favorite", put(set_favorite))
        .route("/playlist/{id}/unset-favorite", put(unset_favorite))
        .route("/playlist/{id}/play", put(play))
        .route("/playlist/{id}/play/{track_position}", put(play_track))
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

async fn index(Path(id): Path<i64>) -> impl IntoResponse {
    let (playlist, now_playing, favorites) = join!(
        hifirs_player::playlist(id),
        hifirs_player::current_track(),
        hifirs_player::user_playlists()
    );

    let now_playing_id = now_playing.map(|track| track.id);
    let is_favorite = favorites.iter().any(|playlist| playlist.id == id as u32);

    render(html! {
        <Page active_page=Page::Search>
            <Playlist playlist=playlist is_favorite=is_favorite now_playing_id=now_playing_id />
        </Page>
    })
}

async fn tracks_partial(Path(id): Path<i64>) -> impl IntoResponse {
    let (playlist, now_playing) =
        join!(hifirs_player::playlist(id), hifirs_player::current_track());

    let now_playing_id = now_playing.map(|track| track.id);
    let tracks: Vec<Track> = playlist.tracks.into_iter().map(|x| x.1).collect();

    render(html! { <Tracks now_playing_id=now_playing_id tracks=tracks playlist_id=playlist.id /> })
}

#[component]
fn tracks(now_playing_id: Option<u32>, tracks: Vec<Track>, playlist_id: u32) -> impl IntoView {
    html! {
        <div
            class="w-full"
            hx-trigger="sse:tracklist"
            hx-swap="outerHTML"
            hx-get=format!("/playlist/{}/tracks", playlist_id)
        >
            <ListTracks
                show_track_number=true
                now_playing_id=now_playing_id
                tracks=tracks
                parent_id=playlist_id.to_string()
            />
        </div>
    }
}

#[component]
fn playlist(playlist: Playlist, is_favorite: bool, now_playing_id: Option<u32>) -> impl IntoView {
    let tracks: Vec<Track> = playlist.tracks.into_iter().map(|x| x.1).collect();

    html! {
        <div class="flex flex-col justify-center items-center sm:p-4">
            <div class="flex flex-wrap gap-4 justify-center items-end p-4 w-full">
                <div class="max-w-sm">
                    <img
                        src=playlist.cover_art
                        alt=playlist.title.clone()
                        class="object-contain rounded-lg size-full aspect-square"
                    />
                </div>

                <div class="flex flex-col flex-grow gap-4 items-center">
                    <div class="flex flex-col gap-2 justify-center items-center w-full text-center">
                        <span class="w-full text-lg sm:text-xl truncate">{playlist.title}</span>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <button
                            class="flex gap-2 justify-center items-center py-2 px-4 bg-blue-500 rounded"
                            hx-swap="none"
                            hx-put=format!("{}/play", playlist.id.clone())
                        >
                            <span class="size-6">
                                <Play />
                            </span>
                            <span>Play</span>
                        </button>

                        <ToggleFavorite id=playlist.id.to_string() is_favorite=is_favorite />
                    </div>
                </div>
            </div>
            <Tracks now_playing_id=now_playing_id tracks=tracks playlist_id=playlist.id />
        </div>
    }
}
