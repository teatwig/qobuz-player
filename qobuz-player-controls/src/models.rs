use qobuz_player_client::qobuz_models::{
    album::Album as QobuzAlbum,
    album_suggestion::AlbumSuggestion,
    artist::Artist as QobuzArtist,
    artist_page::{self, ArtistPage as QobuzArtistPage},
    playlist::Playlist as QobuzPlaylist,
    release::Release,
    search_results::SearchAllResults,
    track::Track as QobuzTrack,
};
use std::{fmt::Debug, str::FromStr};

use crate::CONFIGURATION;

pub fn parse_search_results(search_results: SearchAllResults, user_id: i64) -> SearchResults {
    SearchResults {
        query: search_results.query,
        albums: search_results
            .albums
            .items
            .into_iter()
            .map(|a| a.into())
            .collect(),
        artists: search_results
            .artists
            .items
            .into_iter()
            .map(|a| a.into())
            .collect(),
        playlists: search_results
            .playlists
            .items
            .into_iter()
            .map(|p| parse_playlist(p, user_id))
            .collect(),
        tracks: search_results
            .tracks
            .items
            .into_iter()
            .map(|t| t.into())
            .collect(),
    }
}

impl From<Release> for AlbumSimple {
    fn from(release: Release) -> Self {
        Self {
            id: release.id,
            title: release.title,
            artist: Artist {
                id: release.artist.id,
                name: release.artist.name.display,
                ..Default::default()
            },
            image: release.image.large,
            available: release.rights.streamable,
            hires_available: release.rights.hires_streamable,
            explicit: release.parental_warning,
        }
    }
}

impl From<Album> for AlbumSimple {
    fn from(value: Album) -> Self {
        Self {
            id: value.id,
            title: value.title,
            artist: value.artist,
            image: value.image,
            available: value.available,
            hires_available: value.hires_available,
            explicit: value.explicit,
        }
    }
}

impl From<AlbumSuggestion> for Album {
    fn from(s: AlbumSuggestion) -> Self {
        let year = chrono::NaiveDate::from_str(&s.dates.original)
            .expect("failed to parse date")
            .format("%Y");

        let artist = s.artists.and_then(|vec| vec.into_iter().next());
        let (artist_id, artist_name) = artist.map_or((0, "Unknown".into()), |artist| {
            (artist.id as u32, artist.name)
        });

        Self {
            id: s.id,
            title: s.title,
            artist: Artist {
                id: artist_id,
                name: artist_name,
                ..Default::default()
            },
            release_year: year
                .to_string()
                .parse::<u32>()
                .expect("error converting year"),
            hires_available: hifi_available(s.rights.hires_streamable),
            explicit: s.parental_warning,
            total_tracks: s.track_count as u32,
            tracks: Default::default(),
            available: s.rights.streamable,
            image: s.image.large,
            image_thumbnail: s.image.small,
            duration_seconds: s.duration.map_or(0, |duration| duration as u32),
        }
    }
}

impl From<QobuzAlbum> for Album {
    fn from(value: QobuzAlbum) -> Self {
        let year = chrono::NaiveDate::from_str(&value.release_date_original)
            .expect("failed to parse date")
            .format("%Y");

        let tracks = value.tracks.map_or(Default::default(), |tracks| {
            tracks
                .items
                .into_iter()
                .map(|t| Track {
                    id: t.id,
                    title: t.title,
                    number: t.track_number,
                    explicit: t.parental_warning,
                    hires_available: t.hires_streamable,
                    available: t.streamable,
                    status: Default::default(),
                    image: Some(value.image.large.clone()),
                    image_thumbnail: Some(value.image.small.clone()),
                    duration_seconds: t.duration,
                    artist_name: Some(value.artist.name.clone()),
                    artist_id: Some(value.artist.id),
                    album_title: Some(value.title.clone()),
                    album_id: Some(value.id.clone()),
                })
                .collect()
        });

        Self {
            id: value.id,
            title: value.title,
            artist: value.artist.into(),
            total_tracks: value.tracks_count as u32,
            release_year: year
                .to_string()
                .parse::<u32>()
                .expect("error converting year"),
            hires_available: hifi_available(value.hires_streamable),
            explicit: value.parental_warning,
            available: value.streamable,
            tracks,
            image: value.image.large,
            image_thumbnail: value.image.small,
            duration_seconds: value.duration.map_or(0, |duration| duration as u32),
        }
    }
}

pub fn image_to_string(value: artist_page::Image) -> String {
    format!(
        "https://static.qobuz.com/images/artists/covers/large/{}.{}",
        value.hash, value.format
    )
}

