use crate::{
    qobuz_models::{
        album::{Album, AlbumSearchResults},
        album_suggestion::AlbumSuggestionResponse,
        artist::{Artists, ArtistsResponse},
        artist_page::ArtistPage,
        favorites::Favorites,
        featured::{FeaturedAlbumsResponse, FeaturedPlaylistsResponse},
        playlist::{Playlist, UserPlaylistsResult},
        release::{Release, ReleaseQuery},
        search_results::SearchAllResults,
        track::Track,
        TrackURL,
    },
    Error, Result,
};
use base64::{engine::general_purpose, Engine as _};
use clap::ValueEnum;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Method, Response, StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct Client {
    active_secret: String,
    app_id: String,
    base_url: String,
    http_client: reqwest::Client,
    user_token: String,
    user_id: i64,
    max_audio_quality: AudioQuality,
}

#[derive(Default, Clone, Debug, ValueEnum)]
pub enum AudioQuality {
    Mp3,
    CD,
    HIFI96,
    #[default]
    HIFI192,
}
impl Display for AudioQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AudioQuality::Mp3 => "5",
            AudioQuality::CD => "6",
            AudioQuality::HIFI96 => "7",
            AudioQuality::HIFI192 => "27",
        })
    }
}

pub async fn new(
    username: &str,
    password: &str,
    max_audio_quality: AudioQuality,
) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
            "User-Agent",
            HeaderValue::from_str(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36",
            )
            .unwrap(),
        );

    let http_client = reqwest::Client::builder()
        .cookie_store(true)
        .default_headers(headers)
        .build()
        .unwrap();

    let Secrets { secrets, app_id } = get_secrets(&http_client).await?;

    tracing::debug!("Got login secrets");

    let base_url = "https://www.qobuz.com/api.json/0.2/".to_string();

    let login = login(username, password, &app_id, &base_url, &http_client).await?;
    tracing::debug!("Logged in");

    let active_secret =
        find_active_secret(secrets, &base_url, &http_client, &app_id, &login.user_token).await?;

    tracing::debug!("Found active secrets");

    let client = Client {
        http_client,
        active_secret,
        user_token: login.user_token,
        user_id: login.user_id,
        app_id,
        base_url,
        max_audio_quality,
    };

    Ok(client)
}

enum Endpoint {
    Album,
    ArtistPage,
    SimilarArtists,
    ArtistReleases,
    Login,
    UserPlaylist,
    SearchAlbums,
    Track,
    TrackURL,
    Playlist,
    PlaylistCreate,
    PlaylistDelete,
    PlaylistAddTracks,
    PlaylistDeleteTracks,
    PlaylistUpdatePosition,
    Search,
    Favorites,
    FavoriteAdd,
    FavoriteRemove,
    FavoritePlaylistAdd,
    FavoritePlaylistRemove,
    AlbumSuggest,
    AlbumFeatured,
    PlaylistFeatured,
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let endpoint = match self {
            Endpoint::Album => "album/get",
            Endpoint::ArtistPage => "artist/page",
            Endpoint::ArtistReleases => "artist/getReleasesList",
            Endpoint::SimilarArtists => "artist/getSimilarArtists",
            Endpoint::Login => "user/login",
            Endpoint::Playlist => "playlist/get",
            Endpoint::PlaylistCreate => "playlist/create",
            Endpoint::PlaylistDelete => "playlist/delete",
            Endpoint::PlaylistAddTracks => "playlist/addTracks",
            Endpoint::PlaylistDeleteTracks => "playlist/deleteTracks",
            Endpoint::PlaylistUpdatePosition => "playlist/updateTracksPosition",
            Endpoint::Search => "catalog/search",
            Endpoint::SearchAlbums => "album/search",
            Endpoint::Track => "track/get",
            Endpoint::TrackURL => "track/getFileUrl",
            Endpoint::UserPlaylist => "playlist/getUserPlaylists",
            Endpoint::Favorites => "favorite/getUserFavorites",
            Endpoint::FavoriteAdd => "favorite/create",
            Endpoint::FavoriteRemove => "favorite/delete",
            Endpoint::FavoritePlaylistAdd => "playlist/subscribe",
            Endpoint::FavoritePlaylistRemove => "playlist/unsubscribe",
            Endpoint::AlbumSuggest => "album/suggest",
            Endpoint::AlbumFeatured => "album/getFeatured",
            Endpoint::PlaylistFeatured => "playlist/getFeatured",
        };

