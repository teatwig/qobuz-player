use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use leptos::prelude::*;
use qobuz_player_controls::models::{self, AlbumPage, Artist, ArtistPage};
use std::sync::Arc;
use tokio::join;

use crate::{
    components::{
        list::{ListAlbumsVertical, ListArtistsVertical},
        Info, ToggleFavorite,
    },
    html,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/artist/{id}", get(index))
        .route("/artist/{id}/set-favorite", put(set_favorite))
        .route("/artist/{id}/unset-favorite", put(unset_favorite))
        .route(
            "/artist/{id}/play-top-track/{track_index}",
            put(play_top_track),
        )
}

async fn play_top_track(Path((id, track_id)): Path<(String, i32)>) -> impl IntoResponse {
    qobuz_player_controls::add_favorite_artist(&id).await;
}

async fn set_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::add_favorite_artist(&id).await;
}

async fn unset_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::remove_favorite_artist(&id).await;
}

async fn index(Path(id): Path<i32>) -> impl IntoResponse {
    let (artist, albums, similar_artists, favorites) = join!(
        qobuz_player_controls::artist_page(id),
        qobuz_player_controls::artist_albums(id),
        qobuz_player_controls::similar_artists(id),
        qobuz_player_controls::favorites()
    );

    let is_favorite = favorites
        .artists
        .iter()
        .any(|artist| artist.id == id as u32);

    render(html! {
        <Page active_page=Page::None>
            <Artist
                artist=artist
                albums=albums
                is_favorite=is_favorite
                similar_artists=similar_artists
            />
        </Page>
    })
}

#[component]
fn artist(
    artist: ArtistPage,
    albums: Vec<AlbumPage>,
    similar_artists: Vec<Artist>,
    is_favorite: bool,
) -> impl IntoView {
    let artist_image_style = format!("background-image: url({});", artist.image);
    html! {
        <div class="flex flex-col h-full">
            <div class="flex flex-col gap-4 items-center p-4">
                <div
                    class="bg-gray-800 bg-center bg-no-repeat bg-cover rounded-full aspect-square size-32"
                    style=artist_image_style
                ></div>
                <h1 class="text-2xl">{artist.name}</h1>
                <ToggleFavorite id=artist.id.to_string() is_favorite=is_favorite />
            </div>
            <div class="flex flex-col gap-4">
                <div class="flex flex-col gap-2">
                    <h3 class="px-4 text-lg">Top tracks</h3>
                    <ListTracks tracks=artist.top_tracks _now_playing_id=None />
                </div>
                <div class="flex flex-col gap-2">
                    <h3 class="px-4 text-lg">Albums</h3>
                    <ListAlbumsVertical
                        albums=albums
                        sort=crate::components::list::AlbumSort::ReleaseYear
                    />
                </div>
                {if !similar_artists.is_empty() {
                    Some(
                        html! {
                            <div class="flex flex-col gap-2">
                                <h3 class="px-4 text-lg">Similar artists</h3>
                                <ListArtistsVertical artists=similar_artists />
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

#[component]
fn list_tracks(tracks: Vec<models::Track>, _now_playing_id: Option<u32>) -> impl IntoView {
    html! {
        <div class="flex overflow-x-auto flex-col flex-wrap gap-4 p-4 max-h-92">
            {tracks
                .into_iter()
                .enumerate()
                .map(|(index, track)| {
                    html! { <Track track=track index=index /> }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn track(track: models::Track, index: usize) -> impl IntoView {
    html! {
        <button
            class="flex gap-4 items-center cursor-pointer w-lg"
            hx-put=format!("play-track/{}", index)
            hx-swap="none"
        >
            <img
                class="inline text-sm text-gray-500 bg-gray-800 rounded-md aspect-square size-12"
                alt=track.title.clone()
                src=track.album.as_ref().map(|a| a.image.clone())
            />

            <div class="overflow-hidden w-full">
                <div class="flex gap-2">
                    <h3 class="text-lg truncate">{track.title}</h3>
                    <Info explicit=track.explicit hires_available=track.hires_available />
                </div>

                <h4 class="flex gap-2 text-left text-gray-400">
                    {track
                        .album
                        .as_ref()
                        .map(|album| {
                            html! { <span class="truncate">{album.title.clone()}</span> }
                        })}
                </h4>
            </div>
        </button>
    }
}
