use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use leptos::{component, prelude::*, IntoView};
use qobuz_player_controls::tracklist::{TrackListType, Tracklist};

use crate::{
    components::list::{List, ListTracks, TrackNumberDisplay},
    html,
    page::Page,
    view::render,
};

pub fn routes() -> Router {
    Router::new()
        .route("/queue", get(index))
        .route("/queue/list", get(queue_partial))
        .route("/queue/skip-to/{track_number}", put(skip_to))
}

async fn skip_to(Path(track_number): Path<u32>) -> impl IntoResponse {
    qobuz_player_controls::skip_to_position(track_number, true)
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    let current_tracklist = qobuz_player_controls::current_tracklist().await;

    render(html! {
        <Page active_page=Page::Queue current_tracklist=current_tracklist.list_type.clone()>
            <Queue current_tracklist=current_tracklist />
        </Page>
    })
}

#[component]
fn queue(current_tracklist: Tracklist) -> impl IntoView {
    let entity_title = match current_tracklist.list_type() {
        TrackListType::Album(album) => Some(album.title.clone()),
        TrackListType::Playlist(playlist) => Some(playlist.title.clone()),
        TrackListType::TopTracks(artist) => Some(artist.artist_name.clone()),
        TrackListType::Track(track) => Some(track.track_title.clone()),
        TrackListType::None => None,
    };

    html! {
        <div
            hx-get="/queue/list"
            hx-trigger="sse:tracklist"
            hx-target="#queue-list"
            class="flex flex-col flex-grow gap-4 max-h-full"
        >
            <div class="sticky top-0 p-4 text-center bg-black/20 backdrop-blur">
                <p class="text-lg">{entity_title}</p>
            </div>

            <div id="queue-list">
                <QueueList current_tracklist=current_tracklist />
            </div>
        </div>
    }
}

async fn queue_partial() -> impl IntoResponse {
    let current_tracklist = qobuz_player_controls::current_tracklist().await;

    render(html! { <QueueList current_tracklist=current_tracklist /> })
}

#[component]
fn queue_list(current_tracklist: Tracklist) -> impl IntoView {
    let now_playing_id = current_tracklist.currently_playing();
    let tracks = current_tracklist.queue;

    html! {
        <List>
            <ListTracks
                track_number_display=TrackNumberDisplay::Cover
                now_playing_id=now_playing_id
                tracks=tracks
                show_artist=true
                dim_played=true
                api_call=|index: usize| format!("/queue/skip-to/{}", index)
            />
        </List>
    }
}
