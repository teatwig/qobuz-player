use crate::{
    popup::{ArtistPopupState, PlaylistPopupState, Popup},
    ui,
};
use core::fmt;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use image::load_from_memory;
use qobuz_player_controls::{
    models::{Album, Artist, Playlist, Track},
    tracklist::{self, Tracklist},
};
use ratatui::{DefaultTerminal, widgets::*};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use reqwest::Client;
use std::io;
use tokio::time::{self, Duration};
use tui_input::{Input, backend::crossterm::EventHandler};

pub struct App {
    pub current_screen: Tab,
    pub current_subtab: SubTab,
    pub exit: bool,
    pub should_draw: bool,
    pub state: State,
    pub favorite_filter: Input,
    pub search_filter: Input,
    pub now_playing: NowPlayingState,
    pub favorite_albums: FilteredListState<Album>,
    pub favorite_artists: FilteredListState<Artist>,
    pub favorite_playlists: FilteredListState<Playlist>,
    pub search_albums: UnfilteredListState<Album>,
    pub search_artists: UnfilteredListState<Artist>,
    pub search_playlists: UnfilteredListState<Playlist>,
    pub queue: UnfilteredListState<Track>,
}

#[derive(Default, PartialEq)]
pub enum State {
    #[default]
    Normal,
    Editing,
    Popup(Popup),
    Help,
}

#[derive(Default, PartialEq)]
pub enum Tab {
    #[default]
    Favorites,
    Search,
    Queue,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tab::Favorites => write!(f, "Favorites"),
            Tab::Search => write!(f, "Search"),
            Tab::Queue => write!(f, "Queue"),
        }
    }
}

impl Tab {
    pub const VALUES: [Self; 3] = [Tab::Favorites, Tab::Search, Tab::Queue];
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum SubTab {
    #[default]
    Albums,
    Artists,
    Playlists,
}

impl fmt::Display for SubTab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Albums => write!(f, "Albums"),
            Self::Artists => write!(f, "Artists"),
            Self::Playlists => write!(f, "Playlists"),
        }
    }
}

impl SubTab {
    pub const VALUES: [Self; 3] = [Self::Albums, Self::Artists, Self::Playlists];

    pub fn next(self) -> Self {
        let index = Self::VALUES.iter().position(|&x| x == self).unwrap();
        Self::VALUES[(index + 1) % Self::VALUES.len()]
    }

    pub fn previous(self) -> Self {
        let index = Self::VALUES.iter().position(|&x| x == self).unwrap();
        let len = Self::VALUES.len();
        Self::VALUES[(index + len - 1) % len]
    }
}

#[derive(Default)]
pub struct NowPlayingState {
    pub image: Option<StatefulProtocol>,
    pub entity_title: Option<String>,
    pub playing_track: Option<Track>,
    pub tracklist_length: u32,
    pub tracklist_position: u32,
    pub show_tracklist_position: bool,
    pub status: tracklist::Status,
    pub duration_s: u32,
}

pub struct FilteredListState<T> {
    pub filter: Vec<T>,
    pub all_items: Vec<T>,
    pub state: TableState,
}

pub struct UnfilteredListState<T> {
    pub items: Vec<T>,
    pub state: TableState,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut receiver = qobuz_player_controls::notify_receiver();
        let mut tick_interval = time::interval(Duration::from_millis(10));

        while !self.exit {
            tokio::select! {
                maybe_notification = receiver.recv() => {
                    if let Ok(notification) = maybe_notification {
                        match notification {
                            qobuz_player_controls::notification::Notification::Status { status } => {
                                self.now_playing.status = status;
                                self.should_draw = true;
                            },
                            qobuz_player_controls::notification::Notification::Position { clock } => {
                                self.now_playing.duration_s = clock.seconds() as u32;
                                self.should_draw = true;
                            },
                            qobuz_player_controls::notification::Notification::CurrentTrackList { list } => {
                                self.now_playing = get_current_state(&list).await;
                                self.queue.items = list.queue;
                                self.should_draw = true;
                            }
                            qobuz_player_controls::notification::Notification::Quit => {
                                self.exit = true;
                            }
                            qobuz_player_controls::notification::Notification::Message { message: _ } => (),
                            qobuz_player_controls::notification::Notification::Volume { volume: _ } => (),
                        }
                    }
                }

                _ = tick_interval.tick() => {
                    if event::poll(Duration::from_millis(0))? {
                        self.handle_events().await.unwrap();
                    }
                }
            }

            if self.should_draw {
                terminal.draw(|frame| ui::render(self, frame))?;
                self.should_draw = false;
            }
        }

