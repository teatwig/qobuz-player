use axum::{
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use hifirs_player::queue::{TrackListType, TrackListValue};
use leptos::{component, prelude::*, IntoView};
use std::sync::Arc;

use crate::{
    components::Info,
    html,
    icons::{Backward, Forward, Pause, Play},
    is_htmx_request,
    page::Page,
    view::render,
    AppState,
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index))
        .route("/progress", get(progress_partial))
        .route("/status", get(status_partial))
        .route("/api/play", put(play))
        .route("/api/pause", put(pause))
        .route("/api/previous", put(previous))
        .route("/api/next", put(next))
}

async fn status_partial() -> impl IntoResponse {
    let status = hifirs_player::current_state();

    if status == gstreamer::State::Playing {
        render(html! { <PlayPause play=true /> })
    } else {
        render(html! { <PlayPause play=false /> })
    }
}

#[component]
fn play_pause(play: bool) -> impl IntoView {
    html! {
        <button
            id="play-pause"
            hx-swap="outerHTML"
            hx-target="this"
            hx-trigger="click, sse:status"
            hx-put=format!("api/{}", if play { "pause" } else { "play" })
        >
            {match play {
                true => html! { <Pause /> }.into_any(),
                false => html! { <Play /> }.into_any(),
            }}
        </button>
    }
}

async fn play() -> impl IntoResponse {
    match hifirs_player::play().await {
        Ok(_) => render(html! { <PlayPause play=true /> }),
        Err(_) => render(html! { <PlayPause play=false /> }),
    }
}

async fn pause() -> impl IntoResponse {
    match hifirs_player::pause().await {
        Ok(_) => render(html! { <PlayPause play=false /> }),
        Err(_) => render(html! { <PlayPause play=true /> }),
    }
}

async fn previous() -> impl IntoResponse {
    _ = hifirs_player::previous().await;
}

async fn next() -> impl IntoResponse {
    _ = hifirs_player::next().await;
}

async fn index(headers: HeaderMap) -> impl IntoResponse {
    let current_tracklist = hifirs_player::current_tracklist().await;
    let position_mseconds = hifirs_player::position().map(|position| position.mseconds());
    let current_status = hifirs_player::current_state();

    let inner = html! {
        <NowPlaying
            current_tracklist=current_tracklist
            position_mseconds=position_mseconds
            current_status=current_status
        />
    };

    let hx_request = is_htmx_request(&headers);
    let html = match hx_request {
        true => inner.into_any(),
        false => html! { <Page active_page=Page::NowPlaying>{inner}</Page> }.into_any(),
    };

    render(html)
}

async fn progress_partial() -> impl IntoResponse {
    let position_mseconds = hifirs_player::position().map(|position| position.mseconds());
    let current_track = hifirs_player::current_track().await;
    let duration_seconds = current_track.map(|track| track.duration_seconds);

    render(
        html! { <Progress position_seconds=position_mseconds duration_seconds=duration_seconds /> },
    )
}

#[component]
fn progress(position_seconds: Option<u64>, duration_seconds: Option<u32>) -> impl IntoView {
    let position = position_seconds.map_or("00:00".to_string(), mseconds_to_mm_ss);
    let duration = duration_seconds.map_or("00:00".to_string(), seconds_to_mm_ss);

    let progress = position_seconds
        .and_then(|position| duration_seconds.map(|duration| position as u32 * 100 / duration))
        .unwrap_or(0);

    html! {
        <div hx-get="progress" hx-trigger="sse:position" hx-swap="outerHTML">
            <div class="grid h-2 rounded-full overflow-clip">
                <div style="grid-column: 1; grid-row: 1;" class="w-full bg-gray-800"></div>
                <div
                    id="progress-bar"
                    class="bg-gray-500 transition-all"
                    style=format!("grid-column: 1; grid-row: 1; width: calc({progress}%/1000)")
                ></div>
            </div>
            <div class="flex justify-between text-sm text-gray-500">
                <span>{position}</span>
                <span>{duration}</span>
            </div>
        </div>
    }
}

#[component]
pub fn now_playing(
    current_tracklist: TrackListValue,
    position_mseconds: Option<u64>,
    current_status: gstreamer::State,
) -> impl IntoView {
    let current_track = current_tracklist
        .queue
        .values()
        .find(|track| track.status == hifirs_player::service::TrackStatus::Playing);

    let album = current_tracklist.get_album();
    let cover_image = album.map(|album| album.cover_art.clone());

    let entity_title = match current_tracklist.list_type() {
        TrackListType::Album => album.map(|album| album.title.clone()),
        TrackListType::Playlist => current_tracklist.playlist.map(|playlist| playlist.title),
        TrackListType::Track => album.map(|album| album.title.clone()),
        TrackListType::Unknown => None,
    };

    let (title, current_track_number, artist_name, duration_seconds, explicit, hires_available) =
        current_track.map_or((String::default(), 0, None, None, false, false), |track| {
            (
                track.title.clone(),
                track.number,
                track.artist.as_ref().map(|artist| artist.name.clone()),
                Some(track.duration_seconds),
                track.explicit,
                track.hires_available,
            )
        });

    let number_of_tracks = current_tracklist.queue.len();

    html! {
        <div
            hx-get=""
            hx-trigger="sse:tracklist"
            hx-swap="outerHTML"
            class="flex flex-col gap-8 justify-center items-center p-8 h-full landscape:flex-row"
        >
            <div class="max-h-full rounded-lg shadow-lg aspect-square max-w-[600px] overflow-clip">
                {if let Some(cover_image_url) = cover_image {
                    html! { <img src=cover_image_url alt=title.clone() class="object-contain" /> }
                        .into_any()
                } else {
                    html! { <div class="bg-gray-900 h-[300px] w-[300px]"></div> }.into_any()
                }}
            </div>

            <div class="flex flex-col flex-grow justify-center w-full max-w-md md:max-w-[600px]">
                <div class="flex gap-2 justify-between items-center">
                    <span class="text-xl truncate">{entity_title}</span>
                    <div class="text-gray-500 whitespace-nowrap">
                        {if current_track.is_some() {
                            format!("{} of {}", current_track_number, number_of_tracks)
                        } else {
                            String::default()
                        }}
                    </div>
                </div>

                <div class="text-gray-400 truncate">{artist_name}</div>

                <div class="flex flex-col gap-y-4 w-full">
                    <div class="flex gap-2 justify-between items-center">
                        <span class="text-2xl truncate">{title}</span>
                        <Info explicit=explicit hires_available=hires_available />
                    </div>

                    <Progress
                        position_seconds=position_mseconds
                        duration_seconds=duration_seconds
                    />
                </div>

                <div class="flex flex-row gap-2 justify-center h-10">
                    <button hx-swap="none" hx-put="api/previous">
                        <Backward />
                    </button>

                    {if current_status != gstreamer::State::Playing {
                        html! { <PlayPause play=false /> }.into_any()
                    } else {
                        html! { <PlayPause play=true /> }.into_any()
                    }}
                    <button hx-put="api/next" hx-swap="none">
                        <Forward />
                    </button>
                </div>
            </div>
        </div>
    }
}

fn seconds_to_mm_ss<T: Into<u64>>(seconds: T) -> String {
    let seconds = seconds.into();
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn mseconds_to_mm_ss<T: Into<u64>>(seconds: T) -> String {
    let seconds = seconds.into() / 1000;
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
