use qobuz_player_client::qobuz_models::{
    album::Album as QobuzAlbum,
    album_suggestion::AlbumSuggestion,
    artist::Artist as QobuzArtist,
    playlist::Playlist as QobuzPlaylist,
    release::{Release, Track as QobuzReleaseTrack},
    search_results::SearchAllResults,
    track::Track as QobuzTrack,
    Image,
};
use std::{collections::BTreeMap, fmt::Debug, str::FromStr};

// pub type Result<T, E = qobuz_player_client::Error> = std::result::Result<T, E>;

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

impl From<QobuzReleaseTrack> for Track {
    fn from(s: QobuzReleaseTrack) -> Self {
        Self {
            id: s.id,
            number: s.physical_support.track_number as u32,
            title: s.title,
            album: None,
            artist: Some(Artist {
                id: s.artist.id as u32,
                name: s.artist.name.display,
                ..Default::default()
            }),
            duration_seconds: s.duration as u32,
            explicit: s.parental_warning,
            hires_available: s.rights.streamable,
            sampling_rate: s.audio_info.maximum_sampling_rate,
            bit_depth: s.audio_info.maximum_bit_depth,
            status: TrackStatus::Unplayed,
            track_url: None,
            available: s.rights.streamable,
            cover_art: None,
            position: s.physical_support.track_number as u32,
            media_number: s.physical_support.media_number as u32,
        }
    }
}

