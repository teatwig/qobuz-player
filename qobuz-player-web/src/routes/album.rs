use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use leptos::{component, prelude::*, IntoView};
use qobuz_player_controls::models::{Album, AlbumSimple, Track};
use tokio::join;

use crate::{
    components::{
        list::{ListAlbumsVertical, ListTracks, TrackNumberDisplay},
        parse_duration, ToggleFavorite,
    },
    html,
    icons::Play,
    page::Page,
    view::render,
};

pub fn routes() -> Router {
    Router::new()
        .route("/album/{id}", get(index))
        .route("/album/{id}/tracks", get(album_tracks_partial))
        .route("/album/{id}/set-favorite", put(set_favorite))
        .route("/album/{id}/unset-favorite", put(unset_favorite))
        .route("/album/{id}/play", put(play))
        .route("/album/{id}/play/{track_position}", put(play_track))
}

async fn play_track(Path((id, track_position)): Path<(String, u32)>) -> impl IntoResponse {
    qobuz_player_controls::play_album(&id, track_position)
        .await
        .unwrap();
}

async fn set_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::add_favorite_album(&id)
        .await
        .unwrap();
    render(html! { <ToggleFavorite id=id is_favorite=true /> })
}

async fn unset_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::remove_favorite_album(&id)
        .await
        .unwrap();
    render(html! { <ToggleFavorite id=id is_favorite=false /> })
}

async fn play(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::play_album(&id, 0).await.unwrap();
}

async fn index(Path(id): Path<String>) -> impl IntoResponse {
    let (album, suggested_albums, favorites) = join!(
        qobuz_player_controls::album(id.clone()),
        qobuz_player_controls::suggested_albums(id.clone()),
        qobuz_player_controls::favorites()
    );

    let album = album.unwrap();
    let suggested_albums = suggested_albums.unwrap();
    let favorites = favorites.unwrap();

    let is_favorite = favorites.albums.iter().any(|album| album.id == id);

    render(html! {
        <Page active_page=Page::None>
            <Album album=album suggested_albums=suggested_albums is_favorite=is_favorite />
        </Page>
    })
}

async fn album_tracks_partial(Path(id): Path<String>) -> impl IntoResponse {
    let album = qobuz_player_controls::album(id).await.unwrap();

    render(html! { <AlbumTracks tracks=album.tracks album_id=album.id /> })
}

#[component]
fn album_tracks(tracks: Vec<Track>, album_id: String) -> impl IntoView {
    let album_id_clone = album_id.clone();

    html! {
        <div
            class="w-full"
            hx-get=format!("/album/{}/tracks", album_id_clone)
            hx-trigger="sse:tracklist"
            hx-swap="outerHTML"
        >
            <ListTracks
                track_number_display=TrackNumberDisplay::Number
                tracks=tracks
                show_artist=false
                dim_played=false
                api_call=move |index: usize| format!("/album/{}/play/{}", album_id, index)
            />
        </div>
    }
}

#[component]
fn album(album: Album, suggested_albums: Vec<AlbumSimple>, is_favorite: bool) -> impl IntoView {
    let duration = parse_duration(album.duration_seconds);

    html! {
        <div class="flex flex-col justify-center items-center sm:p-4">
            <div class="flex flex-wrap gap-4 justify-center items-end p-4 w-full *:max-w-sm">
                <img
                    src=album.image
                    alt=album.title.clone()
                    class="object-contain rounded-lg size-full aspect-square"
                />

                <div class="flex flex-col flex-grow gap-4 items-center w-full">
                    <div class="flex flex-col gap-2 justify-center items-center w-full text-center">
                        <a
                            href=format!("/artist/{}", album.artist.id)
                            class="text-gray-400 rounded sm:text-lg"
                        >
                            {album.artist.name}
                        </a>
                        <span class="text-lg sm:text-xl">{album.title}</span>
                        <span class="flex gap-2 text-gray-400 sm:text-lg">
                            <span>{album.release_year}</span>
                            <span>"•︎"</span>
                            <span>{format!("{} minutes", duration.minutes)}</span>
                        </span>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <button
                            class="flex gap-2 justify-center items-center py-2 px-4 bg-blue-500 rounded cursor-pointer"
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
                </div>
            </div>
            <div class="flex flex-col gap-4 w-full">
                <AlbumTracks tracks=album.tracks album_id=album.id.clone() />

                {if !suggested_albums.is_empty() {
                    Some(
                        html! {
                            <div class="flex flex-col gap-2 w-full">
                                <h3 class="px-4 text-lg">Album suggestions</h3>
                                <ListAlbumsVertical albums=suggested_albums />
                            </div>
                        },
                    )
                } else {
                    None
                }}
            </div>
        </div>
    }
}
