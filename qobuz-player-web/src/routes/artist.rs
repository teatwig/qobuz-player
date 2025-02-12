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
    icons::Play,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/artist/{id}", get(index))
        .route("/artist/{id}/top-tracks", get(top_tracks_partial))
        .route("/artist/{id}/set-favorite", put(set_favorite))
        .route("/artist/{id}/unset-favorite", put(unset_favorite))
        .route(
            "/artist/{artist_id}/play-top-track/{track_index}",
            put(play_top_track),
        )
}

async fn top_tracks_partial(Path(id): Path<u32>) -> impl IntoResponse {
    let (artist, tracklist) = join!(
        qobuz_player_controls::artist_page(id),
        qobuz_player_controls::current_tracklist(),
    );

    let now_playing_id = tracklist.currently_playing();
    let artist = artist.unwrap();

    render(
        html! { <ListTracks artist_id=artist.id tracks=artist.top_tracks now_playing_id=now_playing_id /> },
    )
}

async fn play_top_track(Path((artist_id, track_index)): Path<(u32, u32)>) -> impl IntoResponse {
    qobuz_player_controls::play_top_tracks(artist_id, track_index)
        .await
        .unwrap();
}

async fn set_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::add_favorite_artist(&id)
        .await
        .unwrap();
}

async fn unset_favorite(Path(id): Path<String>) -> impl IntoResponse {
    qobuz_player_controls::remove_favorite_artist(&id)
        .await
        .unwrap();
}

async fn index(Path(id): Path<u32>) -> impl IntoResponse {
    let (artist, albums, similar_artists, favorites, tracklist) = join!(
        qobuz_player_controls::artist_page(id),
        qobuz_player_controls::artist_albums(id),
        qobuz_player_controls::similar_artists(id),
        qobuz_player_controls::favorites(),
        qobuz_player_controls::current_tracklist(),
    );

    let now_playing_id = tracklist.currently_playing();

    let artist = artist.unwrap();
    let similar_artists = similar_artists.unwrap();
    let albums = albums.unwrap();
    let favorites = favorites.unwrap();

    let is_favorite = favorites.artists.iter().any(|artist| artist.id == id);

    render(html! {
        <Page active_page=Page::None>
            <Artist
                artist=artist
                albums=albums
                is_favorite=is_favorite
                similar_artists=similar_artists
                now_playing_id=now_playing_id
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
    now_playing_id: Option<u32>,
) -> impl IntoView {
    let artist_image_style = artist
        .image
        .map(|image| format!("background-image: url({});", image));

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
                <div
                    hx-get=format!("{}/top-tracks", artist.id)
                    hx-trigger="sse:tracklist"
                    hx-target="#top-tracks"
                    class="flex flex-col gap-2"
                >
                    <h3 class="px-4 text-lg">Top tracks</h3>
                    <div
                        id="top-tracks"
                        class="flex overflow-x-auto flex-col flex-wrap gap-4 p-4 max-h-92"
                    >
                        <ListTracks
                            artist_id=artist.id
                            tracks=artist.top_tracks
                            now_playing_id=now_playing_id
                        />
                    </div>
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
fn list_tracks(
    artist_id: u32,
    tracks: Vec<models::Track>,
    now_playing_id: Option<u32>,
) -> impl IntoView {
    tracks
        .into_iter()
        .enumerate()
        .map(|(index, track)| {
            let is_playing = now_playing_id.is_some_and(|id| id == track.id);
            html! { <Track artist_id=artist_id track=track index=index is_playing=is_playing /> }
        })
        .collect::<Vec<_>>()
}

#[component]
fn track(artist_id: u32, track: models::Track, index: usize, is_playing: bool) -> impl IntoView {
    html! {
        <button
            class="flex gap-4 items-center w-5/6 max-w-lg rounded cursor-pointer hover:bg-blue-800"
            hx-put=format!("{}/play-top-track/{}", artist_id, index)
            hx-swap="none"
        >
            <img
                class="inline text-sm text-gray-500 bg-gray-800 rounded-md aspect-square size-12"
                alt=track.title.clone()
                src=track.album.as_ref().map(|a| a.image.clone())
            />

            <div class="overflow-hidden w-full">
                <div class="flex gap-2 items-center">
                    <h3 class="text-lg truncate">{track.title}</h3>
                    <Info explicit=track.explicit hires_available=track.hires_available />
                    {is_playing
                        .then_some({
                            html! {
                                <div class="text-blue-500 size-6">
                                    <Play />
                                </div>
                            }
                        })}
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
