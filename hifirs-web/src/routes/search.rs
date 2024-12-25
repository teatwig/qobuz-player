use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use hifirs_player::service::SearchResults;
use leptos::{component, prelude::*, IntoView};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    components::{
        list::{ListAlbums, ListArtists, ListPlaylists},
        Tab,
    },
    html,
    icons::MagnifyingGlass,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(index))
}

#[derive(Deserialize, Clone)]
struct SearchQuery {
    query: Option<String>,
    tab: Option<Tab>,
}

async fn index(Query(query): Query<SearchQuery>) -> impl IntoResponse {
    let tab = query.tab.unwrap_or(Tab::Albums);
    let query = query.query;

    let search_results = match &query {
        Some(query) => hifirs_player::search(query).await,
        None => SearchResults {
            query: query.unwrap_or("".into()),
            albums: vec![],
            tracks: vec![],
            artists: vec![],
            playlists: vec![],
        },
    };

    let html = html! {
        <Page active_page=Page::Search>
            <Search search_results=search_results tab=tab />
        </Page>
    };

    render(html)
}

#[component]
fn search(search_results: SearchResults, tab: Tab) -> impl IntoView {
    html! {
        <form id="search-form" class="flex flex-col flex-grow gap-4 p-4 max-h-full peer" action="#">
            <div class="flex flex-row gap-4 items-center">
                <input
                    id="query"
                    name="query"
                    class="p-2 w-full text-black rounded"
                    autocapitalize="off"
                    autocomplete="off"
                    autocorrect="off"
                    placeholder="Search"
                    spellcheck="false"
                    type="search"
                    autofocus
                />
                <span class="size-8">
                    <MagnifyingGlass />
                </span>
            </div>

            <input
                type="radio"
                id="albums"
                value="albums"
                class="sr-only peer/albums"
                name="tab"
                checked=tab == Tab::Albums
            />
            <input
                type="radio"
                id="artists"
                value="artists"
                class="sr-only peer/artists"
                name="tab"
                checked=tab == Tab::Artists
            />
            <input
                type="radio"
                id="playlists"
                value="playlists"
                class="sr-only peer/playlists"
                name="tab"
                checked=tab == Tab::Playlists
            />

            <div class="flex justify-between group *:rounded-full *:px-2 *:py-1 *:transition-colors *:cursor-pointer">
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
        </form>

        <div class="hidden overflow-auto h-full peer-[:has(#albums:checked)]:block">
            <ListAlbums
                albums=search_results.albums
                sort=crate::components::list::AlbumSort::Default
            />
        </div>
        <div class="hidden overflow-auto h-full peer-[:has(#artists:checked)]:block">
            <ListArtists
                artists=search_results.artists
                sort=crate::components::list::ArtistSort::Default
            />
        </div>
        <div class="hidden overflow-auto h-full peer-[:has(#playlists:checked)]:block">
            <ListPlaylists
                playlists=search_results.playlists
                sort=crate::components::list::PlaylistSort::Default
            />
        </div>
    }
}
