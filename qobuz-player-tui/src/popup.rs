use qobuz_player_controls::models::AlbumSimple;
use ratatui::{prelude::*, symbols::border, widgets::*};

use crate::ui::center;

#[derive(PartialEq)]
pub struct ArtistPopupState {
    pub artist_name: String,
    pub albums: Vec<AlbumSimple>,
    pub state: ListState,
}

#[derive(PartialEq)]
pub struct PlaylistPopupState {
    pub playlist_name: String,
    pub playlist_id: u32,
    pub shuffle: bool,
}

#[derive(PartialEq)]
pub enum Popup {
    Artist(ArtistPopupState),
    Playlist(PlaylistPopupState),
}

impl Popup {
    pub fn render(&mut self, frame: &mut Frame) {
        match self {
            Popup::Artist(artist) => {
                let area = center(
                    frame.area(),
                    Constraint::Percentage(50),
                    Constraint::Length(artist.albums.len() as u16 + 2),
                );
                let block = Block::bordered()
                    .title(artist.artist_name.clone())
                    .title_alignment(Alignment::Center)
                    .border_set(border::ROUNDED);

                let list: Vec<ListItem> = artist
                    .albums
                    .iter()
                    .map(|album| ListItem::from(Line::from(album.title.clone())))
                    .collect();

                let list = List::new(list)
                    .block(block)
                    .highlight_style(Style::default().bg(Color::Blue))
                    .highlight_symbol(">")
                    .highlight_spacing(HighlightSpacing::Always);

                frame.render_widget(Clear, area);
                frame.render_stateful_widget(list, area, &mut artist.state);
            }
            Popup::Playlist(playlist) => {
                let area = center(frame.area(), Constraint::Length(18), Constraint::Length(3));
                let block = Block::bordered()
                    .title(playlist.playlist_name.clone())
                    .title_alignment(Alignment::Center)
                    .border_set(border::ROUNDED);

                let tabs = Tabs::new(["Play", "Shuffle"])
                    .block(block)
                    .not_underlined()
                    .highlight_style(Style::default().bg(Color::Blue))
                    .select(if playlist.shuffle { 1 } else { 0 })
                    .divider(symbols::line::VERTICAL);

                frame.render_widget(Clear, area);
                frame.render_widget(tabs, area);
            }
        };
    }
}
