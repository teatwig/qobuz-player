use axum::{response::IntoResponse, routing::get, Router};
use leptos::prelude::*;
use qobuz_player_controls::models::{Album, Playlist};
use tokio::try_join;

use crate::{
    components::list::{ListAlbumsVertical, ListPlaylistsVertical},
    html,
    page::Page,
    view::render,
};

pub fn routes() -> Router {
    Router::new().route("/discover", get(index))
}

async fn index() -> impl IntoResponse {
    let (press_awards, new_releases, qobuzissims, editor_picks) = try_join!(
        qobuz_player_controls::featured_albums(
            qobuz_player_controls::AlbumFeaturedType::PressAwards
        ),
        qobuz_player_controls::featured_albums(
            qobuz_player_controls::AlbumFeaturedType::NewReleasesFull
        ),
        qobuz_player_controls::featured_albums(
            qobuz_player_controls::AlbumFeaturedType::Qobuzissims
        ),
        qobuz_player_controls::featured_playlists(
            qobuz_player_controls::PlaylistFeaturedType::EditorPicks
        ),
    )
    .unwrap();

    render(html! {
        <Page active_page=Page::Discover>
            <div class="flex flex-col gap-8 p-4">
                <h1 class="text-2xl">Discover</h1>
                <AlbumFeature albums=press_awards name="Press awards".to_string() />
                <AlbumFeature albums=new_releases name="New releases".to_string() />
                <AlbumFeature albums=qobuzissims name="Qobuzissims".to_string() />
                <PlaylistFeature playlists=editor_picks name="Featured playlists".to_string() />
            </div>
        </Page>
    })
}

#[component]
fn album_feature(albums: Vec<Album>, name: String) -> impl IntoView {
    html! {
        <div class="flex flex-col gap-2">
            <h3 class="text-lg">{name}</h3>
            <ListAlbumsVertical albums=albums sort=crate::components::list::AlbumSort::Default />
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