        f.write_str(endpoint)
    }
}

macro_rules! get {
    ($self:ident, $endpoint:expr, $params:expr) => {
        match $self.make_get_call($endpoint, $params).await {
            Ok(response) => match serde_json::from_str(response.as_str()) {
                Ok(item) => Ok(item),
                Err(error) => Err(Error::DeserializeJSON {
                    message: error.to_string(),
                }),
            },
            Err(error) => Err(Error::Api {
                message: error.to_string(),
            }),
        }
    };
}

macro_rules! post {
    ($self:ident, $endpoint:expr, $form:expr) => {
        match $self.make_post_call($endpoint, $form).await {
            Ok(response) => match serde_json::from_str(response.as_str()) {
                Ok(item) => Ok(item),
                Err(error) => Err(Error::DeserializeJSON {
                    message: error.to_string(),
                }),
            },
            Err(error) => Err(Error::Api {
                message: error.to_string(),
            }),
        }
    };
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum AlbumFeaturedType {
    PressAwards,
    NewReleasesFull,
    Qobuzissims,
    IdealDiscography,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum PlaylistFeaturedType {
    EditorPicks,
}

impl Client {
    pub async fn featured_albums(
        &self,
        featured_type: AlbumFeaturedType,
    ) -> Result<FeaturedAlbumsResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::AlbumFeatured);

        let type_string = match featured_type {
            AlbumFeaturedType::PressAwards => "press-awards",
            AlbumFeaturedType::NewReleasesFull => "new-releases-full",
            AlbumFeaturedType::Qobuzissims => "qobuzissims",
            AlbumFeaturedType::IdealDiscography => "ideal-discography",
        };

        let params = vec![("type", type_string), ("offset", "0"), ("limit", "20")];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn featured_playlists(
        &self,
        featured_type: PlaylistFeaturedType,
    ) -> Result<FeaturedPlaylistsResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::PlaylistFeatured);

        let type_string = match featured_type {
            PlaylistFeaturedType::EditorPicks => "editor-picks",
        };

        let params = vec![("type", type_string), ("offset", "0"), ("limit", "20")];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn user_playlists(&self) -> Result<UserPlaylistsResult> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::UserPlaylist);
        let params = vec![("limit", "500"), ("extra", "tracks"), ("offset", "0")];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn playlist(&self, playlist_id: i64) -> Result<Playlist> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::Playlist);
        let id_string = playlist_id.to_string();
        let params = vec![
            ("limit", "500"),
            ("extra", "tracks"),
            ("playlist_id", id_string.as_str()),
            ("offset", "0"),
        ];
        let playlist: Result<Playlist> = get!(self, &endpoint, Some(&params));

        if let Ok(mut playlist) = playlist {
            self.playlist_items(&mut playlist, &endpoint).await?;

            Ok(playlist)
        } else {
            Err(Error::Api {
                message: "error fetching playlist".to_string(),
            })
        }
    }

    async fn playlist_items(&self, playlist: &mut Playlist, endpoint: &str) -> Result<()> {
        let total_tracks = playlist.tracks_count as usize;

        if let Some(tracks) = playlist.tracks.as_mut() {
            while tracks.items.len() < total_tracks {
                let id = playlist.id.to_string();
                let limit_string = (total_tracks - tracks.items.len()).to_string();
                let offset_string = tracks.items.len().to_string();

                let params = vec![
                    ("limit", limit_string.as_str()),
                    ("extra", "tracks"),
                    ("playlist_id", id.as_str()),
                    ("offset", offset_string.as_str()),
                ];

                let playlist: Result<Playlist> = get!(self, endpoint, Some(&params));

                match &playlist {
                    Ok(playlist) => {
                        tracing::debug!("appending tracks to playlist");
                        if let Some(new_tracks) = &playlist.tracks {
                            tracks.items.append(&mut new_tracks.clone().items);
                        }
                    }
                    Err(error) => tracing::error!("{}", error.to_string()),
                }
            }
        }

        Ok(())
    }

    pub async fn create_playlist(
        &self,
        name: String,
        is_public: bool,
        description: Option<String>,
        is_collaborative: Option<bool>,
    ) -> Result<Playlist> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::PlaylistCreate);

        let mut form_data = HashMap::new();
        form_data.insert("name", name.as_str());

        let is_collaborative = if !is_public || is_collaborative.is_none() {
            "false".to_string()
        } else if let Some(is_collaborative) = is_collaborative {
            is_collaborative.to_string()
        } else {
            "false".to_string()
        };

        form_data.insert("is_collaborative", is_collaborative.as_str());

        let is_public = is_public.to_string();
        form_data.insert("is_public", is_public.as_str());

        let description = if let Some(description) = description {
            description
        } else {
            "".to_string()
        };
        form_data.insert("description", description.as_str());

        post!(self, &endpoint, form_data)
    }

    pub async fn delete_playlist(&self, playlist_id: String) -> Result<SuccessfulResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::PlaylistDelete);

        let mut form_data = HashMap::new();
        form_data.insert("playlist_id", playlist_id.as_str());

        post!(self, &endpoint, form_data)
    }

    pub async fn playlist_add_track(
        &self,
        playlist_id: &str,
        track_ids: Vec<&str>,
    ) -> Result<Playlist> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::PlaylistAddTracks);

        let track_ids = track_ids.join(",");

        let mut form_data = HashMap::new();
        form_data.insert("playlist_id", playlist_id);
        form_data.insert("track_ids", track_ids.as_str());
        form_data.insert("no_duplicate", "true");

        post!(self, &endpoint, form_data)
    }

    pub async fn playlist_delete_track(
        &self,
        playlist_id: String,
        playlist_track_ids: Vec<String>,
    ) -> Result<Playlist> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::PlaylistDeleteTracks);

        let playlist_track_ids = playlist_track_ids.join(",");

        let mut form_data = HashMap::new();
        form_data.insert("playlist_id", playlist_id.as_str());
        form_data.insert("playlist_track_ids", playlist_track_ids.as_str());

        post!(self, &endpoint, form_data)
    }

    pub async fn update_playlist_track_position(
        &self,
        index: usize,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<Playlist> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::PlaylistUpdatePosition);

        let index = index.to_string();

        let mut form_data = HashMap::new();
        form_data.insert("playlist_id", playlist_id);
        form_data.insert("playlist_track_ids", track_id);
        form_data.insert("insert_before", index.as_str());

        post!(self, &endpoint, form_data)
    }

    pub async fn track_url(&self, track_id: u32) -> Result<String> {
        track_url(
            track_id,
            &self.active_secret,
            &self.base_url,
            &self.http_client,
            &self.app_id,
            &self.user_token,
            &self.max_audio_quality,
        )
        .await
        .map(|u| u.url)
    }

    pub async fn favorites(&self, limit: i32) -> Result<Favorites> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::Favorites);

        let limit = limit.to_string();
        let params = vec![("limit", limit.as_str())];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn add_favorite_album(&self, id: &str) -> Result<SuccessfulResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::FavoriteAdd);
        let mut form_data = HashMap::new();
        form_data.insert("album_ids", id);

        post!(self, &endpoint, form_data)
    }

    pub async fn remove_favorite_album(&self, id: &str) -> Result<SuccessfulResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::FavoriteRemove);
        let mut form_data = HashMap::new();
        form_data.insert("album_ids", id);

        post!(self, &endpoint, form_data)
    }

    pub async fn add_favorite_artist(&self, id: &str) -> Result<SuccessfulResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::FavoriteAdd);
        let mut form_data = HashMap::new();
        form_data.insert("artist_ids", id);

        post!(self, &endpoint, form_data)
    }

    pub async fn remove_favorite_artist(&self, id: &str) -> Result<SuccessfulResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::FavoriteRemove);
        let mut form_data = HashMap::new();
        form_data.insert("artist_ids", id);

        post!(self, &endpoint, form_data)
    }

    pub async fn add_favorite_playlist(&self, id: &str) -> Result<SuccessfulResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::FavoritePlaylistAdd);
        let mut form_data = HashMap::new();
        form_data.insert("playlist_id", id);

        post!(self, &endpoint, form_data)
    }

    pub async fn remove_favorite_playlist(&self, id: &str) -> Result<SuccessfulResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::FavoritePlaylistRemove);
        let mut form_data = HashMap::new();
        form_data.insert("playlist_id", id);

        post!(self, &endpoint, form_data)
    }

    pub async fn search_all(&self, query: &str, limit: i32) -> Result<SearchAllResults> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::Search);
        let limit = limit.to_string();
        let params = vec![("query", query), ("limit", &limit)];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn album(&self, album_id: &str) -> Result<Album> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::Album);
        let params = vec![
            ("album_id", album_id),
            ("extra", "track_ids"),
            ("offset", "0"),
            ("limit", "500"),
        ];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn track(&self, track_id: u32) -> Result<Track> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::Track);
        let track_id_string = track_id.to_string();
        let params = vec![("track_id", track_id_string.as_str())];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn suggested_albums(&self, album_id: &str) -> Result<AlbumSuggestionResponse> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::AlbumSuggest);
        let params = vec![("album_id", album_id)];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn search_albums(
        &self,
        query: &str,
        limit: Option<i32>,
    ) -> Result<AlbumSearchResults> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::SearchAlbums);
        let limit = limit.unwrap_or(100).to_string();
        let params = vec![("query", query), ("limit", limit.as_str())];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn artist(&self, artist_id: u32) -> Result<ArtistPage> {
        let app_id = &self.app_id;

        let endpoint = format!("{}{}", self.base_url, Endpoint::ArtistPage);

        let artistid_string = artist_id.to_string();

        let params = vec![
            ("artist_id", artistid_string.as_str()),
            ("app_id", app_id),
            ("sort", "relevant"),
        ];

        get!(self, &endpoint, Some(&params))
    }

    pub async fn similar_artists(&self, artist_id: u32, limit: Option<i32>) -> Result<Artists> {
        let limit = limit.unwrap_or(10).to_string();

        let endpoint = format!("{}{}", self.base_url, Endpoint::SimilarArtists);
        let artistid_string = artist_id.to_string();

        let params = vec![
            ("artist_id", artistid_string.as_str()),
            ("limit", &limit),
            ("offset", "0"),
        ];

        let response: Result<ArtistsResponse> = get!(self, &endpoint, Some(&params));

        response.map(|res| res.artists)
    }

    pub async fn artist_releases(
        &self,
        artist_id: u32,
        limit: Option<i32>,
    ) -> Result<Vec<Release>> {
        let endpoint = format!("{}{}", self.base_url, Endpoint::ArtistReleases);
        let limit = limit.unwrap_or(100).to_string();

        let artistid_string = artist_id.to_string();

        let params = vec![
            ("artist_id", artistid_string.as_str()),
            ("limit", &limit),
            ("release_type", "album"),
            ("sort", "release_date"),
            ("offset", "0"),
            ("track_size", "1"),
        ];

        let result: Result<ReleaseQuery> = get!(self, &endpoint, Some(&params));
        match result {
            Ok(res) => Ok(res.items),
            Err(err) => Err(err),
        }
    }

    pub fn get_user_id(&self) -> i64 {
        self.user_id
    }

    async fn make_get_call(
        &self,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<String> {
        make_get_call(
            endpoint,
            params,
            &self.http_client,
            &self.app_id,
            Some(&self.user_token),
        )
        .await
    }

    async fn make_post_call(&self, endpoint: &str, params: HashMap<&str, &str>) -> Result<String> {
        let headers = client_headers(&self.app_id, Some(&self.user_token));

        tracing::debug!("calling {} endpoint, with params {params:?}", endpoint);
        let response = self
            .http_client
            .request(Method::POST, endpoint)
            .headers(headers)
            .form(&params)
            .send()
            .await?;

        handle_response(response).await
    }
}