impl From<Release> for Album {
    fn from(s: Release) -> Self {
        let year = chrono::NaiveDate::from_str(&s.dates.original)
            .expect("failed to parse date")
            .format("%Y");

        let tracks = if let Some(tracks) = s.tracks {
            let mut position = 1_u32;

            tracks
                .items
                .into_iter()
                .filter_map(|t| {
                    if t.rights.streamable {
                        let mut track: Track = t.into();

                        let next_position = position;
                        track.position = next_position;

                        position += 1;

                        Some((next_position, track))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<u32, Track>>()
        } else {
            BTreeMap::new()
        };

        Self {
            id: s.id,
            title: s.title,
            artist: Artist {
                id: s.artist.id as u32,
                name: s.artist.name.display,
                ..Default::default()
            },
            release_year: year
                .to_string()
                .parse::<u32>()
                .expect("error converting year"),
            hires_available: s.rights.hires_streamable,
            explicit: s.parental_warning,
            total_tracks: s.tracks_count as u32,
            tracks,
            available: s.rights.streamable,
            cover_art: s.image.large,
            cover_art_small: s.image.small,
            duration_seconds: s.duration.map_or(0, |duration| duration as u32),
        }
    }
}

impl From<AlbumSuggestion> for Album {
    fn from(s: AlbumSuggestion) -> Self {
        let year = chrono::NaiveDate::from_str(&s.dates.original)
            .expect("failed to parse date")
            .format("%Y");

        let tracks = BTreeMap::new();

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
            hires_available: s.rights.hires_streamable,
            explicit: s.parental_warning,
            total_tracks: s.track_count as u32,
            tracks,
            available: s.rights.streamable,
            cover_art: s.image.large,
            cover_art_small: s.image.small,
            duration_seconds: s.duration.map_or(0, |duration| duration as u32),
        }
    }
}

impl From<QobuzAlbum> for Album {
    fn from(value: QobuzAlbum) -> Self {
        let year = chrono::NaiveDate::from_str(&value.release_date_original)
            .expect("failed to parse date")
            .format("%Y");

        let tracks = if let Some(tracks) = value.tracks {
            let mut position = 1_u32;

            tracks
                .items
                .into_iter()
                .filter_map(|t| {
                    if t.streamable {
                        let mut track: Track = t.into();

                        let next_position = position;
                        track.position = next_position;

                        position += 1;

                        Some((next_position, track))
                    } else {
                        None
                    }
                })
                .collect::<BTreeMap<u32, Track>>()
        } else {
            BTreeMap::new()
        };

        Self {
            id: value.id,
            title: value.title,
            artist: value.artist.into(),
            total_tracks: value.tracks_count as u32,
            release_year: year
                .to_string()
                .parse::<u32>()
                .expect("error converting year"),
            hires_available: value.hires_streamable,
            explicit: value.parental_warning,
            available: value.streamable,
            tracks,
            cover_art: value.image.large,
            cover_art_small: value.image.small,
            duration_seconds: value.duration.map_or(0, |duration| duration as u32),
        }
    }
}

impl From<&QobuzAlbum> for Album {
    fn from(value: &QobuzAlbum) -> Self {
        value.clone().into()
    }
}

impl From<QobuzArtist> for Artist {
    fn from(a: QobuzArtist) -> Self {
        Self {
            id: a.id as u32,
            name: a.name,
            image: a.image,
            albums: a.albums.map(|a| {
                a.items
                    .into_iter()
                    .map(|a| a.into())
                    .collect::<Vec<Album>>()
            }),
        }
    }
}

pub fn parse_playlist(playlist: QobuzPlaylist, user_id: i64) -> Playlist {
    let tracks = if let Some(tracks) = playlist.tracks {
        let mut position = 1_u32;

        tracks
            .items
            .into_iter()
            .filter_map(|t| {
                if t.streamable {
                    let mut track: Track = t.into();

                    let next_position = position;
                    track.position = next_position;

                    position += 1;

                    Some((next_position, track))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<u32, Track>>()
    } else {
        BTreeMap::new()
    };

    let cover_art = if let Some(image) = playlist.image_rectangle.first() {
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
        cover_art,
        tracks,
    }
}

impl From<QobuzTrack> for Track {
    fn from(value: QobuzTrack) -> Self {
        let album = value.album.as_ref().map(|a| {
            let album: Album = a.into();

            album
        });

        let artist = if let Some(p) = &value.performer {
            Some(Artist {
                id: p.id as u32,
                name: p.name.clone(),
                albums: None,
                image: None,
            })
        } else {
            value.album.as_ref().map(|a| a.clone().artist.into())
        };

        let cover_art = value.album.as_ref().map(|a| a.image.large.clone());

        let status = if value.streamable {
            TrackStatus::Unplayed
        } else {
            TrackStatus::Unplayable
        };

        Self {
            id: value.id as u32,
            number: value.track_number as u32,
            title: value.title,
            album,
            artist,
            duration_seconds: value.duration as u32,
            explicit: value.parental_warning,
            hires_available: value.hires_streamable,
            sampling_rate: value.maximum_sampling_rate.unwrap_or(0.0) as f32,
            bit_depth: value.maximum_bit_depth as u32,
            status,
            track_url: None,
            available: value.streamable,
            position: value.position.unwrap_or(value.track_number as usize) as u32,
            cover_art,
            media_number: value.media_number as u32,
        }
    }
}

impl From<&QobuzTrack> for Track {
    fn from(value: &QobuzTrack) -> Self {
        value.clone().into()
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

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: u32,
    pub number: u32,
    pub title: String,
    pub album: Option<Album>,
    pub artist: Option<Artist>,
    pub duration_seconds: u32,
    pub explicit: bool,
    pub hires_available: bool,
    pub sampling_rate: f32,
    pub bit_depth: u32,
    pub status: TrackStatus,
    pub track_url: Option<String>,
    pub available: bool,
    pub cover_art: Option<String>,
    pub position: u32,
    pub media_number: u32,
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
    pub tracks: BTreeMap<u32, Track>,
    pub available: bool,
    pub cover_art: String,
    pub cover_art_small: String,
    pub duration_seconds: u32,
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
    pub image: Option<Image>,
    pub albums: Option<Vec<Album>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Playlist {
    pub is_owned: bool,
    pub title: String,
    pub duration_seconds: u32,
    pub tracks_count: u32,
    pub id: u32,
    pub cover_art: Option<String>,
    pub tracks: BTreeMap<u32, Track>,
}
