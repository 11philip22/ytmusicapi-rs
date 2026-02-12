//! YouTube Music API client.

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};

use crate::auth::BrowserAuth;
use crate::context::{create_context, default_headers, YTM_BASE_API, YTM_PARAMS, YTM_PARAMS_KEY};
use crate::error::{Error, Result};
use crate::nav::nav;
use crate::parsers::{
    get_continuation_token, parse_library_playlists, parse_playlist_response, parse_playlist_tracks,
};
use crate::path;
use crate::types::{
    CreatePlaylistResponse, LikeStatus, MovePlaylistItemsResult, Playlist, PlaylistSummary,
    PlaylistTrack, Privacy, Song,
};

fn validate_playlist_id(playlist_id: &str) -> &str {
    playlist_id.strip_prefix("VL").unwrap_or(playlist_id)
}

fn status_succeeded(response: &Value) -> bool {
    response
        .get("status")
        .and_then(|v| v.as_str())
        .map(|s| s.contains("SUCCEEDED"))
        .unwrap_or(false)
}

fn collect_movable_items(items: &[PlaylistTrack]) -> Result<(Vec<String>, Vec<PlaylistTrack>)> {
    let mut video_ids = Vec::new();
    let mut removable = Vec::new();

    for item in items {
        if let (Some(video_id), Some(_set_video_id)) = (&item.video_id, &item.set_video_id) {
            video_ids.push(video_id.clone());
            removable.push(item.clone());
        }
    }

    if video_ids.is_empty() {
        return Err(Error::InvalidInput(
            "No playlist items include both video_id and set_video_id".to_string(),
        ));
    }

    Ok((video_ids, removable))
}

/// The main YouTube Music API client.
///
/// Use [`YTMusicClient::builder()`] to create a new instance.
pub struct YTMusicClient {
    http: reqwest::Client,
    auth: Option<BrowserAuth>,
    language: String,
    location: Option<String>,
    user: Option<String>,
}

/// Builder for constructing a [`YTMusicClient`].
pub struct YTMusicClientBuilder {
    auth: Option<BrowserAuth>,
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

    /// Create a new playlist.
    pub async fn create_playlist(
        &self,
        title: &str,
        description: Option<&str>,
        privacy: Privacy,
    ) -> Result<CreatePlaylistResponse> {
        self.check_auth()?;
        if title.trim().is_empty() {
            return Err(Error::InvalidInput(
                "title must include at least one character".to_string(),
            ));
        }

        let privacy_status = match privacy {
            Privacy::Public => "PUBLIC",
            Privacy::Private => "PRIVATE",
            Privacy::Unlisted => "UNLISTED",
        };

        let mut body = json!({
            "title": title,
            "privacyStatus": privacy_status
        });

        if let Some(desc) = description {
            if !desc.trim().is_empty() {
                body["description"] = json!(desc);
            }
        }

        let response = self.send_request("playlist/create", body).await?;
        let created: CreatePlaylistResponse = serde_json::from_value(response)?;
        Ok(created)
    }

