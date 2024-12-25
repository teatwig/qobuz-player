use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use hifirs_player::service::{Album, Artist};
use leptos::prelude::*;
use std::sync::Arc;
use tokio::join;

use crate::{
    components::{list::ListAlbums, ToggleFavorite},
    html,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:id", get(index))
        .route("/:id/set-favorite", put(set_favorite))
        .route("/:id/unset-favorite", put(unset_favorite))
}

async fn set_favorite(Path(id): Path<String>) -> impl IntoResponse {
    hifirs_player::add_favorite_artist(&id).await;
}

async fn unset_favorite(Path(id): Path<String>) -> impl IntoResponse {
    hifirs_player::remove_favorite_artist(&id).await;
}

async fn index(Path(id): Path<i32>) -> impl IntoResponse {
    let (artist, albums, favorites) = join!(
        hifirs_player::artist(id),
        hifirs_player::artist_albums(id),
        hifirs_player::favorites()
    );

    let is_favorite = favorites
        .artists
        .iter()
        .any(|artist| artist.id == id as u32);

    render(html! {
        <Page active_page=Page::Search>
            <Artist artist=artist albums=albums is_favorite=is_favorite />
        </Page>
    })
}

#[component]
fn artist(artist: Artist, albums: Vec<Album>, is_favorite: bool) -> impl IntoView {
    html! {
        <div class="flex flex-col flex-grow max-h-full">
            <div class="flex gap-4 justify-between items-center p-4">
                <h1 class="text-2xl">{artist.name}</h1>

                <ToggleFavorite id=artist.id.to_string() is_favorite=is_favorite />
            </div>
            <ListAlbums albums=albums sort=crate::components::list::AlbumSort::ReleaseYear />
        </div>
    }
}
