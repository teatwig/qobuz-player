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
        <div class="flex flex-col flex-grow max-h-full" x-data="{ tab: 'albums' }">
            <div class="flex flex-col gap-4 p-4">
                <h1 class="text-2xl">Favorites</h1>

                <div class="flex justify-between *:rounded-full *:px-2 *:py-1 *:transition-colors">
                    <button
                        x-bind:class="{'bg-blue-800' : tab === 'albums'}"
                        x-on:click.prevent="tab = 'albums'"
                    >
                        Albums
                    </button>
                    <button
                        x-bind:class="{'bg-blue-800' : tab === 'artists'}"
                        x-on:click.prevent="tab = 'artists'"
                    >
                        Artists
                    </button>
                    <button
                        x-bind:class="{'bg-blue-800' : tab === 'playlists'}"
                        x-on:click.prevent="tab = 'playlists'"
                    >
                        Playlists
                    </button>
                </div>
            </div>

            <div class="contents" x-show="tab === 'albums'">
                <ListAlbums albums=favorites.albums />
            </div>
            <div class="contents" x-show="tab === 'artists'">
                <ListArtists artists=favorites.artists />
            </div>
            <div class="contents" x-show="tab === 'playlists'">
                <ListPlaylists playlists=playlists />
            </div>
        </div>
    }
}
