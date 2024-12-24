use axum::{response::IntoResponse, routing::get, Router};
use hifirs_player::service::{Favorites, Playlist};
use leptos::{component, prelude::*, IntoView};
use std::sync::Arc;
use tokio::join;

use crate::{
    components::list::{ListAlbums, ListArtists, ListPlaylists},
    html,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(index))
}

async fn index() -> impl IntoResponse {
    let (favorites, favorite_playlists) =
        join!(hifirs_player::favorites(), hifirs_player::user_playlists());

    render(html! {
        <Page active_page=Page::Favorites>
            <Favorites favorites=favorites playlists=favorite_playlists />
        </Page>
    })
}

#[component]
fn favorites(favorites: Favorites, playlists: Vec<Playlist>) -> impl IntoView {
    html! {
        <div class="flex flex-col flex-grow gap-4 p-4 max-h-full peer">
            <input
                type="radio"
                id="albums"
                value="albums"
                class="sr-only peer/albums"
                name="tab"
                checked
            />
            <input
                type="radio"
                id="artists"
                value="artists"
                class="sr-only peer/artists"
                name="tab"
            />
            <input
                type="radio"
                id="playlists"
                value="playlists"
                class="sr-only peer/playlists"
                name="tab"
            />

            <h1 class="text-2xl">Favorites</h1>

            <div class="flex justify-between group *:rounded-full *:px-2 *:py-1 *:transition-colors">
                <label for="albums" class="hover:bg-blue-600 group-[#albums:checked~&]:bg-blue-800">
                    Albums
                </label>
                <label
                    for="artists"
                    class="hover:bg-blue-600 group-[#artists:checked~&]:bg-blue-800"
                >
                    Artists
                </label>
                <label
                    for="playlists"
                    class="hover:bg-blue-600 group-[#playlists:checked~&]:bg-blue-800"
                >
                    Playlists
                </label>
            </div>
        </div>

        <div class="hidden h-full peer-[:has(#albums:checked)]:block">
            <ListAlbums albums=favorites.albums />
        </div>
        <div class="hidden h-full peer-[:has(#artists:checked)]:block">
            <ListArtists artists=favorites.artists />
        </div>
        <div class="hidden h-full peer-[:has(#playlists:checked)]:block">
            <ListPlaylists playlists=playlists />
        </div>
    }
}
