use axum::{extract::Path, response::IntoResponse, routing::get, Form, Router};
use leptos::{component, prelude::*, IntoView};
use qobuz_player_controls::service::SearchResults;
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
    Router::new().route("/search/{tab}", get(index).post(search))
}

#[derive(Deserialize, Clone)]
struct SearchParameters {
    query: String,
}

async fn index(Path(tab): Path<Tab>) -> impl IntoResponse {
    let search_results = SearchResults {
        query: "".into(),
        albums: vec![],
        tracks: vec![],
        artists: vec![],
        playlists: vec![],
    };

    let html = html! {
        <Page active_page=Page::Search>
            <Search search_results=search_results tab=tab />
        </Page>
    };

    render(html)
}

async fn search(
    Path(tab): Path<Tab>,
    Form(parameters): Form<SearchParameters>,
) -> impl IntoResponse {
    let search_results = qobuz_player_controls::search(&parameters.query).await;

    let html = html! {
        <SearchPartial search_results=search_results tab=tab.clone() />

        <div hx-swap-oob="true">
            <TabBar tab=tab />
        </div>
    };

    render(html)
}

#[component]
fn search_partial(search_results: SearchResults, tab: Tab) -> impl IntoView {
    match tab {
        Tab::Albums => html! {
            <ListAlbums
                albums=search_results.albums
                sort=crate::components::list::AlbumSort::Default
            />
        }
        .into_any(),
        Tab::Artists => html! {
            <ListArtists
                artists=search_results.artists
                sort=crate::components::list::ArtistSort::Default
            />
        }
        .into_any(),
        Tab::Playlists => html! {
            <ListPlaylists
                playlists=search_results.playlists
                sort=crate::components::list::PlaylistSort::Default
            />
        }
        .into_any(),
    }
}

#[component]
fn tab_bar(tab: Tab) -> impl IntoView {
    html! {
        <div
            id="tabs"
            class="flex justify-between *:rounded-full *:px-2 *:py-1 *:transition-colors"
        >
            {html! {
                <a
                    href="albums"
                    class=format!(
                        "hover:bg-blue-600 {}",
                        if tab == Tab::Albums { "bg-blue-800" } else { "" },
                    )
                >

                    Albums
                </a>
            }
                .attr("preload", "mouseover")
                .attr("preload-images", "true")}
            {html! {
                <a
                    href="artists"
                    class=format!(
                        "hover:bg-blue-600 {}",
                        if tab == Tab::Artists { "bg-blue-800" } else { "" },
                    )
                >
                    Artists
                </a>
            }
                .attr("preload", "mouseover")
                .attr("preload-images", "true")}
            {html! {
                <a
                    href="playlists"
                    class=format!(
                        "hover:bg-blue-600 {}",
                        if tab == Tab::Playlists { "bg-blue-800" } else { "" },
                    )
                >
                    Playlists
                </a>
            }
                .attr("preload", "mouseover")
                .attr("preload-images", "true")}

        </div>
    }
}

#[component]
fn search(search_results: SearchResults, tab: Tab) -> impl IntoView {
    html! {
        <div class="flex flex-col h-full">
            <div class="flex flex-col flex-grow gap-4 p-4 max-h-full">
                <div class="flex flex-row gap-4 items-center" id="search-form">
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
                        oninput="setSearchQuery(this.value)"
                        hx-post=""
                        hx-trigger="input changed delay:500ms, keyup[key=='Enter'], load"
                        hx-target="#search-results"
                        hx-swap="innerHTML"
                    />
                    <span class="size-8">
                        <MagnifyingGlass />
                    </span>
                    <script>loadSearchInput()</script>
                </div>

                <TabBar tab=tab.clone() />
            </div>

            <div id="search-results" class="overflow-auto h-full">
                <SearchPartial search_results=search_results tab=tab />
            </div>
        </div>
    }
}
