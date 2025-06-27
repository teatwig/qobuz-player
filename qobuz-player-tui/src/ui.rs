use ratatui::{layout::Flex, prelude::*, style::Styled, symbols::border, widgets::*};
use tui_input::Input;

use crate::{
    app::{App, State, SubTab, Tab},
    now_playing::{self},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(10),
        ])
        .split(area);

    let tabs = Tabs::new(
        Tab::VALUES
            .iter()
            .enumerate()
            .map(|(i, tab)| format!("[{}] {}", i + 1, tab)),
    )
    .block(Block::bordered().border_type(BorderType::Rounded))
    .highlight_style(Style::default().bg(Color::Blue))
    .select(
        Tab::VALUES
            .iter()
            .position(|tab| tab == &app.current_screen)
            .unwrap_or(0),
    )
    .divider(symbols::line::VERTICAL);
    frame.render_widget(tabs, chunks[0]);

    if app.now_playing.playing_track.is_some() {
        now_playing::render(frame, chunks[2], &mut app.now_playing);
    }

    let sub_tab = match app.current_screen {
        Tab::Favorites | Tab::Search => Some(app.current_subtab),
        Tab::Queue => None,
    };

    let mut tab_content_area = if app.now_playing.entity_title.is_some() {
        chunks[1]
    } else {
        chunks[1].union(chunks[2])
    };

    let tab_content_area_split = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(tab_content_area);

    match app.current_screen {
        Tab::Favorites => {
            render_input(
                &app.favorite_filter,
                &app.state,
                tab_content_area_split[0],
                frame,
                "Filter",
            );
            tab_content_area = tab_content_area_split[1]
        }
        Tab::Search => {
            render_input(
                &app.search_filter,
                &app.state,
                tab_content_area_split[0],
                frame,
                "Search",
            );
            tab_content_area = tab_content_area_split[1]
        }
        Tab::Queue => {}
    }

    let title = format!(
        " {}{}{} ",
        if sub_tab.is_some() {
            "< "
        } else {
            Default::default()
        },
        app.current_screen,
        sub_tab
            .map(|sub_tab| format!(": {sub_tab} >"))
            .unwrap_or_default()
    );
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_set(border::ROUNDED);

    let row_highlight_style = Style::default().bg(Color::Blue);
    let highlight_symbol = ">";
    let highlight_spacing = HighlightSpacing::Always;

    let (table, list_state): (Table, &mut TableState) = match app.current_screen {
        Tab::Favorites => match app.current_subtab {
            SubTab::Albums => {
                let max_title_length = app
                    .favorite_albums
                    .filter
                    .iter()
                    .map(|album| album.title.len())
                    .max()
                    .unwrap_or(0);

                let max_artist_name_length = app
                    .favorite_albums
                    .filter
                    .iter()
                    .map(|album| album.artist.name.len())
                    .max()
                    .unwrap_or(0);

                let rows: Vec<_> = app
                    .favorite_albums
                    .filter
                    .iter()
                    .map(|album| {
                        Row::new(vec![
                            Span::from(album.title.clone()),
                            Span::from(album.artist.name.clone()),
                            Span::from(album.release_year.to_string()),
                        ])
                    })
                    .collect();

                let empty = rows.is_empty();
                let mut table = Table::new(
                    rows,
                    [
                        Constraint::Fill(max_title_length as u16),
                        Constraint::Min(max_artist_name_length as u16),
                        Constraint::Length(4),
                    ],
                )
                .block(block)
                .row_highlight_style(row_highlight_style)
                .highlight_symbol(highlight_symbol)
                .highlight_spacing(highlight_spacing);

                if !empty {
                    table = table
                        .header(Row::new(["Title", "Artist", "Year"]).add_modifier(Modifier::BOLD));
                }

                (table, &mut app.favorite_albums.state)
            }
            SubTab::Artists => (
                Table::new(
                    app.favorite_artists
                        .filter
                        .iter()
                        .map(|artist| Row::new(Line::from(artist.name.clone())))
                        .collect::<Vec<_>>(),
                    [Constraint::Min(1)],
                ),
                &mut app.favorite_artists.state,
            ),
            SubTab::Playlists => (
                Table::new(
                    app.favorite_playlists
                        .filter
                        .iter()
                        .map(|playlist| Row::new(Line::from(playlist.title.clone())))
                        .collect::<Vec<_>>(),
                    [Constraint::Min(1)],
                )
                .block(block)
                .row_highlight_style(row_highlight_style)
                .highlight_symbol(highlight_symbol)
                .highlight_spacing(highlight_spacing),
                &mut app.favorite_playlists.state,
            ),
        },
        Tab::Search => match app.current_subtab {
            SubTab::Albums => {
                let max_title_length = app
                    .search_albums
                    .items
                    .iter()
                    .map(|album| album.title.len())
                    .max()
                    .unwrap_or(0);

                let max_artist_name_length = app
                    .search_albums
                    .items
                    .iter()
                    .map(|album| album.artist.name.len())
                    .max()
                    .unwrap_or(0);

                let rows: Vec<_> = app
                    .search_albums
                    .items
                    .iter()
                    .map(|album| {
                        Row::new(vec![
                            Span::from(album.title.clone()),
                            Span::from(album.artist.name.clone()),
                            Span::from(album.release_year.to_string()),
                        ])
                    })
                    .collect();

                let empty = rows.is_empty();
                let mut table = Table::new(
                    rows,
                    [
                        Constraint::Fill(max_title_length as u16),
                        Constraint::Min(max_artist_name_length as u16),
                        Constraint::Length(4),
                    ],
                )
                .block(block)
                .row_highlight_style(row_highlight_style)
                .highlight_symbol(highlight_symbol)
                .highlight_spacing(highlight_spacing);

                if !empty {
                    table = table
                        .header(Row::new(["Title", "Artist", "Year"]).add_modifier(Modifier::BOLD));
                }

                (table, &mut app.search_albums.state)
            }
            SubTab::Artists => (
                Table::new(
                    app.search_artists
                        .items
                        .iter()
                        .map(|artist| Row::new(Line::from(artist.name.clone())))
                        .collect::<Vec<_>>(),
                    [Constraint::Min(1)],
                )
                .block(block)
                .row_highlight_style(row_highlight_style)
                .highlight_symbol(highlight_symbol)
                .highlight_spacing(highlight_spacing),
                &mut app.search_artists.state,
            ),
            SubTab::Playlists => (
                Table::new(
                    app.search_playlists
                        .items
                        .iter()
                        .map(|playlist| Row::new(Line::from(playlist.title.clone())))
                        .collect::<Vec<_>>(),
                    [Constraint::Min(1)],
                )
                .block(block)
                .row_highlight_style(row_highlight_style)
                .highlight_symbol(highlight_symbol)
                .highlight_spacing(highlight_spacing),
                &mut app.search_playlists.state,
            ),
        },
        Tab::Queue => (
            Table::new(
                app.queue
                    .items
                    .iter()
                    .enumerate()
                    .map(|(index, track)| {
                        let style = match track.status {
                            qobuz_player_controls::models::TrackStatus::Played => {
                                Style::default().add_modifier(Modifier::CROSSED_OUT)
                            }
                            qobuz_player_controls::models::TrackStatus::Playing => {
                                Style::default().add_modifier(Modifier::BOLD)
                            }
                            qobuz_player_controls::models::TrackStatus::Unplayed => {
                                Style::default()
                            }
                            qobuz_player_controls::models::TrackStatus::Unplayable => {
                                Style::default().add_modifier(Modifier::CROSSED_OUT)
                            }
                        };
                        Row::new(Line::from(vec![
                            Span::from(format!("{}", index + 1)),
                            track.title.clone().set_style(style),
                        ]))
                    })
                    .collect::<Vec<_>>(),
                [Constraint::Length(1), Constraint::Min(1)],
            )
            .block(block)
            .row_highlight_style(row_highlight_style)
            .highlight_symbol(highlight_symbol)
            .highlight_spacing(highlight_spacing),
            &mut app.queue.state,
        ),
    };

    frame.render_stateful_widget(table, tab_content_area, list_state);

    if let State::Popup(popup) = &mut app.state {
        popup.render(frame);
    }

    if matches!(app.state, State::Help) {
        render_help(frame);
    }
}