// Check the retrieved secrets to see which one works.
async fn find_active_secret(
    secrets: HashMap<String, String>,
    base_url: &str,
    client: &reqwest::Client,
    app_id: &str,
    user_token: &str,
) -> Result<String> {
    tracing::debug!("testing secrets: {secrets:?}");

    for (timezone, secret) in secrets.into_iter() {
        let response = track_url(
            64868955,
            &secret,
            base_url,
            client,
            app_id,
            user_token,
            &AudioQuality::Mp3,
        )
        .await;

        if response.is_ok() {
            tracing::debug!("found good secret: {}\t{}", timezone, secret);
            let secret_string = secret;

            return Ok(secret_string);
        }
    }

    Err(Error::ActiveSecret)
}

async fn track_url(
    track_id: u32,
    secret: &str,
    base_url: &str,
    client: &reqwest::Client,
    app_id: &str,
    user_token: &str,
    max_audio_quality: &AudioQuality,
) -> Result<TrackURL> {
    let endpoint = format!("{}{}", base_url, Endpoint::TrackURL);
    let now = format!("{}", chrono::Utc::now().timestamp());

    let sig = format!(
        "trackgetFileUrlformat_id{}intentstreamtrack_id{}{}{}",
        max_audio_quality, track_id, now, secret
    );

    println!("{sig}");
    let hashed_sig = format!("{:x}", md5::compute(sig.as_str()));

    let track_id = track_id.to_string();

    let quality_string = max_audio_quality.to_string();

    let params = vec![
        ("request_ts", now.as_str()),
        ("request_sig", hashed_sig.as_str()),
        ("track_id", track_id.as_str()),
        ("format_id", &quality_string),
        ("intent", "stream"),
    ];

    match make_get_call(&endpoint, Some(&params), client, app_id, Some(user_token)).await {
        Ok(response) => match serde_json::from_str(response.as_str()) {
            Ok(item) => Ok(item),
            Err(error) => Err(Error::DeserializeJSON {
                message: error.to_string(),
            }),
        },
        Err(error) => Err(Error::Api {
            message: error.to_string(),
        }),
    }
}

