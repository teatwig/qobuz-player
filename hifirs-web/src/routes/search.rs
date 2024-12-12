use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use hifirs_player::service::SearchResults;
use leptos::{component, prelude::*, IntoView};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    components::list::{ListAlbums, ListArtists, ListPlaylists},
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
}

async fn index(query: Query<SearchQuery>) -> impl IntoResponse {
    let query = query.0.query;

    let search_results = match &query {
        Some(query) => Some(hifirs_player::search(query).await),
        None => None,
    };

    render(html! {
        <Page active_page=Page::Search>
            <Search search_results=search_results query=query />
        </Page>
    })
}

#[component]
fn search_results(search_results: Option<SearchResults>) -> impl IntoView {
    html! {
        <div id="search-results" class="contents *:h-full">
            {search_results
                .map(|search_results| {
                    html! {
                        <div x-show="tab === 'albums'">
                            <ListAlbums albums=search_results.albums />
                        </div>
                        <div x-show="tab === 'artists'">
                            <ListArtists artists=search_results.artists />
                        </div>
                        <div x-show="tab === 'playlists'">
                            <ListPlaylists playlists=search_results.playlists />
                        </div>
                    }
                })}
        </div>
    }
}

#[component]
fn search(search_results: Option<SearchResults>, query: Option<String>) -> impl IntoView {
    html! {
        <div class="contents" x-data="{ tab: 'albums' }">
            <div class="flex flex-col flex-grow gap-4 p-4 max-h-full">
                <form class="flex flex-row gap-4 items-center" action="#">
                    <input
                        name="query"
                        class="p-2 w-full text-black rounded"
                        autocapitalize="off"
                        autocomplete="off"
                        placeholder="Search"
                        spellcheck="false"
                        type="text"
                        value=query.unwrap_or("".into())
                    />
                    <span class="size-8">
                        <MagnifyingGlass />
                    </span>
                </form>

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
            <SearchResults search_results=search_results />
        </div>
    }
}