impl From<QobuzArtistPage> for ArtistPage {
    fn from(value: QobuzArtistPage) -> Self {
        let artist_image_url = value.images.portrait.map(image_to_string);

        Self {
            id: value.id,
            name: value.name.display.clone(),
            image: artist_image_url.clone(),
            top_tracks: value
                .top_tracks
                .into_iter()
                .map(|t| {
                    let album_image_url = t.album.image.large;
                    let album_image_url_small = t.album.image.small;
                    Track {
                        id: t.id,
                        number: t.physical_support.track_number,
                        title: t.title,
                        explicit: t.parental_warning,
                        hires_available: t.rights.hires_streamable,
                        available: t.rights.streamable,
                        status: Default::default(),
                        image: Some(album_image_url),
                        image_thumbnail: Some(album_image_url_small),
                        duration_seconds: t.duration,
                        artist_name: Some(value.name.display.clone()),
                        artist_id: Some(value.id),
                        album_title: Some(t.album.title),
                        album_id: Some(t.album.id),
                    }
                })
                .collect(),
        }
    }
}

impl From<QobuzArtist> for Artist {
    fn from(value: QobuzArtist) -> Self {
        Self {
            id: value.id,
            name: value.name,
            image: value.image.map(|i| i.large),
        }
    }
}

pub fn parse_playlist(playlist: QobuzPlaylist, user_id: i64) -> Playlist {
    let tracks = playlist.tracks.map_or(Default::default(), |tracks| {
        tracks.items.into_iter().map(|t| t.into()).collect()
    });

    let image = if let Some(image) = playlist.image_rectangle.first() {
        Some(image.clone())
    } else if let Some(images) = playlist.images300 {
        images.first().cloned()
    } else {
        None
    };

    Playlist {
        id: playlist.id as u32,
        is_owned: user_id == playlist.owner.id,
        title: playlist.name,
        duration_seconds: playlist.duration as u32,
        tracks_count: playlist.tracks_count as u32,
        image,
        tracks,
    }
}

impl From<QobuzTrack> for Track {
    fn from(value: QobuzTrack) -> Self {
        let artist = if let Some(p) = &value.performer {
            Some(Artist {
                id: p.id as u32,
                name: p.name.clone(),
                image: None,
            })
        } else {
            value.album.as_ref().map(|a| a.clone().artist.into())
        };

        let image = value.album.as_ref().map(|a| a.image.large.clone());
        let image_thumbnail = value.album.as_ref().map(|a| a.image.small.clone());

        Self {
            id: value.id,
            number: value.track_number,
            title: value.title,
            duration_seconds: value.duration,
            explicit: value.parental_warning,
            hires_available: hifi_available(value.hires_streamable),
            available: value.streamable,
            status: Default::default(),
            image,
            image_thumbnail,
            artist_name: artist.as_ref().map(move |a| a.name.clone()),
            artist_id: artist.as_ref().map(move |a| a.id),
            album_title: value.album.as_ref().map(|a| a.title.clone()),
            album_id: value.album.as_ref().map(|a| a.id.clone()),
        }
    }
}

fn hifi_available(track_has_hires_available: bool) -> bool {
    if !track_has_hires_available {
        return false;
    }

    match CONFIGURATION.get().unwrap().max_audio_quality {
        qobuz_player_client::client::AudioQuality::Mp3 => false,
        qobuz_player_client::client::AudioQuality::CD => false,
        qobuz_player_client::client::AudioQuality::HIFI96 => true,
        qobuz_player_client::client::AudioQuality::HIFI192 => true,
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum TrackStatus {
    Played,
    Playing,
    #[default]
    Unplayed,
    Unplayable,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Track {
    pub id: u32,
    pub title: String,
    pub number: u32,
    pub explicit: bool,
    pub hires_available: bool,
    pub available: bool,
    pub status: TrackStatus,
    pub image: Option<String>,
    pub image_thumbnail: Option<String>,
    pub duration_seconds: u32,
    pub artist_name: Option<String>,
    pub artist_id: Option<u32>,
    pub album_title: Option<String>,
    pub album_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: Artist,
    pub release_year: u32,
    pub hires_available: bool,
    pub explicit: bool,
    pub total_tracks: u32,
    pub tracks: Vec<Track>,
    pub available: bool,
    pub image: String,
    pub image_thumbnail: String,
    pub duration_seconds: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlbumSimple {
    pub id: String,
    pub title: String,
    pub artist: Artist,
    pub image: String,
    pub available: bool,
    pub hires_available: bool,
    pub explicit: bool,
}

#[derive(Default, Debug, Clone)]
pub struct SearchResults {
    pub query: String,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
    pub tracks: Vec<Track>,
}

#[derive(Default, Debug, Clone)]
pub struct Favorites {
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Artist {
    pub id: u32,
    pub name: String,
    pub image: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ArtistPage {
    pub id: u32,
    pub name: String,
    pub image: Option<String>,
    pub top_tracks: Vec<Track>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Playlist {
    pub is_owned: bool,
    pub title: String,
    pub duration_seconds: u32,
    pub tracks_count: u32,
    pub id: u32,
    pub image: Option<String>,
    pub tracks: Vec<Track>,
}
