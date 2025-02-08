use serde::{Deserialize, Serialize};
use snafu::prelude::*;

pub mod album;
pub mod album_suggestion;
pub mod artist;
pub mod artist_page;
pub mod favorites;
pub mod playlist;
pub mod release;
pub mod search_results;
pub mod track;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Composer {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub albums_count: i64,
    pub image: Option<Image>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    pub small: String,
    pub thumbnail: Option<String>,
    pub large: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackURL {
    pub track_id: i32,
    pub duration: i32,
    pub url: String,
    pub format_id: i32,
    pub mime_type: String,
    pub sampling_rate: f64,
    pub bit_depth: i32,
}

pub enum UrlType {
    Album { id: String },
    Playlist { id: i64 },
    Track { id: i32 },
}

#[derive(Snafu, Debug)]
pub enum UrlTypeError {
    #[snafu(display("This uri contains an unfamiliar domain."))]
    WrongDomain,
    #[snafu(display("the url contains an invalid path"))]
    InvalidPath,
    #[snafu(display("the url is invalid."))]
    InvalidUrl,
    #[snafu(display("an unknown error has occurred"))]
    Unknown,
}

pub type ParseUrlResult<T, E = UrlTypeError> = std::result::Result<T, E>;

pub fn parse_url(string_url: &str) -> ParseUrlResult<UrlType> {
    if let Ok(url) = url::Url::parse(string_url) {
        if let (Some(host), Some(mut path)) = (url.host_str(), url.path_segments()) {
            if host == "play.qobuz.com" || host == "open.qobuz.com" {
                debug!("got a qobuz url");

                match path.next() {
                    Some("album") => {
                        debug!("this is an album");
                        let id = path.next().unwrap().to_string();

                        Ok(UrlType::Album { id })
                    }
                    Some("playlist") => {
                        debug!("this is a playlist");
                        let id = path
                            .next()
                            .unwrap()
                            .parse::<i64>()
                            .expect("failed to convert id");

                        Ok(UrlType::Playlist { id })
                    }
                    Some("track") => {
                        debug!("this is a track");
                        let id = path
                            .next()
                            .unwrap()
                            .parse::<i32>()
                            .expect("failed to convert id");

                        Ok(UrlType::Track { id })
                    }
                    None => {
                        debug!("no path, cannot use path");
                        Err(UrlTypeError::InvalidPath)
                    }
                    _ => Err(UrlTypeError::Unknown),
                }
            } else {
                Err(UrlTypeError::WrongDomain)
            }
        } else {
            Err(UrlTypeError::InvalidUrl)
        }
    } else {
        Err(UrlTypeError::InvalidUrl)
    }
}
