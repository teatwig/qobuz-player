use crate::{
    service::{Album, Artist, Favorites, Playlist, SearchResults, Track, TrackStatus},
    sql::db,
};
use qobuz_api::client::{
    album::Album as QobuzAlbum,
    album_suggestion::AlbumSuggestion,
    api::{self, Client as QobuzClient},
    artist::Artist as QobuzArtist,
    favorites::Favorites as QobuzFavorites,
    playlist::Playlist as QobuzPlaylist,
    release::{Release, Track as QobuzReleaseTrack},
    search_results::SearchAllResults,
    track::Track as QobuzTrack,
};
use std::{collections::BTreeMap, str::FromStr};
use tracing::{debug, info};

pub type Result<T, E = qobuz_api::Error> = std::result::Result<T, E>;

pub async fn make_client(username: Option<&str>, password: Option<&str>) -> Result<QobuzClient> {
    let mut client = api::new(None, None, None).await?;

    setup_client(&mut client, username, password).await
}

/// Setup app_id, secret and user credentials for authentication
async fn setup_client(
    client: &mut QobuzClient,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<QobuzClient> {
    debug!("setting up the api client");

    if let Some(config) = db::get_config().await {
        let mut refresh_config = false;

        if let Some(app_id) = config.app_id {
            debug!("using app_id from cache");
            client.set_app_id(app_id);
        } else {
            debug!("app_id not found, will have to refresh config");
            refresh_config = true;
        }

        if let Some(secret) = config.active_secret {
            debug!("using active secret from cache");
            client.set_active_secret(secret);
        } else {
            debug!("active_secret not found, will have to refresh config");
            refresh_config = true;
        }

        if refresh_config {
            client.refresh().await?;

            if let Some(id) = client.get_app_id() {
                db::set_app_id(id).await;
            }

            if let Some(secret) = client.get_active_secret() {
                db::set_active_secret(secret).await;
            }
        }

        if let Some(token) = config.user_token {
            info!("using token from cache");
            client.set_token(token);
        } else {
            let (username, password): (Option<String>, Option<String>) =
                if let (Some(u), Some(p)) = (username, password) {
                    (Some(u.to_string()), Some(p.to_string()))
                } else if let (Some(u), Some(p)) = (config.username, config.password) {
                    (Some(u), Some(p))
                } else {
                    (None, None)
                };

            if let (Some(username), Some(password)) = (username, password) {
                info!("setting auth using username and password from cache");
                client.login(&username, &password).await?;
                client.test_secrets().await?;

                if let Some(token) = client.get_token() {
                    db::set_user_token(token).await;
                }

                if let Some(secret) = client.get_active_secret() {
                    db::set_active_secret(secret).await;
                }
            }
        }
    }

    Ok(client.clone())
}

impl From<SearchAllResults> for SearchResults {
    fn from(s: SearchAllResults) -> Self {
        Self {
            query: s.query,
            albums: s
                .albums
                .items
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<Album>>(),
            tracks: s
                .tracks
                .items
                .into_iter()
                .map(|t| t.into())
                .collect::<Vec<Track>>(),
            artists: s
                .artists
                .items
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<Artist>>(),
            playlists: s
                .playlists
                .items
                .into_iter()
                .map(|p| p.into())
                .collect::<Vec<Playlist>>(),
        }
    }
}

impl From<QobuzFavorites> for Favorites {
    fn from(s: QobuzFavorites) -> Self {
        Self {
            albums: s
                .albums
                .items
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<Album>>(),
            tracks: s
                .tracks
                .items
                .into_iter()
                .map(|t| t.into())
                .collect::<Vec<Track>>(),
            artists: s
                .artists
                .items
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<Artist>>(),
        }
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
            status: crate::service::TrackStatus::Unplayed,
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

impl From<QobuzPlaylist> for Playlist {
    fn from(value: QobuzPlaylist) -> Self {
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

        let cover_art = if let Some(image) = value.image_rectangle.first() {
            Some(image.clone())
        } else if let Some(images) = value.images300 {
            images.first().cloned()
        } else {
            None
        };

        Self {
            id: value.id as u32,
            title: value.name,
            duration_seconds: value.duration as u32,
            tracks_count: value.tracks_count as u32,
            cover_art,
            tracks,
        }
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
