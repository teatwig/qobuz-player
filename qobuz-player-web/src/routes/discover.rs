use std::sync::Arc;

use axum::{Router, extract::State, response::IntoResponse, routing::get};
use leptos::prelude::*;
use qobuz_player_controls::models::{AlbumSimple, Playlist};
use tokio::try_join;

use crate::{
    AppState,
    components::list::{ListAlbumsVertical, ListPlaylistsVertical},
    html,
    page::Page,
    view::render,
};

pub(crate) fn routes() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new().route("/discover", get(index))
}

async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (press_awards, new_releases, qobuzissims, ideal_discography, editor_picks) = try_join!(
        qobuz_player_controls::featured_albums(
            qobuz_player_controls::AlbumFeaturedType::PressAwards
        ),
        qobuz_player_controls::featured_albums(
            qobuz_player_controls::AlbumFeaturedType::NewReleasesFull
        ),
        qobuz_player_controls::featured_albums(
            qobuz_player_controls::AlbumFeaturedType::Qobuzissims
        ),
        qobuz_player_controls::featured_albums(
            qobuz_player_controls::AlbumFeaturedType::IdealDiscography
        ),
        qobuz_player_controls::featured_playlists(
            qobuz_player_controls::PlaylistFeaturedType::EditorPicks
        ),
    )
    .unwrap();

    let tracklist = state.player_state.tracklist.read().await;
    let current_status = qobuz_player_controls::current_state().await;

    render(html! {
        <Page active_page=Page::Discover current_status=current_status tracklist=&tracklist>
            <div class="flex flex-col gap-8 px-4">
                <div class="flex sticky top-0 flex-col flex-grow gap-4 pb-2 max-h-full pt-safe-or-4 bg-black/80 backdrop-blur">
                    <h1 class="text-2xl">Discover</h1>
                </div>
                <AlbumFeature albums=press_awards name="Press awards".to_string() />
                <AlbumFeature albums=new_releases name="New releases".to_string() />
                <AlbumFeature albums=qobuzissims name="Qobuzissims".to_string() />
                <AlbumFeature albums=ideal_discography name="Ideal discography".to_string() />
                <PlaylistFeature playlists=editor_picks name="Featured playlists".to_string() />
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