async fn handle_response(response: Response) -> Result<String> {
    if response.status() == StatusCode::OK {
        let res = response.text().await.unwrap();
        Ok(res)
    } else {
        Err(Error::Api {
            message: response.status().to_string(),
        })
    }
}

async fn make_get_call(
    endpoint: &str,
    params: Option<&[(&str, &str)]>,
    client: &reqwest::Client,
    app_id: &str,
    user_token: Option<&str>,
) -> Result<String> {
    let headers = client_headers(app_id, user_token);

    tracing::debug!("calling {} endpoint, with params {params:?}", endpoint);
    let request = client.request(Method::GET, endpoint).headers(headers);

    if let Some(p) = params {
        let response = request.query(&p).send().await?;
        handle_response(response).await
    } else {
        let response = request.send().await?;
        handle_response(response).await
    }
}

fn client_headers(app_id: &str, user_token: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();

    tracing::debug!("adding app_id to request headers: {}", app_id);
    headers.insert("X-App-Id", HeaderValue::from_str(app_id).unwrap());

    if let Some(token) = user_token {
        tracing::debug!("adding token to request headers: {}", token);
        headers.insert("X-User-Auth-Token", HeaderValue::from_str(token).unwrap());
    }

    headers.insert(
        "Access-Control-Request-Headers",
        HeaderValue::from_str("x-app-id,x-user-auth-token").unwrap(),
    );

    headers.insert(
        "Accept-Language",
        HeaderValue::from_str("en,en-US;q=0.8,ko;q=0.6,zh;q=0.4,zh-CN;q=0.2").unwrap(),
    );

    headers
}

