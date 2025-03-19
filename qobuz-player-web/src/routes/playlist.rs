use axum::{
    Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, put},
};
use leptos::prelude::*;
use qobuz_player_controls::models::{Playlist, Track};
use tokio::join;

use crate::{
    components::{
        ButtonGroup, ToggleFavorite, button_class,
        list::{ListTracks, TrackNumberDisplay},
        parse_duration,
    },
    html,
    icons::{Link, Play},
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
        .route("/playlist/{id}/link", put(link))
}

async fn play_track(Path((id, track_position)): Path<(u32, u32)>) -> impl IntoResponse {
    qobuz_player_controls::play_playlist(id, track_position, false)
        .await
        .unwrap();
}

async fn play(Path(id): Path<u32>) -> impl IntoResponse {
    qobuz_player_controls::play_playlist(id, 0, false)
        .await
        .unwrap();
}

async fn link(Path(id): Path<u32>) -> impl IntoResponse {
    qobuz_player_rfid::link_playlist(id).await;
}

async fn shuffle(Path(id): Path<u32>) -> impl IntoResponse {
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

async fn index(Path(id): Path<u32>) -> impl IntoResponse {
    let (playlist, favorites) = join!(
        qobuz_player_controls::playlist(id),
        qobuz_player_controls::favorites()
    );

    let playlist = playlist.unwrap();
    let favorites = favorites.unwrap();

    let is_favorite = favorites.playlists.iter().any(|playlist| playlist.id == id);

    render(html! {
        <Page active_page=Page::None>
            <Playlist playlist=playlist is_favorite=is_favorite />
        </Page>
    })
}

async fn tracks_partial(Path(id): Path<u32>) -> impl IntoResponse {
    let playlist = qobuz_player_controls::playlist(id).await.unwrap();

    render(html! { <Tracks tracks=playlist.tracks playlist_id=playlist.id /> })
}

#[component]
fn tracks(tracks: Vec<Track>, playlist_id: u32) -> impl IntoView {
    html! {
        <div
            class="w-full"
            hx-trigger="sse:tracklist"
            hx-swap="outerHTML"
            hx-get=format!("/playlist/{}/tracks", playlist_id)
        >
            <ListTracks
                track_number_display=TrackNumberDisplay::Cover
                tracks=tracks
                show_artist=true
                dim_played=false
                api_call=move |index: usize| format!("/playlist/{}/play/{}", playlist_id, index)
            />
        </div>
    }
}

#[component]
fn playlist(playlist: Playlist, is_favorite: bool) -> impl IntoView {
    let duration = parse_duration(playlist.duration_seconds);
    let rfid = qobuz_player_rfid::is_initiated();

    html! {
        <div class="flex flex-col justify-center items-center">
            <div class="flex flex-wrap gap-4 justify-center items-end p-4 w-full">
                <div class="max-w-sm">
                    <img
                        src=playlist.image
                        alt=playlist.title.clone()
                        class="object-contain rounded-lg size-full"
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
                            <ButtonGroup>
                                <button
                                    class=button_class()
                                    hx-swap="none"
                                    hx-put=format!("{}/play", playlist.id.clone())
                                >
                                    <span class="size-6">
                                        <Play />
                                    </span>
                                    <span>Play</span>
                                </button>

                                <button
                                    class=button_class()
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

                                {rfid
                                    .then_some(
                                        html! {
                                            <button
                                                class=button_class()
                                                hx-swap="none"
                                                hx-put=format!("{}/link", playlist.id.clone())
                                            >
                                                <span class="size-6">
                                                    <Link />
                                                </span>
                                                <span>Link RFID</span>
                                            </button>
                                        },
                                    )}

                            </ButtonGroup>
                        }
                    }
                </div>
            </div>
            <div class="sm:p-4">
                <Tracks tracks=playlist.tracks playlist_id=playlist.id />
            </div>
        </div>
    }
}
