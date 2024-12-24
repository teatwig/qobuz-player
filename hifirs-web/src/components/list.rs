use hifirs_player::service::{Album, Artist, Playlist, Track};
use leptos::{component, prelude::*, IntoView};

use crate::{components::Info, html, icons::Play};

#[component]
pub fn list(children: Children) -> impl IntoView {
    html! { <ul class="overflow-y-auto w-full h-full leading-tight align-start">{children()}</ul> }
}

#[component]
pub fn list_item(children: Children) -> impl IntoView {
    html! {
        <li class="w-full text-left border-b border-gray-700 *:p-4 hover:bg-blue-500/25">
            {children()}
        </li>
    }
}

#[component]
pub fn list_albums(albums: Vec<Album>) -> impl IntoView {
    html! {
        <List>
            {albums
                .into_iter()
                .map(|album| {
                    html! {
                        <ListItem>
                            <a
                                class="flex gap-4 items-center w-full"
                                href=format!("/album/{}", album.id)
                            >
                                <img
                                    class="text-sm text-gray-500 bg-gray-800 rounded-md aspect-square size-12"
                                    alt=album.title.clone()
                                    src=album.cover_art_small
                                />

                                <div class="overflow-hidden w-full">
                                    <div class="flex justify-between">
                                        <h3 class="text-lg truncate">{album.title}</h3>
                                        <Info
                                            explicit=album.explicit
                                            hires_available=album.hires_available
                                        />
                                    </div>

                                    <h4 class="flex gap-2 text-left text-gray-400">
                                        <span class="truncate">{album.artist.name}</span>
                                        <span>"•︎"</span>
                                        <span>{album.release_year}</span>
                                    </h4>
                                </div>
                            </a>
                        </ListItem>
                    }
                })
                .collect::<Vec<_>>()}
        </List>
    }
}

#[component]
pub fn list_artists(artists: Vec<Artist>) -> impl IntoView {
    html! {
        <List>
            {artists
                .into_iter()
                .map(|artist| {
                    html! {
                        <ListItem>
                            <a href=format!("/artist/{}", artist.id) class="block text-lg truncate">
                                {artist.name}
                            </a>
                        </ListItem>
                    }
                })
                .collect::<Vec<_>>()}
        </List>
    }
}

#[component]
pub fn list_tracks(
    tracks: Vec<Track>,
    now_playing_id: Option<u32>,
    show_track_number: bool,
    parent_id: String,
) -> impl IntoView {
    html! {
        <List>
            {tracks
                .into_iter()
                .map(|track| {
                    let now_playing = now_playing_id.is_some_and(|id| id == track.id);
                    let parent_id = parent_id.clone();
                    html! {
                        <ListItem>
                            <button

                                hx-swap="none"
                                hx-put=format!("{}/play/{}", parent_id, track.position)
                                class="flex justify-between items-center w-full text-left"
                            >
                                <span class="flex gap-4 items-center">
                                    <span class="w-5 text-center">
                                        {now_playing
                                            .then(|| {
                                                html! {
                                                    <span class="text-blue-500 size-4">
                                                        <Play />
                                                    </span>
                                                }
                                            })}
                                        {(!now_playing && show_track_number)
                                            .then(|| {
                                                html! {
                                                    <span class="text-gray-400">{track.position}.</span>
                                                }
                                            })}
                                    </span>

                                    <h2 class="truncate">{track.title}</h2>
                                </span>
                                <Info
                                    explicit=track.explicit
                                    hires_available=track.hires_available
                                />
                            </button>
                        </ListItem>
                    }
                })
                .collect::<Vec<_>>()}
        </List>
    }
}

#[component]
pub fn list_playlists(playlists: Vec<Playlist>) -> impl IntoView {
    html! {
        <List>
            {playlists
                .into_iter()
                .map(|playlist| {
                    html! {
                        <ListItem>
                            <a
                                class="flex gap-4 items-center w-full text-lg text-left"
                                href=format!("/playlist/{}", playlist.id)
                            >
                                <img
                                    class="text-sm text-gray-500 bg-gray-800 rounded-md aspect-square size-12"
                                    alt=playlist.title.clone()
                                    src=playlist.cover_art
                                />
                                <span class="overflow-hidden w-full truncate">
                                    {playlist.title}
                                </span>
                            </a>
                        </ListItem>
                    }
                })
                .collect::<Vec<_>>()}
        </List>
    }
}
