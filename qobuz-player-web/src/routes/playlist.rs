use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use leptos::prelude::*;
use qobuz_player_controls::models::{Playlist, Track};
use tokio::join;

use crate::{
    components::{
        list::{ListTracks, TrackNumberDisplay},
        parse_duration, ToggleFavorite,
    },
    html,
    icons::Play,
    page::Page,
    view::render,
};

pub fn routes() -> Router {
    Router::new()
        .route("/playlist/{id}", get(index))
        .route("/playlist/{id}/tracks", get(tracks_partial))
        .route("/playlist/{id}/set-favorite", put(set_favorite))
        .route("/playlist/{id}/unset-favorite", put(unset_favorite))
        .route("/playlist/{id}/play", put(play))
        .route("/playlist/{id}/play/shuffle", put(shuffle))
        .route("/playlist/{id}/play/{track_position}", put(play_track))
}

async fn play_track(Path((id, track_position)): Path<(i64, u32)>) -> impl IntoResponse {
    qobuz_player_controls::play_playlist(id, track_position, false)
        .await
        .unwrap();
}

async fn play(Path(id): Path<i64>) -> impl IntoResponse {
    qobuz_player_controls::play_playlist(id, 0, false)
        .await
        .unwrap();
}

async fn shuffle(Path(id): Path<i64>) -> impl IntoResponse {
    qobuz_player_controls::play_playlist(id, 0, true)
        .await
        .unwrap();
}

async fn set_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::add_favorite_playlist(&id)
        .await
        .unwrap();
    render(html! { <ToggleFavorite id=id is_favorite=true /> })
}

async fn unset_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::remove_favorite_playlist(&id)
        .await
        .unwrap();
    render(html! { <ToggleFavorite id=id is_favorite=false /> })
}

async fn index(Path(id): Path<i64>) -> impl IntoResponse {
    let (playlist, tracklist, favorites) = join!(
        qobuz_player_controls::playlist(id),
        qobuz_player_controls::current_tracklist(),
        qobuz_player_controls::favorites()
    );

    let playlist = playlist.unwrap();
    let favorites = favorites.unwrap();

    let now_playing_id = tracklist.currently_playing();
    let is_favorite = favorites
        .playlists
        .iter()
        .any(|playlist| playlist.id == id as u32);

    let current_tracklist = qobuz_player_controls::current_tracklist().await;

    render(html! {
        <Page active_page=Page::None current_tracklist=current_tracklist.list_type>
            <Playlist playlist=playlist is_favorite=is_favorite now_playing_id=now_playing_id />
        </Page>
    })
}

async fn tracks_partial(Path(id): Path<i64>) -> impl IntoResponse {
    let (playlist, tracklist) = join!(
        qobuz_player_controls::playlist(id),
        qobuz_player_controls::current_tracklist(),
    );
    let playlist = playlist.unwrap();

    let now_playing_id = tracklist.currently_playing();

    render(
        html! { <Tracks now_playing_id=now_playing_id tracks=playlist.tracks playlist_id=playlist.id /> },
    )
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
                track_number_display=TrackNumberDisplay::Cover
                now_playing_id=now_playing_id
                tracks=tracks
                parent_id=playlist_id.to_string()
                show_artist=true
            />
        </div>
    }
}

#[component]
fn playlist(playlist: Playlist, is_favorite: bool, now_playing_id: Option<u32>) -> impl IntoView {
    let duration = parse_duration(playlist.duration_seconds);

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
                        <span class="text-lg sm:text-xl">{playlist.title}</span>
                        <span class="text-gray-400 sm:text-lg">
                            {format!("{} minutes", duration.minutes)}
                        </span>
                    </div>

                    {
                        html! {
                            <div class=format!(
                                "grid {} gap-4",
                                if playlist.is_owned { "grid-cols-2" } else { "grid-cols-3" },
                            )>
                                <button
                                    class="flex gap-2 justify-center items-center py-2 px-4 bg-blue-500 rounded cursor-pointer"
                                    hx-swap="none"
                                    hx-put=format!("{}/play", playlist.id.clone())
                                >
                                    <span class="size-6">
                                        <Play />
                                    </span>
                                    <span>Play</span>
                                </button>

                                <button
                                    class="flex gap-2 justify-center items-center py-2 px-4 bg-blue-500 rounded cursor-pointer"
                                    hx-swap="none"
                                    hx-put=format!("{}/play/shuffle", playlist.id.clone())
                                >
                                    <span class="size-6">
                                        <Play />
                                    </span>
                                    <span>Shuffle</span>
                                </button>

                                {(!playlist.is_owned)
                                    .then_some({
                                        html! {
                                            <ToggleFavorite
                                                id=playlist.id.to_string()
                                                is_favorite=is_favorite
                                            />
                                        }
                                    })}

                            </div>
                        }
                    }
                </div>
            </div>
            <Tracks now_playing_id=now_playing_id tracks=playlist.tracks playlist_id=playlist.id />
        </div>
    }
}