    /// Delete a playlist.
    pub async fn delete_playlist(&self, playlist_id: &str) -> Result<()> {
        self.check_auth()?;
        if playlist_id.trim().is_empty() {
            return Err(Error::InvalidInput(
                "playlist_id must include at least one character".to_string(),
            ));
        }

        let body = json!({
            "playlistId": validate_playlist_id(playlist_id)
        });

        self.send_request("playlist/delete", body).await?;
        Ok(())
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

    /// Rate a song (like/dislike/indifferent).
    pub async fn rate_song(&self, video_id: &str, rating: LikeStatus) -> Result<Value> {
        self.check_auth()?;

        let body = json!({
            "target": {
                "videoId": video_id
            }
        });

        self.send_request(rating.endpoint(), body).await
    }

    /// Like a song.
    pub async fn like_song(&self, video_id: &str) -> Result<Value> {
        self.rate_song(video_id, LikeStatus::Like).await
    }

    /// Remove like/dislike from a song.
    pub async fn unlike_song(&self, video_id: &str) -> Result<Value> {
        self.rate_song(video_id, LikeStatus::Indifferent).await
    }

    /// Add items to a playlist by video ID.
    pub async fn add_playlist_items(
        &self,
        playlist_id: &str,
        video_ids: &[String],
        allow_duplicates: bool,
    ) -> Result<Value> {
        self.check_auth()?;
        if video_ids.is_empty() {
            return Err(Error::InvalidInput(
                "video_ids must include at least one item".to_string(),
            ));
        }

        let mut actions = Vec::new();
        for video_id in video_ids {
            let mut action = json!({
                "action": "ACTION_ADD_VIDEO",
                "addedVideoId": video_id
            });
            if allow_duplicates {
                action["dedupeOption"] = json!("DEDUPE_OPTION_SKIP");
            }
            actions.push(action);
        }

        let body = json!({
            "playlistId": validate_playlist_id(playlist_id),
            "actions": actions
        });

        self.send_request("browse/edit_playlist", body).await
    }

    /// Remove items from a playlist using playlist track metadata.
    pub async fn remove_playlist_items(
        &self,
        playlist_id: &str,
        items: &[PlaylistTrack],
    ) -> Result<Value> {
        self.check_auth()?;

        let mut actions = Vec::new();
        for item in items {
            if let (Some(set_video_id), Some(video_id)) = (&item.set_video_id, &item.video_id) {
                actions.push(json!({
                    "action": "ACTION_REMOVE_VIDEO",
                    "setVideoId": set_video_id,
                    "removedVideoId": video_id
                }));
            }
        }

        if actions.is_empty() {
            return Err(Error::InvalidInput(
                "No playlist items include both video_id and set_video_id".to_string(),
            ));
        }

        let body = json!({
            "playlistId": validate_playlist_id(playlist_id),
            "actions": actions
        });

        self.send_request("browse/edit_playlist", body).await
    }

    /// Move items from one playlist to another (add to destination, then remove from source).
    pub async fn move_playlist_items(
        &self,
        from_playlist_id: &str,
        to_playlist_id: &str,
        items: &[PlaylistTrack],
        allow_duplicates: bool,
    ) -> Result<MovePlaylistItemsResult> {
        self.check_auth()?;
        let (video_ids, removable_items) = collect_movable_items(items)?;

        let add_response = self
            .add_playlist_items(to_playlist_id, &video_ids, allow_duplicates)
            .await?;
        if !status_succeeded(&add_response) {
            let status = add_response
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown status");
            return Err(Error::Server {
                status: 500,
                message: format!(
                    "Failed to add items to destination playlist: {}",
                    status
                ),
            });
        }

        let remove_response = self
            .remove_playlist_items(from_playlist_id, &removable_items)
            .await?;

        Ok(MovePlaylistItemsResult {
            add_response,
            remove_response,
        })
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
        let params = if self.auth.is_some() {
            format!("{}{}", YTM_PARAMS, YTM_PARAMS_KEY)
        } else {
            YTM_PARAMS.to_string()
        };
        let url = format!("{}{}{}", YTM_BASE_API, endpoint, params);

        // Build request
        let mut request = self.http.post(&url).json(&body);

        // Add auth headers if authenticated
        if let Some(ref auth) = self.auth {
            // Combine user cookies with required SOCS cookie
            let combined_cookie = format!("{}; SOCS=CAI", auth.cookie);
            request = request
                .header("authorization", auth.get_authorization()?)
                .header("cookie", combined_cookie)
                .header("x-goog-authuser", &auth.x_goog_authuser);
        } else {
            // Add only SOCS cookie for unauthenticated requests
            request = request.header("cookie", "SOCS=CAI");
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
        self.auth = Some(auth);
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
