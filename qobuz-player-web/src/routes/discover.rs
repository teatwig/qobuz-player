use std::sync::Arc;

use axum::{Router, extract::State, response::IntoResponse, routing::get};
use leptos::prelude::*;
use qobuz_player_controls::models::{AlbumSimple, Playlist};
use tokio::try_join;

use crate::{
    AppState, Discover,
    components::list::{ListAlbumsVertical, ListPlaylistsVertical},
    html,
    page::Page,
    view::render,
};

pub(crate) fn routes() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new().route("/discover", get(index))
}

async fn get_discover(state: &AppState) -> Discover {
    if let Some(cached) = state.discover_cache.get().await {
        return cached;
    }

    let (albums, playlists) = try_join!(
        state.player_state.client.featured_albums(),
        state.player_state.client.featured_playlists(),
    )
    .unwrap();

    let discover = Discover { albums, playlists };

    state.discover_cache.set(discover.clone()).await;

    discover
}

async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let discover = get_discover(&state).await;

    let tracklist = state.player_state.tracklist.read().await;
    let current_status = state.player_state.target_status.read().await;

    let album_features = discover
        .albums
        .into_iter()
        .map(|x| html! { <AlbumFeature albums=x.1 name=x.0 /> })
        .collect::<Vec<_>>();

    let playlist_features = discover
        .playlists
        .into_iter()
        .map(|x| html! { <PlaylistFeature playlists=x.1 name=x.0 /> })
        .collect::<Vec<_>>();

    render(html! {
        <Page active_page=Page::Discover current_status=&current_status tracklist=&tracklist>
            <div class="flex flex-col gap-8 px-4">
                <div class="flex sticky top-0 flex-col flex-grow gap-4 pb-2 max-h-full pt-safe-or-4 bg-black/80 backdrop-blur">
                    <h1 class="text-2xl">Discover</h1>
                </div>
                {album_features}
                {playlist_features}
            </div>
        </Page>
    })
}

#[component]
fn album_feature(albums: Vec<AlbumSimple>, name: String) -> impl IntoView {
    html! {
        <div class="flex flex-col gap-2">
            <h3 class="text-lg">{name}</h3>
            <ListAlbumsVertical albums=albums />
        </div>
    }
}

#[component]
fn playlist_feature(playlists: Vec<Playlist>, name: String) -> impl IntoView {
    html! {
        <div class="flex flex-col gap-2">
            <h3 class="text-lg">{name}</h3>
            <ListPlaylistsVertical playlists=playlists />
        </div>
    }
}
