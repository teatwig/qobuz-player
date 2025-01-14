use leptos::{component, prelude::*, IntoView};

use crate::{
    html,
    icons::{MagnifyingGlass, PlayCircle, QueueList, Star},
};

#[derive(PartialEq)]
pub enum Page {
    NowPlaying,
    Queue,
    Favorites,
    Search,
    None,
}

#[component]
fn Navigation(active_page: Page) -> impl IntoView {
    html! {
        <nav class="flex gap-2 justify-evenly w-full p-safe *:flex *:h-[3.25rem] *:w-20 *:flex-col *:items-center *:overflow-visible *:text-nowrap *:px-4 *:py-1 *:text-[10px] *:font-medium *:transition-colors">
            {html! {
                <a
                    href="/"
                    class=if active_page == Page::NowPlaying {
                        "text-blue-500"
                    } else {
                        "text-gray-500"
                    }
                >
                    <PlayCircle />
                    Now Playing
                </a>
            }
                .attr("preload", "mousedown")}
            <a
                href="/queue"
                class=if active_page == Page::Queue { "text-blue-500" } else { "text-gray-500" }
            >
                <QueueList />
                Queue
            </a>
            {html! {
                <a
                    href="/favorites/albums"
                    class=if active_page == Page::Favorites {
                        "text-blue-500"
                    } else {
                        "text-gray-500"
                    }
                >
                    <Star solid=true />
                    Favorites
                </a>
            }
                .attr("preload", "mouseover")}
            {if active_page == Page::Search {
                html! {
                    <button class="text-blue-500" onclick="focusSearch()">
                        <MagnifyingGlass />
                        Search
                    </button>
                }
                    .into_any()
            } else {
                html! {
                    <a href="/search/albums" class="text-gray-500">
                        <MagnifyingGlass />
                        Search
                    </a>
                }
                    .into_any()
            }}
        </nav>
    }
}

#[component]
pub fn page(children: Children, active_page: Page) -> impl IntoView {
    let style_url = "/assets/styles.css?version=15";

    html! {
        <!DOCTYPE html>
        <html lang="en" class="h-full dark">
            <head>
                <title>Qobuz Player</title>
                <link
                    rel="icon"
                    href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>🎵</text></svg>"
                />
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1, maximum-scale=5 viewport-fit=cover"
                />
                <meta name="theme-color" content="#000" />
                <link rel="stylesheet" href=style_url />
                <script src="https://unpkg.com/htmx.org@2.0.4"></script>
                <script src="https://unpkg.com/htmx-ext-sse@2.2.2/sse.js"></script>
                <script src="https://unpkg.com/htmx-ext-preload@2.1.0/preload.js"></script>
                <script src="/assets/script.js?version=1"></script>
            </head>

            <body
                class="flex flex-col justify-between text-gray-50 bg-black h-dvh touch-none overflow-clip px-safe pt-safe"
                hx-ext="sse, preload"
                sse-connect="/sse"
                hx-boost="true"
            >
                <div class="overflow-auto h-full">{children()}</div>

                <Navigation active_page=active_page />
            </body>
        </html>
    }
}