fn render_input(input: &Input, state: &State, area: Rect, frame: &mut Frame, title: &str) {
    let width = area.width.max(3) - 3;
    let scroll = input.visual_scroll(width as usize);
    let style = match state {
        State::Editing => Color::Yellow.into(),
        _ => Style::default(),
    };
    let input_paragraph = Paragraph::new(input.value())
        .style(style)
        .scroll((0, scroll as u16))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(title)
                .title_alignment(Alignment::Center),
        );

    frame.render_widget(input_paragraph, area);

    if state == &State::Editing {
        let x = input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y + 1))
    }
}

pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn render_help(frame: &mut Frame) {
    let rows = [
        ["Next song", "n"],
        ["Previous song", "p"],
        ["e", "Edit filter"],
        ["esc", "Stop edit filter"],
        ["Up/Down", "Select in list"],
        ["Enter", "Select selected item"],
        ["Left/right", "Cycle subgrup"],
        ["q", "Exit"],
    ];

    let max_left = rows.iter().map(|x| x[0].len()).max().unwrap();
    let max_right = rows.iter().map(|x| x[1].len()).max().unwrap();
    let max = max_left + max_right;

    let rows: Vec<_> = rows.into_iter().map(Row::new).collect();

    let area = center(
        frame.area(),
        Constraint::Length(max as u16 + 2 + 9),
        Constraint::Length(rows.len() as u16 + 2),
    );

    let block = Block::bordered()
        .title("Help")
        .title_alignment(Alignment::Center)
        .border_set(border::ROUNDED);

    let table = Table::default().rows(rows).block(block);

    frame.render_widget(Clear, area);
    frame.render_widget(table, area);
}
