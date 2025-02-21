use axum::{response::IntoResponse, routing::get, Router};
use leptos::{component, prelude::*, IntoView};
use qobuz_player_controls::tracklist::TracklistType;

use crate::{
    html,
    routes::now_playing::{Next, Previous, State},
    view::render,
};

pub fn routes() -> Router {
    Router::new().route("/controls", get(controls))
}

#[component]
pub fn controls() -> impl IntoView {
    html! {
        <div
            hx-get="/controls"
            hx-trigger="sse:tracklist"
            hx-target="this"
            hx-preserve
            id="controls"
        >
            <ControlsPartial />
        </div>
    }
}

async fn controls() -> impl IntoResponse {
    render(html! { <ControlsPartial /> })
}

#[component]
fn controls_partial() -> impl IntoView {
    let current_status = futures::executor::block_on(qobuz_player_controls::current_state());
    let current_tracklist = futures::executor::block_on(qobuz_player_controls::current_tracklist());

    let (playing, show) = match current_status {
        qobuz_player_controls::State::VoidPending => (false, false),
        qobuz_player_controls::State::Null => (false, false),
        qobuz_player_controls::State::Ready => (false, true),
        qobuz_player_controls::State::Paused => (false, true),
        qobuz_player_controls::State::Playing => (true, true),
    };

    let (image, title, entity_link) = match current_tracklist.list_type {
        TracklistType::Album(tracklist) => (
            image(tracklist.image, false).into_any(),
            Some(tracklist.title),
            Some(format!("/album/{}", tracklist.id)),
        ),
        TracklistType::Playlist(tracklist) => (
            image(tracklist.image, false).into_any(),
            Some(tracklist.title),
            Some(format!("/playlist/{}", tracklist.id)),
        ),
        TracklistType::TopTracks(tracklist) => (
            image(tracklist.image, true).into_any(),
            Some(tracklist.artist_name),
            Some(format!("/artist/{}", tracklist.id)),
        ),
        TracklistType::Track(tracklist) => (
            image(tracklist.image, false).into_any(),
            Some(tracklist.track_title),
            tracklist.album_id.map(|id| format!("/album/{}", id)),
        ),
        TracklistType::None => (image(None, false).into_any(), None, None),
    };

    html! {
        {show
            .then(|| {
                html! {
                    <div class="h-16"></div>
                    <div class="fixed right-0 left-0 bottom-14 px-safe-offset-2 py-safe">
                        <div class="flex gap-2 justify-between items-center p-2 rounded-md bg-gray-900/70 backdrop-blur">
                            <a
                                class="flex overflow-hidden gap-2 items-center w-full"
                                hx-target="unset"
                                href=entity_link
                            >
                                {image}
                                <span class="truncate">{title}</span>
                            </a>
                            <div class="flex gap-4 items-center">
                                <span class="hidden w-8 sm:flex">
                                    <Previous />
                                </span>
                                <span class="flex w-8">
                                    <State playing=playing />
                                </span>
                                <span class="flex w-8">
                                    <Next />
                                </span>
                            </div>
                        </div>
                    </div>
                }
            })}
    }
}

fn image(url: Option<String>, cicle: bool) -> impl IntoView {
    let image_style = url.map(|url| format!("background-image: url({});", url));

    html! {
        <div
            class=format!(
                "bg-gray-800 bg-center bg-no-repeat bg-cover shadow aspect-square size-10 {}",
                if cicle { "rounded-full" } else { "rounded-md" },
            )
            style=image_style
        ></div>
    }
}