        Ok(())
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;

        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match &mut self.state {
                    State::Help => {
                        self.state = State::Normal;
                        self.should_draw = true;
                    }
                    State::Normal => match key_event.code {
                        KeyCode::Char('h') => {
                            self.state = State::Help;
                            self.should_draw = true;
                        }
                        KeyCode::Char('q') => {
                            self.should_draw = true;
                            self.exit()
                        }
                        KeyCode::Char('1') => {
                            self.navigate_to_favorites();
                            self.should_draw = true;
                        }
                        KeyCode::Char('2') => {
                            self.navigate_to_search();
                            self.should_draw = true;
                        }
                        KeyCode::Char('3') => {
                            self.navigate_to_queue();
                            self.should_draw = true;
                        }
                        KeyCode::Char(' ') => {
                            qobuz_player_controls::play_pause().await.unwrap();
                            self.should_draw = true;
                        }
                        KeyCode::Char('n') => {
                            qobuz_player_controls::next().await.unwrap();
                            self.should_draw = true;
                        }
                        KeyCode::Char('p') => {
                            qobuz_player_controls::previous().await.unwrap();
                            self.should_draw = true;
                        }
                        KeyCode::Char('e') => {
                            self.start_editing();
                            self.should_draw = true;
                        }
                        KeyCode::Down => {
                            let state = self.current_list_state();
                            state.select_next();
                            self.should_draw = true;
                        }
                        KeyCode::Up => {
                            let state = self.current_list_state();
                            state.select_previous();
                            self.should_draw = true;
                        }
                        KeyCode::Left => {
                            self.cycle_subtab_backwards();
                            self.should_draw = true;
                        }
                        KeyCode::Right => {
                            self.cycle_subtab();
                            self.should_draw = true;
                        }
                        KeyCode::Enter => {
                            match self.current_screen {
                                Tab::Favorites => match self.current_subtab {
                                    SubTab::Albums => {
                                        let index = self.favorite_albums.state.selected();

                                        let id = index
                                            .map(|index| &self.favorite_albums.filter[index])
                                            .map(|album| album.id.clone());

                                        if let Some(id) = id {
                                            qobuz_player_controls::play_album(&id, 0)
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    SubTab::Artists => {
                                        let index = self.favorite_artists.state.selected();
                                        let selected =
                                            index.map(|index| &self.favorite_artists.filter[index]);

                                        let Some(selected) = selected else {
                                            return Ok(());
                                        };

                                        let artist_albums =
                                            qobuz_player_controls::artist_albums(selected.id)
                                                .await
                                                .unwrap();

                                        self.state =
                                            State::Popup(Popup::Artist(ArtistPopupState {
                                                artist_name: selected.name.clone(),
                                                albums: artist_albums,
                                                state: Default::default(),
                                            }));
                                    }
                                    SubTab::Playlists => {
                                        let index = self.favorite_playlists.state.selected();
                                        let selected = index
                                            .map(|index| &self.favorite_playlists.filter[index]);

                                        let Some(selected) = selected else {
                                            return Ok(());
                                        };

                                        self.state =
                                            State::Popup(Popup::Playlist(PlaylistPopupState {
                                                playlist_name: selected.title.clone(),
                                                playlist_id: selected.id,
                                                shuffle: false,
                                            }))
                                    }
                                },
                                Tab::Search => match self.current_subtab {
                                    SubTab::Albums => {
                                        let index = self.search_albums.state.selected();

                                        let id = index
                                            .map(|index| &self.search_albums.items[index])
                                            .map(|album| album.id.clone());

                                        if let Some(id) = id {
                                            qobuz_player_controls::play_album(&id, 0)
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    SubTab::Artists => {
                                        let index = self.search_artists.state.selected();
                                        let selected =
                                            index.map(|index| &self.search_artists.items[index]);

                                        let Some(selected) = selected else {
                                            return Ok(());
                                        };

                                        let artist_albums =
                                            qobuz_player_controls::artist_albums(selected.id)
                                                .await
                                                .unwrap();

                                        self.state =
                                            State::Popup(Popup::Artist(ArtistPopupState {
                                                artist_name: selected.name.clone(),
                                                albums: artist_albums,
                                                state: Default::default(),
                                            }));
                                    }
                                    SubTab::Playlists => {
                                        let index = self.search_playlists.state.selected();
                                        let selected =
                                            index.map(|index| &self.search_playlists.items[index]);

                                        let Some(selected) = selected else {
                                            return Ok(());
                                        };

                                        self.state =
                                            State::Popup(Popup::Playlist(PlaylistPopupState {
                                                playlist_name: selected.title.clone(),
                                                playlist_id: selected.id,
                                                shuffle: false,
                                            }))
                                    }
                                },

                                Tab::Queue => {
                                    let index = self.search_playlists.state.selected();

                                    if let Some(index) = index {
                                        qobuz_player_controls::skip_to_position(index as u32, true)
                                            .await
                                            .unwrap();
                                    }
                                }
                            }

                            self.should_draw = true;
                        }
                        _ => {}
                    },
                    State::Editing => match key_event.code {
                        KeyCode::Esc => {
                            self.stop_editing();
                            if matches!(self.current_screen, Tab::Search)
                                && !self.search_filter.value().is_empty()
                            {
                                self.update_search().await;
                            };
                            self.should_draw = true;
                        }
                        KeyCode::Enter => {
                            self.stop_editing();
                            if matches!(self.current_screen, Tab::Search)
                                && !self.search_filter.value().is_empty()
                            {
                                self.update_search().await;
                            };
                            self.should_draw = true;
                        }
                        _ => match self.current_screen {
                            Tab::Favorites => {
                                self.favorite_filter.handle_event(&event);

                                self.favorite_albums.filter =
                                    self.favorite_albums
                                        .all_items
                                        .iter()
                                        .filter(|x| {
                                            x.title.to_lowercase().contains(
                                                &self.favorite_filter.value().to_lowercase(),
                                            ) || x.artist.name.to_lowercase().contains(
                                                &self.favorite_filter.value().to_lowercase(),
                                            )
                                        })
                                        .cloned()
                                        .collect();

                                self.favorite_artists.filter = self
                                    .favorite_artists
                                    .all_items
                                    .iter()
                                    .filter(|x| {
                                        x.name
                                            .to_lowercase()
                                            .contains(&self.favorite_filter.value().to_lowercase())
                                    })
                                    .cloned()
                                    .collect();

                                self.favorite_playlists.filter = self
                                    .favorite_playlists
                                    .all_items
                                    .iter()
                                    .filter(|x| {
                                        x.title
                                            .to_lowercase()
                                            .contains(&self.favorite_filter.value().to_lowercase())
                                    })
                                    .cloned()
                                    .collect();

                                self.should_draw = true;
                            }
                            Tab::Search => {
                                self.search_filter.handle_event(&event);
                                self.should_draw = true;
                            }
                            Tab::Queue => (),
                        },
                    },
                    State::Popup(popup) => {
                        if key_event.code == KeyCode::Esc {
                            self.state = State::Normal;
                            self.should_draw = true;
                            return Ok(());
                        }
                        match popup {
                            Popup::Artist(artist_popup_state) => match key_event.code {
                                KeyCode::Up => {
                                    artist_popup_state.state.select_previous();
                                    self.should_draw = true;
                                }
                                KeyCode::Down => {
                                    artist_popup_state.state.select_next();
                                    self.should_draw = true;
                                }
                                KeyCode::Enter => {
                                    let index = artist_popup_state.state.selected();

                                    let id = index
                                        .map(|index| &artist_popup_state.albums[index])
                                        .map(|album| album.id.clone());

                                    if let Some(id) = id {
                                        qobuz_player_controls::play_album(&id, 0).await.unwrap();
                                        self.state = State::Normal;
                                        self.should_draw = true;
                                        return Ok(());
                                    }
                                }
                                _ => {}
                            },
                            Popup::Playlist(playlist_popup_state) => match key_event.code {
                                KeyCode::Left => {
                                    playlist_popup_state.shuffle = !playlist_popup_state.shuffle;
                                    self.should_draw = true;
                                }
                                KeyCode::Right => {
                                    playlist_popup_state.shuffle = !playlist_popup_state.shuffle;
                                    self.should_draw = true;
                                }
                                KeyCode::Enter => {
                                    let id = playlist_popup_state.playlist_id;

                                    qobuz_player_controls::play_playlist(
                                        id,
                                        0,
                                        playlist_popup_state.shuffle,
                                    )
                                    .await
                                    .unwrap();
                                    self.state = State::Normal;
                                    self.should_draw = true;
                                    return Ok(());
                                }
                                _ => {}
                            },
                        }

                        return Ok(());
                    }
                }
            }
            Event::Resize(_, _) => self.should_draw = true,
            _ => {}
        };
        Ok(())
    }

    fn start_editing(&mut self) {
        self.state = State::Editing
    }

    fn stop_editing(&mut self) {
        self.state = State::Normal
    }

    fn navigate_to_favorites(&mut self) {
        self.current_screen = Tab::Favorites;
    }

    fn navigate_to_search(&mut self) {
        self.current_screen = Tab::Search;
    }

    fn navigate_to_queue(&mut self) {
        self.current_screen = Tab::Queue;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn cycle_subtab_backwards(&mut self) {
        self.current_subtab = self.current_subtab.previous();
    }

    fn cycle_subtab(&mut self) {
        self.current_subtab = self.current_subtab.next();
    }

    fn current_list_state(&mut self) -> &mut TableState {
        match &self.current_screen {
            Tab::Favorites => match self.current_subtab {
                SubTab::Albums => &mut self.favorite_albums.state,
                SubTab::Artists => &mut self.favorite_artists.state,
                SubTab::Playlists => &mut self.favorite_playlists.state,
            },
            Tab::Search => match self.current_subtab {
                SubTab::Albums => &mut self.search_albums.state,
                SubTab::Artists => &mut self.search_artists.state,
                SubTab::Playlists => &mut self.search_playlists.state,
            },
            Tab::Queue => &mut self.queue.state,
        }
    }

    async fn update_search(&mut self) {
        let search_results = qobuz_player_controls::search(self.search_filter.value().to_string())
            .await
            .unwrap();

        self.search_albums.items = search_results.albums;
        self.search_artists.items = search_results.artists;
        self.search_playlists.items = search_results.playlists;

        self.should_draw = true;
    }
}

async fn fetch_image(image_url: &str) -> Option<StatefulProtocol> {
    let client = Client::new();
    let response = client.get(image_url).send().await.ok()?;
    let img_bytes = response.bytes().await.ok()?;

    let image = load_from_memory(&img_bytes).ok()?;

    let picker = Picker::from_query_stdio().ok()?;
    Some(picker.new_resize_protocol(image))
}

async fn get_current_state(tracklist: &Tracklist) -> NowPlayingState {
    let (entity, image_url, show_tracklist_position) = match &tracklist.list_type {
        qobuz_player_controls::tracklist::TracklistType::Album(tracklist) => (
            Some(tracklist.title.clone()),
            tracklist.image.clone(),
            false,
        ),
        qobuz_player_controls::tracklist::TracklistType::Playlist(tracklist) => {
            (Some(tracklist.title.clone()), tracklist.image.clone(), true)
        }
        qobuz_player_controls::tracklist::TracklistType::TopTracks(tracklist) => (
            Some(tracklist.artist_name.clone()),
            tracklist.image.clone(),
            true,
        ),
        qobuz_player_controls::tracklist::TracklistType::Track(tracklist) => {
            (None, tracklist.image.clone(), true)
        }
        qobuz_player_controls::tracklist::TracklistType::None => (None, None, false),
    };

    let status = qobuz_player_controls::current_state().await;

    let track = tracklist.current_track().cloned();

    let image = if let Some(image_url) = image_url {
        Some(fetch_image(&image_url).await)
    } else {
        None
    }
    .flatten();

    let tracklist_length = tracklist.total();

    NowPlayingState {
        image,
        entity_title: entity,
        playing_track: track,
        tracklist_length,
        status,
        tracklist_position: tracklist.current_position(),
        show_tracklist_position,
        duration_s: 0,
    }
}