struct LoginResult {
    user_token: String,
    user_id: i64,
}

async fn login(
    username: &str,
    password: &str,
    app_id: &str,
    base_url: &str,
    client: &reqwest::Client,
) -> Result<LoginResult> {
    let endpoint = format!("{}{}", base_url, Endpoint::Login);

    tracing::debug!(
        "logging in with email ({}) and password **HIDDEN** for app_id {}",
        username,
        app_id
    );

    let params = vec![
        ("email", username),
        ("password", password),
        ("app_id", app_id),
    ];

    match make_get_call(&endpoint, Some(&params), client, app_id, None).await {
        Ok(response) => {
            let json: Value = serde_json::from_str(response.as_str()).unwrap();
            tracing::info!("Successfully logged in");
            tracing::debug!("{}", json);
            let mut user_token = json["user_auth_token"].to_string();
            user_token = user_token[1..user_token.len() - 1].to_string();

            let user_id = json["user"]["id"].to_string().parse::<i64>().unwrap();

            Ok(LoginResult {
                user_token,
                user_id,
            })
        }
        Err(err) => {
            tracing::error!("error logging into qobuz: {}", err);
            Err(Error::Login)
        }
    }
}

struct Secrets {
    secrets: HashMap<String, String>,
    app_id: String,
}

