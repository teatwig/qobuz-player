use app::{App, FilteredListState, UnfilteredListState};
use favorites::FavoritesState;
use queue::QueueState;
use ratatui::{prelude::*, widgets::*};
use search::SearchState;
use ui::center;

mod app;
mod favorites;
mod now_playing;
mod popup;
mod queue;
mod search;
mod ui;

pub async fn init() {
    let mut terminal = ratatui::init();

    draw_loading_screen(&mut terminal);

    let favorites = qobuz_player_controls::favorites().await.unwrap();

    let mut app = App {
        now_playing: Default::default(),
        current_screen: Default::default(),
        exit: Default::default(),
        should_draw: true,
        state: Default::default(),
        favorites: FavoritesState {
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
    };

    let _app_result = app.run(&mut terminal).await;
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
