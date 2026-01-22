//! YouTube Music API client.

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::{Auth, BrowserAuth, OAuthCredentials, OAuthState, OAuthToken};
use crate::context::{create_context, default_headers, YTM_BASE_API, YTM_PARAMS, YTM_PARAMS_KEY};
use crate::error::{Error, Result};
use crate::nav::nav;
use crate::parsers::{
    get_continuation_token, parse_library_playlists, parse_playlist_response, parse_playlist_tracks,
};
use crate::path;
use crate::types::{Playlist, PlaylistSummary, PlaylistTrack, Song};

/// The main YouTube Music API client.
///
/// Use [`YTMusicClient::builder()`] to create a new instance.
pub struct YTMusicClient {
    http: reqwest::Client,
    auth: Option<Auth>,
    language: String,
    location: Option<String>,
    user: Option<String>,
}

/// Builder for constructing a [`YTMusicClient`].
pub struct YTMusicClientBuilder {
    auth: Option<Auth>,
    language: String,
    location: Option<String>,
    user: Option<String>,
}

impl YTMusicClient {
    /// Create a new client builder.
    pub fn builder() -> YTMusicClientBuilder {
        YTMusicClientBuilder {
            auth: None,
            language: "en".to_string(),
            location: None,
            user: None,
        }
    }

    /// Check if the client is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_some()
    }

    /// Get all playlists from the user's library.
    ///
    /// This fetches "Your Likes", "Albums", and user-created playlists.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of playlists to return. `None` for all.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ytmusicapi::YTMusicClient;
    /// # async fn example(client: &YTMusicClient) -> ytmusicapi::Result<()> {
    /// let playlists = client.get_library_playlists(Some(10)).await?;
    /// for playlist in playlists {
    ///     println!("{}", playlist.title);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_library_playlists(&self, limit: Option<u32>) -> Result<Vec<PlaylistSummary>> {
        self.check_auth()?;

        let body = json!({
            "browseId": "FEmusic_liked_playlists"
        });

        let response = self.send_request("browse", body).await?;
        let mut playlists = parse_library_playlists(&response);

        // Handle pagination if needed
        if let Some(lim) = limit {
            playlists.truncate(lim as usize);
        }

        // TODO: Handle continuations for large libraries

        Ok(playlists)
    }

    /// Get a playlist with its tracks.
    ///
    /// Fetches metadata and tracks for a given playlist ID.
    /// Automatically handles pagination to fetch all tracks if requested.
    ///
    /// # Arguments
    ///
    /// * `playlist_id` - The playlist ID (can be with or without `VL` prefix).
    /// * `limit` - Maximum number of tracks to return. `None` for all (up to ~5000).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ytmusicapi::YTMusicClient;
    /// # async fn example(client: &YTMusicClient) -> ytmusicapi::Result<()> {
    /// let playlist = client.get_playlist("PL123456789", None).await?;
    /// println!("Title: {}", playlist.title);
    /// for track in playlist.tracks {
    ///     println!(" - {} by {:?}", track.title.unwrap_or_default(), track.artists);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_playlist(&self, playlist_id: &str, limit: Option<u32>) -> Result<Playlist> {
        // Ensure playlist ID has VL prefix for browse endpoint
        let browse_id = if playlist_id.starts_with("VL") {
            playlist_id.to_string()
        } else {
            format!("VL{}", playlist_id)
        };

        let body = json!({
            "browseId": browse_id
        });

        let response = self.send_request("browse", body.clone()).await?;
        let mut playlist = parse_playlist_response(&response, playlist_id);

        // Handle pagination for tracks
        let track_limit = limit.unwrap_or(5000) as usize;

        // Get continuation token if present and we need more tracks
        let secondary_contents = nav(
            &response,
            &path![
                "contents",
                "twoColumnBrowseResultsRenderer",
                "secondaryContents",
                "sectionListRenderer",
                "contents",
                0,
                "musicPlaylistShelfRenderer"
            ],
        );

        if let Some(shelf) = secondary_contents {
            if playlist.tracks.len() < track_limit {
                if let Some(token) = get_continuation_token(shelf) {
                    let more_tracks = self
                        .fetch_playlist_continuations(&token, track_limit - playlist.tracks.len())
                        .await?;
                    playlist.tracks.extend(more_tracks);
                }
            }
        }

        // Apply limit
        if let Some(lim) = limit {
            playlist.tracks.truncate(lim as usize);
        }

        // Recalculate duration
        playlist.duration_seconds = Some(
            playlist
                .tracks
                .iter()
                .filter_map(|t| t.duration_seconds)
                .sum(),
        );

        Ok(playlist)
    }

    /// Get the "Liked Songs" playlist.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of tracks to return. `None` for all.
    pub async fn get_liked_songs(&self, limit: Option<u32>) -> Result<Playlist> {
        self.check_auth()?;
        self.get_playlist("LM", limit).await
    }

    /// Get song metadata (including genre/category).
    pub async fn get_song(&self, video_id: &str) -> Result<Song> {
        let body = json!({
            "video_id": video_id,
            "playbackContext": {
                "contentPlaybackContext": {
                    "signatureTimestamp": 0 // We might need a real timestamp for streaming, but 0 often works for metadata
                }
            }
        });

        let response = self.send_request("player", body).await?;
        let song: Song = serde_json::from_value(response)?;
        Ok(song)
    }

    /// Fetch additional tracks via continuation token.
    async fn fetch_playlist_continuations(
        &self,
        initial_token: &str,
        max_items: usize,
    ) -> Result<Vec<PlaylistTrack>> {
        let mut all_tracks = Vec::new();
        let mut token = Some(initial_token.to_string());

        while let Some(current_token) = token {
            if all_tracks.len() >= max_items {
                break;
            }

            let body = json!({
                "continuation": current_token
            });

            let response = self.send_request("browse", body).await?;

            // Parse continuation response
            let continuation_items = nav(
                &response,
                &path![
                    "continuationContents",
                    "musicPlaylistShelfContinuation",
                    "contents"
                ],
            )
            .or_else(|| {
                nav(
                    &response,
                    &path![
                        "onResponseReceivedActions",
                        0,
                        "appendContinuationItemsAction",
                        "continuationItems"
                    ],
                )
            });

            if let Some(Value::Array(items)) = continuation_items {
                let tracks = parse_playlist_tracks(items);
                if tracks.is_empty() {
                    break;
                }
                all_tracks.extend(tracks);

                // Check for next continuation
                let next_token = items.last().and_then(|last| {
                    nav(
                        last,
                        &path![
                            "continuationItemRenderer",
                            "continuationEndpoint",
                            "continuationCommand",
                            "token"
                        ],
                    )
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                });

                token = next_token;
            } else {
                break;
            }
        }

        all_tracks.truncate(max_items);
        Ok(all_tracks)
    }

    /// Send a request to the YouTube Music API.
    pub async fn send_request(&self, endpoint: &str, mut body: Value) -> Result<Value> {
        // Merge context into body
        let context = create_context(
            &self.language,
            self.location.as_deref(),
            self.user.as_deref(),
        );
        if let Value::Object(ref mut map) = body {
            if let Value::Object(ctx) = context {
                for (k, v) in ctx {
                    map.insert(k, v);
                }
            }
        }

        // Build URL
        let params = match &self.auth {
            Some(Auth::Browser(_)) => format!("{}{}", YTM_PARAMS, YTM_PARAMS_KEY),
            _ => YTM_PARAMS.to_string(),
        };
        let url = format!("{}{}{}", YTM_BASE_API, endpoint, params);

        // Build request
        let mut request = self.http.post(&url).json(&body);

        // Add auth headers if authenticated
        match &self.auth {
            Some(Auth::Browser(auth)) => {
                // Combine user cookies with required SOCS cookie
                let combined_cookie = format!("{}; SOCS=CAI", auth.cookie);
                request = request
                    .header("authorization", auth.get_authorization()?)
                    .header("cookie", combined_cookie)
                    .header("x-goog-authuser", &auth.x_goog_authuser);
            }
            Some(Auth::OAuth(oauth_state)) => {
                let token = oauth_state.ensure_access_token().await?;
                let scheme = token.token_type.as_deref().unwrap_or("Bearer");
                let request_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                request = request
                    .header(
                        "authorization",
                        format!("{} {}", scheme, token.access_token),
                    )
                    .header("x-goog-request-time", request_time.to_string())
                    .header("cookie", "SOCS=CAI");
            }
            None => {
                // Add only SOCS cookie for unauthenticated requests
                request = request.header("cookie", "SOCS=CAI");
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Server {
                status,
                message: text,
            });
        }

        let json: Value = response.json().await?;

        // Check for API error in response
        if let Some(error) = json.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            let code = error.get("code").and_then(|c| c.as_u64()).unwrap_or(500) as u16;
            return Err(Error::Server {
                status: code,
                message,
            });
        }

        Ok(json)
    }

    /// Check that the client is authenticated, returning an error if not.
    fn check_auth(&self) -> Result<()> {
        if self.auth.is_none() {
            Err(Error::AuthRequired)
        } else {
            Ok(())
        }
    }
}

