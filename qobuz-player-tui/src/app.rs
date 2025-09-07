use crate::{
    discover::DiscoverState, favorites::FavoritesState, now_playing::NowPlayingState, popup::Popup,
    queue::QueueState, search::SearchState,
};
use core::fmt;
use image::load_from_memory;
use qobuz_player_controls::tracklist::{self, Tracklist};
use qobuz_player_state::State;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    widgets::*,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use reqwest::Client;
use std::{io, sync::Arc};
use tokio::time::{self, Duration};

pub(crate) struct App {
    pub(crate) state: Arc<State>,
    pub(crate) current_screen: Tab,
    pub(crate) exit: bool,
    pub(crate) should_draw: bool,
    pub(crate) app_state: AppState,
    pub(crate) now_playing: NowPlayingState,
    pub(crate) favorites: FavoritesState,
    pub(crate) search: SearchState,
    pub(crate) queue: QueueState,
    pub(crate) discover: DiscoverState,
}

#[derive(Default, PartialEq)]
pub(crate) enum AppState {
    #[default]
    Normal,
    Popup(Popup),
    Help,
}

pub(crate) enum Output {
    Consumed,
    NotConsumed,
    Popup(Popup),
    PlayOutcome(PlayOutcome),
}

pub(crate) enum PlayOutcome {
    Album(String),
    Playlist((u32, bool)),
    Track(u32),
    SkipToPosition(u32),
}

#[derive(Default, PartialEq)]
pub(crate) enum Tab {
    #[default]
    Favorites,
    Search,
    Queue,
    Discover,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tab::Favorites => write!(f, "Favorites"),
            Tab::Search => write!(f, "Search"),
            Tab::Queue => write!(f, "Queue"),
            Tab::Discover => write!(f, "Discover"),
        }
    }
}

impl Tab {
    pub(crate) const VALUES: [Self; 4] = [Tab::Favorites, Tab::Search, Tab::Queue, Tab::Discover];
}

pub(crate) struct FilteredListState<T> {
    pub(crate) filter: Vec<T>,
    pub(crate) all_items: Vec<T>,
    pub(crate) state: TableState,
}

pub(crate) struct UnfilteredListState<T> {
    pub(crate) items: Vec<T>,
    pub(crate) state: TableState,
}

impl App {
    pub(crate) async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut receiver = self.state.broadcast.notify_receiver();
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
                            qobuz_player_controls::notification::Notification::Position { position } => {
                                self.now_playing.duration_ms = position.mseconds() as u32;
                                self.should_draw = true;
                            },
                            qobuz_player_controls::notification::Notification::CurrentTrackList { tracklist } => {
                                self.queue.queue.items = tracklist.queue().to_vec();
                                let status = self.state.target_status.read().await;
                                self.now_playing = get_current_state(tracklist, *status).await;
                                self.should_draw = true;
                            }
                            qobuz_player_controls::notification::Notification::Quit => {
                                self.exit = true;
                            }
                            qobuz_player_controls::notification::Notification::Message { message: _ } => (),
                            qobuz_player_controls::notification::Notification::Volume { volume: _ } => (),
                            qobuz_player_controls::notification::Notification::Play(_play_notification) => {},
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
                terminal.draw(|frame| self.render(frame))?;
                self.should_draw = false;
            }
        }

        Ok(())
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;

        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match &mut self.app_state {
                    AppState::Help => {
                        self.app_state = AppState::Normal;
                        self.should_draw = true;
                        return Ok(());
                    }
                    AppState::Popup(popup) => {
                        if key_event.code == KeyCode::Esc {
                            self.app_state = AppState::Normal;
                            self.should_draw = true;
                            return Ok(());
                        }

                        if let Some(outcome) = popup.handle_event(key_event.code).await {
                            self.handle_playoutcome(outcome).await;
                            self.app_state = AppState::Normal;
                        };

                        self.should_draw = true;
                        return Ok(());
                    }
                    _ => {}
                };

                let screen_output = match self.current_screen {
                    Tab::Favorites => self.favorites.handle_events(event).await,
                    Tab::Search => self.search.handle_events(event).await,
                    Tab::Queue => self.queue.handle_events(event).await,
                    Tab::Discover => self.discover.handle_events(event).await,
                };

                match screen_output {
                    Output::Consumed => {
                        self.should_draw = true;
                        return Ok(());
                    }
                    Output::NotConsumed => {}
                    Output::Popup(popup) => {
                        self.app_state = AppState::Popup(popup);
                        self.should_draw = true;
                        return Ok(());
                    }
                    Output::PlayOutcome(outcome) => {
                        self.handle_playoutcome(outcome).await;
                    }
                }

                match key_event.code {
                    KeyCode::Char('h') => {
                        self.app_state = AppState::Help;
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
                    KeyCode::Char('4') => {
                        self.navigate_to_discover();
                        self.should_draw = true;
                    }
                    KeyCode::Char(' ') => {
                        self.state.broadcast.play_pause();
                        self.should_draw = true;
                    }
                    KeyCode::Char('n') => {
                        self.state.broadcast.next();
                        self.should_draw = true;
                    }
                    KeyCode::Char('p') => {
                        self.state.broadcast.previous();
                        self.should_draw = true;
                    }
                    KeyCode::Char('f') => {
                        self.state.broadcast.jump_forward();
                        self.should_draw = true;
                    }
                    KeyCode::Char('b') => {
                        self.state.broadcast.jump_backward();
                        self.should_draw = true;
                    }
                    _ => {}
                };
            }

            Event::Resize(_, _) => self.should_draw = true,
            _ => {}
        };
        Ok(())
    }

    async fn handle_playoutcome(&mut self, outcome: PlayOutcome) {
        match outcome {
            PlayOutcome::Album(id) => {
                self.state.broadcast.play_album(&id, 0);
            }

            PlayOutcome::Playlist(outcome) => {
                self.state.broadcast.play_playlist(outcome.0, 0, outcome.1);
            }

            PlayOutcome::Track(id) => {
                self.state.broadcast.play_track(id);
            }

            PlayOutcome::SkipToPosition(index) => {
                self.state.broadcast.skip_to_position(index, true);
            }
        }
    }

    fn navigate_to_favorites(&mut self) {
        self.current_screen = Tab::Favorites;
    }

    fn navigate_to_search(&mut self) {
        self.search.editing = true;
        self.current_screen = Tab::Search;
    }

    fn navigate_to_queue(&mut self) {
        self.current_screen = Tab::Queue;
    }

    fn navigate_to_discover(&mut self) {
        self.current_screen = Tab::Discover;
    }

    fn exit(&mut self) {
        self.exit = true;
        self.state.broadcast.quit();
    }
}

async fn fetch_image(image_url: &str) -> Option<(StatefulProtocol, f32)> {
    let client = Client::new();
    let response = client.get(image_url).send().await.ok()?;
    let img_bytes = response.bytes().await.ok()?;

    let image = load_from_memory(&img_bytes).ok()?;
    let ratio = image.width() as f32 / image.height() as f32;

    let picker = Picker::from_query_stdio().ok()?;
    Some((picker.new_resize_protocol(image), ratio))
}

pub(crate) async fn get_current_state(
    tracklist: Tracklist,
    status: tracklist::Status,
) -> NowPlayingState {
    let (entity, image_url, show_tracklist_position) = match &tracklist.list_type() {
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
        duration_ms: 0,
    }
}
