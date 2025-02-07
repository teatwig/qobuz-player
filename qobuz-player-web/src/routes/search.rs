use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    routing::{get, put},
    Form, Router,
};
use leptos::{component, prelude::*};
use qobuz_player_controls::models::{SearchResults, Track as TrackModel};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Tab {
    Albums,
    Artists,
    Playlists,
    Tracks,
}

use crate::{
    components::{
        list::{List, ListAlbums, ListArtists, ListItem, ListPlaylists},
        Info,
    },
    html,
    icons::MagnifyingGlass,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search/{tab}", get(index).post(search))
        .route("/play/{track_id}", put(play_track))
}

async fn play_track(Path(track_id): Path<i32>) -> impl IntoResponse {
    qobuz_player_controls::play_track(track_id).await.unwrap();
}

#[derive(Deserialize, Clone)]
struct SearchParameters {
    query: Option<String>,
}

async fn index(
    Path(tab): Path<Tab>,
    Query(parameters): Query<SearchParameters>,
) -> impl IntoResponse {
    let query = parameters.query;
    let search_results = match &query {
        Some(query) => qobuz_player_controls::search(query).await,
        None => SearchResults::default(),
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
    let query = parameters.query;
    let search_results = match &query {
        Some(query) => qobuz_player_controls::search(query).await,
        None => SearchResults::default(),
    };

    let html = html! {
        <SearchPartial search_results=search_results tab=tab.clone() />

        {html! { <TabBar query=query.unwrap_or_default() tab=tab /> }.attr("hx-swap-oob", "true")}
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
        Tab::Tracks => html! { <ListTracks tracks=search_results.tracks /> }.into_any(),
    }
}

#[component]
pub fn list_tracks(tracks: Vec<TrackModel>) -> impl IntoView {
    html! {
        <List>
            {tracks
                .into_iter()
                .map(|track| {
                    html! {
                        <ListItem>
                            <Track track=track />
                        </ListItem>
                    }
                })
                .collect::<Vec<_>>()}
        </List>
    }
}

#[component]
fn track(track: TrackModel) -> impl IntoView {
    html! {
        <button
            class="flex gap-4 items-center w-full cursor-pointer"
            hx-put=format!("/play/{}", track.id)
            hx-swap="none"
        >
            <img
                class="inline text-sm text-gray-500 bg-gray-800 rounded-md aspect-square size-12"
                alt=track.title.clone()
                src=track.cover_art
            />

            <div class="overflow-hidden w-full">
                <div class="flex justify-between">
                    <h3 class="text-lg truncate">{track.title}</h3>
                    <Info explicit=track.explicit hires_available=track.hires_available />
                </div>

                <h4 class="flex gap-2 text-left text-gray-400">
                    {track
                        .artist
                        .map(|artist| {
                            html! { <span class="truncate">{artist.name}</span> }
                        })}
                    {track
                        .album
                        .map(|album| {
                            html! {
                                <span>"•︎"</span>
                                <span>{album.release_year}</span>
                            }
                                .into_any()
                        })}
                </h4>
            </div>
        </button>
    }
    .attr("preload", "mousedown")
    .attr("preload-images", "true")
}

#[component]
fn tab_bar(query: String, tab: Tab) -> impl IntoView {
    html! {
        <div
            id="tabs"
            class="flex justify-between *:rounded-full *:px-2 *:py-1 *:transition-colors"
        >
            {html! {
                <a
                    href=format!("albums?query={}", query)
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
                    href=format!("artists?query={}", query)
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
                    href=format!("playlists?query={}", query)
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
            {html! {
                <a
                    href=format!("tracks?query={}", query)
                    class=format!(
                        "hover:bg-blue-600 {}",
                        if tab == Tab::Tracks { "bg-blue-800" } else { "" },
                    )
                >
                    Tracks
                </a>
            }
                .attr("preload", "mouseover")
                .attr("preload-images", "true")}

        </div>
    }
}

#[component]
fn search(search_results: SearchResults, tab: Tab) -> impl IntoView {
    let query = search_results.query.clone();
    html! {
        <div class="flex flex-col h-full">
            <div class="flex flex-col flex-grow gap-4 p-4 max-h-full">
                <div class="flex flex-row gap-4 items-center" id="search-form">
                    <input
                        id="query"
                        name="query"
                        class="p-2 w-full text-black bg-white rounded"
                        autocapitalize="off"
                        autocomplete="off"
                        autocorrect="off"
                        placeholder="Search"
                        spellcheck="false"
                        type="search"
                        oninput="setSearchQuery(this.value)"
                        hx-post=""
                        hx-trigger="input changed delay:500ms, keyup[key=='Enter']"
                        hx-target="#search-results"
                        hx-swap="innerHTML"
                    />
                    <span class="size-8">
                        <MagnifyingGlass />
                    </span>
                    <script>loadSearchInput()</script>
                </div>

                <TabBar query=query tab=tab.clone() />
            </div>

            <div id="search-results" class="overflow-auto h-full">
                <SearchPartial search_results=search_results tab=tab />
            </div>
        </div>
    }
}