// ported from https://github.com/vitiko98/qobuz-dl/blob/master/qobuz_dl/bundle.py
// Retrieve the app_id and generate the secrets needed to authenticate
async fn get_secrets(client: &reqwest::Client) -> Result<Secrets> {
    tracing::debug!("fetching login page");
    let play_url = "https://play.qobuz.com";
    let login_page = client.get(format!("{play_url}/login")).send().await?;

    let contents = login_page.text().await.unwrap();

    let bundle_regex = regex::Regex::new(
        r#"<script src="(/resources/\d+\.\d+\.\d+-[a-z0-9]\d{3}/bundle\.js)"></script>"#,
    )
    .unwrap();
    let app_id_regex = regex::Regex::new(
        r#"production:\{api:\{appId:"(?P<app_id>\d{9})",appSecret:"(?P<app_secret>\w{32})""#,
    )
    .unwrap();
    let seed_regex = regex::Regex::new(
        r#"[a-z]\.initialSeed\("(?P<seed>[\w=]+)",window\.utimezone\.(?P<timezone>[a-z]+)\)"#,
    )
    .unwrap();

    let mut secrets = HashMap::new();

    let app_id = if let Some(captures) = bundle_regex.captures(contents.as_str()) {
        let bundle_path = captures.get(1).map_or("", |m| m.as_str());
        let bundle_url = format!("{play_url}{bundle_path}");
        if let Ok(bundle_page) = client.get(bundle_url).send().await {
            if let Ok(bundle_contents) = bundle_page.text().await {
                if let Some(captures) = app_id_regex.captures(bundle_contents.as_str()) {
                    let found_app_id = captures
                        .name("app_id")
                        .map_or("".to_string(), |m| m.as_str().to_string());

                    let seed_data = seed_regex.captures_iter(bundle_contents.as_str());

                    seed_data.for_each(|s| {
                            let seed = s.name("seed").map_or("", |m| m.as_str()).to_string();
                            let mut timezone =
                                s.name("timezone").map_or("", |m| m.as_str()).to_string();
                            capitalize(timezone.as_mut_str());

                            let info_regex = format!(r#"name:"\w+/(?P<timezone>{}([a-z]?))",info:"(?P<info>[\w=]+)",extras:"(?P<extras>[\w=]+)""#, &timezone);
                            regex::Regex::new(info_regex.as_str())
                                .unwrap()
                                .captures_iter(bundle_contents.as_str())
                                .for_each(|c| {
                                    let timezone =
                                        c.name("timezone").map_or("", |m| m.as_str()).to_string();
                                    let info =
                                        c.name("info").map_or("", |m| m.as_str()).to_string();
                                    let extras =
                                        c.name("extras").map_or("", |m| m.as_str()).to_string();

                                    let chars = format!("{seed}{info}{extras}");

                                    let encoded_secret = chars[..chars.len() - 44].to_string();
                                    let decoded_secret = general_purpose::URL_SAFE
                                        .decode(encoded_secret)
                                        .expect("failed to decode base64 secret");
                                    let secret_utf8 = std::str::from_utf8(&decoded_secret)
                                        .expect("failed to convert base64 to string")
                                        .to_string();

                                    secrets.insert(timezone, secret_utf8);
                                });
                        });
                    found_app_id
                } else {
                    return Err(Error::AppID);
                }
            } else {
                return Err(Error::AppID);
            }
        } else {
            return Err(Error::AppID);
        }
    } else {
        return Err(Error::AppID);
    };

    Ok(Secrets { secrets, app_id })
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SuccessfulResponse {
    status: String,
}

fn capitalize(s: &mut str) {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
}
