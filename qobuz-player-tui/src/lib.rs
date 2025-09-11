use std::{sync::Arc, time::Duration};

use app::{App, FilteredListState, UnfilteredListState, get_current_state};
use favorites::FavoritesState;
use queue::QueueState;
use ratatui::{prelude::*, widgets::*};
use search::SearchState;
use tokio::{sync::watch, try_join};
use ui::center;

mod app;
mod discover;
mod favorites;
mod now_playing;
mod popup;
mod queue;
mod search;
mod ui;

pub async fn init(state: Arc<qobuz_player_state::State>, position: watch::Receiver<Duration>) {
    let mut terminal = ratatui::init();

    draw_loading_screen(&mut terminal);

    let (favorites, featured_albums, featured_playlists) = try_join!(
        state.client.favorites(),
        state.client.featured_albums(),
        state.client.featured_playlists(),
    )
    .unwrap();

    let featured_albums = featured_albums
        .into_iter()
        .map(|x| {
            (
                x.0,
                UnfilteredListState {
                    items: x.1,
                    state: Default::default(),
                },
            )
        })
        .collect();

    let featured_playlists = featured_playlists
        .into_iter()
        .map(|x| {
            (
                x.0,
                UnfilteredListState {
                    items: x.1,
                    state: Default::default(),
                },
            )
        })
        .collect();

    let tracklist = state.tracklist.read().await.clone();
    let status = *state.target_status.read().await;
    let now_playing = get_current_state(tracklist, status).await;

    let client_clone = state.client.clone();

    let mut app = App {
        state,
        now_playing,
        position,
        current_screen: Default::default(),
        exit: Default::default(),
        should_draw: true,
        app_state: Default::default(),
        favorites: FavoritesState {
            client: client_clone.clone(),
            editing: Default::default(),
            filter: Default::default(),
            albums: FilteredListState {
                filter: favorites.albums.clone(),
                all_items: favorites.albums,
                state: Default::default(),
            },
            artists: FilteredListState {
                filter: favorites.artists.clone(),
                all_items: favorites.artists,
                state: Default::default(),
            },
            playlists: FilteredListState {
                filter: favorites.playlists.clone(),
                all_items: favorites.playlists,
                state: Default::default(),
            },
            sub_tab: Default::default(),
        },
        search: SearchState {
            client: client_clone,
            editing: Default::default(),
            filter: Default::default(),
            albums: UnfilteredListState {
                items: Default::default(),
                state: Default::default(),
            },
            artists: UnfilteredListState {
                items: Default::default(),
                state: Default::default(),
            },
            playlists: UnfilteredListState {
                items: Default::default(),
                state: Default::default(),
            },
            tracks: UnfilteredListState {
                items: Default::default(),
                state: Default::default(),
            },
            sub_tab: Default::default(),
        },
        queue: QueueState {
            queue: UnfilteredListState {
                items: Default::default(),
                state: Default::default(),
            },
        },
        discover: discover::DiscoverState {
            featured_albums,
            featured_playlists,
            sub_tab: Default::default(),
        },
    };

    _ = app.run(&mut terminal).await;
    ratatui::restore();
}

fn draw_loading_screen<B: Backend>(terminal: &mut Terminal<B>) {
    let ascii_art = r#"
              _                       _                       
   __ _  ___ | |__  _   _ ___   _ __ | | __ _ _   _  ___ _ __ 
  / _` |/ _ \| '_ \| | | / __| | '_ \| |/ _` | | | |/ _ \ '__|
 | (_| | (_) | |_) | |_| \__ \ | |_) | | (_| | |_| |  __/ |   
  \__, |\___/|_.__/ \__,_|___/ | .__/|_|\__,_|\__, |\___|_|   
     |_|                       |_|            |___/           
"#;

    terminal
        .draw(|f| {
            let area = center(f.area(), Constraint::Length(64), Constraint::Length(7));
            let paragraph = Paragraph::new(ascii_art)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);
        })
        .unwrap();
}