impl YTMusicClientBuilder {
    /// Set browser authentication.
    pub fn with_browser_auth(mut self, auth: BrowserAuth) -> Self {
        self.auth = Some(Auth::Browser(auth));
        self
    }

    /// Set OAuth authentication using an existing token.
    ///
    /// To enable automatic refresh, use [`with_oauth_token_and_credentials`].
    pub fn with_oauth_token(mut self, token: OAuthToken) -> Self {
        self.auth = Some(Auth::OAuth(OAuthState::new(token, None)));
        self
    }

    /// Set OAuth authentication with refresh support.
    pub fn with_oauth_token_and_credentials(
        mut self,
        token: OAuthToken,
        credentials: OAuthCredentials,
    ) -> Self {
        self.auth = Some(Auth::OAuth(OAuthState::new(token, Some(credentials))));
        self
    }

    /// Set the language for responses.
    ///
    /// Default is "en" (English).
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set the location for results.
    ///
    /// Use ISO 3166-1 alpha-2 country codes (e.g., "US", "GB", "DE").
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set a user ID for brand account requests.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<YTMusicClient> {
        let mut headers = HeaderMap::new();

        for (key, value) in default_headers() {
            if let Ok(header_value) = HeaderValue::from_str(&value) {
                if let Ok(header_name) = key.parse::<HeaderName>() {
                    headers.insert(header_name, header_value);
                }
            }
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .gzip(true)
            .build()?;

        Ok(YTMusicClient {
            http,
            auth: self.auth,
            language: self.language,
            location: self.location,
            user: self.user,
        })
    }
}
