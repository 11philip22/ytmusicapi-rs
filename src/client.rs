//! YouTube Music API client.

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Value, json};

use crate::auth::BrowserAuth;
use crate::context::{YTM_BASE_API, YTM_PARAMS, YTM_PARAMS_KEY, create_context, default_headers};
use crate::error::{Error, Result};
use crate::nav::nav;
use crate::parsers::{
    get_continuation_token, parse_library_playlists, parse_playlist_response, parse_playlist_tracks,
};
use crate::types::{
    CreatePlaylistResponse, LikeStatus, MovePlaylistItemsResult, Playlist, PlaylistSummary,
    PlaylistTrack, Privacy, Song,
};

fn validate_id<'a>(name: &str, value: &'a str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::InvalidInput(format!(
            "{name} must include at least one character"
        )));
    }
    Ok(value)
}

fn validate_playlist_id(playlist_id: &str) -> Result<&str> {
    let playlist_id = validate_id("playlist_id", playlist_id)?;
    Ok(playlist_id.strip_prefix("VL").unwrap_or(playlist_id))
}

fn validate_video_id(video_id: &str) -> Result<&str> {
    validate_id("video_id", video_id)
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
        if let Some((_set_video_id, video_id)) = playlist_item_ids(item) {
            video_ids.push(video_id.to_string());
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

fn playlist_item_ids(item: &PlaylistTrack) -> Option<(&str, &str)> {
    let set_video_id = item.set_video_id.as_deref()?.trim();
    let video_id = item.video_id.as_deref()?.trim();
    if set_video_id.is_empty() || video_id.is_empty() {
        return None;
    }
    Some((set_video_id, video_id))
}

fn song_request_body(video_id: &str) -> Result<Value> {
    let video_id = validate_video_id(video_id)?;
    Ok(json!({
        "videoId": video_id,
        "playbackContext": {
            "contentPlaybackContext": {
                "signatureTimestamp": 0
            }
        }
    }))
}

fn rating_request_body(video_id: &str) -> Result<Value> {
    let video_id = validate_video_id(video_id)?;
    Ok(json!({
        "target": {
            "videoId": video_id
        }
    }))
}

fn add_playlist_items_body(
    playlist_id: &str,
    video_ids: &[String],
    allow_duplicates: bool,
) -> Result<Value> {
    let playlist_id = validate_playlist_id(playlist_id)?;
    if video_ids.is_empty() {
        return Err(Error::InvalidInput(
            "video_ids must include at least one item".to_string(),
        ));
    }

    let mut actions = Vec::new();
    for video_id in video_ids {
        let video_id = validate_video_id(video_id)?;
        let mut action = json!({
            "action": "ACTION_ADD_VIDEO",
            "addedVideoId": video_id
        });
        if !allow_duplicates {
            action["dedupeOption"] = json!("DEDUPE_OPTION_SKIP");
        }
        actions.push(action);
    }

    Ok(json!({
        "playlistId": playlist_id,
        "actions": actions
    }))
}

fn remove_playlist_items_body(playlist_id: &str, items: &[PlaylistTrack]) -> Result<Value> {
    let playlist_id = validate_playlist_id(playlist_id)?;
    let mut actions = Vec::new();
    for item in items {
        if let Some((set_video_id, video_id)) = playlist_item_ids(item) {
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

    Ok(json!({
        "playlistId": playlist_id,
        "actions": actions
    }))
}

/// The main YouTube Music API client.
///
/// Construct with [`YTMusicClient::builder()`]. Methods that require
/// authentication return [`Error::AuthRequired`](crate::Error::AuthRequired) if
/// no [`BrowserAuth`] is configured.
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
    ///
    /// Defaults:
    /// - language: `"en"`
    /// - location: `None`
    /// - user: `None`
    pub fn builder() -> YTMusicClientBuilder {
        YTMusicClientBuilder {
            auth: None,
            language: "en".to_string(),
            location: None,
            user: None,
        }
    }

    /// Check whether browser authentication is configured.
    ///
    /// This does not validate the cookie or perform a network request.
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_some()
    }

    /// Get playlists from the user's library.
    ///
    /// Requires authentication. This currently fetches only the first page of
    /// playlists returned by the web client and does not follow continuations.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of playlists to return. `None` returns the
    ///   entire first page.
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
    /// Fetches metadata and tracks for a given playlist ID. The client does not
    /// enforce authentication, but private playlists may be rejected by the API.
    /// If `limit` is `None`, the client follows continuations and returns up to
    /// 5,000 tracks.
    ///
    /// # Arguments
    ///
    /// * `playlist_id` - The playlist ID (can be with or without `VL` prefix).
    /// * `limit` - Maximum number of tracks to return. `None` for all (capped at 5,000).
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
        let playlist_id = validate_id("playlist_id", playlist_id)?;
        // Ensure playlist ID has VL prefix for browse endpoint
        let browse_id = if playlist_id.starts_with("VL") {
            playlist_id.to_string()
        } else {
            format!("VL{}", playlist_id)
        };

        let body = json!({
            "browseId": browse_id
        });

        let response = self.send_request("browse", body).await?;
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

        if let Some(shelf) = secondary_contents
            && playlist.tracks.len() < track_limit
            && let Some(token) = get_continuation_token(shelf)
        {
            let more_tracks = self
                .fetch_playlist_continuations(&token, track_limit - playlist.tracks.len())
                .await?;
            playlist.tracks.extend(more_tracks);
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
    /// Requires authentication.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of tracks to return. `None` for all.
    pub async fn get_liked_songs(&self, limit: Option<u32>) -> Result<Playlist> {
        self.check_auth()?;
        self.get_playlist("LM", limit).await
    }

    /// Create a new playlist.
    ///
    /// Requires authentication. An empty `description` is omitted from the request.
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

        if let Some(desc) = description
            && !desc.trim().is_empty()
        {
            body["description"] = json!(desc);
        }

        let response = self.send_request("playlist/create", body).await?;
        let created: CreatePlaylistResponse = serde_json::from_value(response)?;
        Ok(created)
    }

    /// Delete a playlist.
    ///
    /// Requires authentication. The ID may be provided with or without the `VL` prefix.
    pub async fn delete_playlist(&self, playlist_id: &str) -> Result<()> {
        self.check_auth()?;

        let body = json!({
            "playlistId": validate_playlist_id(playlist_id)?
        });

        self.send_request("playlist/delete", body).await?;
        Ok(())
    }

    /// Get song metadata from the `player` endpoint.
    ///
    /// This does not require authentication and does not return stream URLs.
    pub async fn get_song(&self, video_id: &str) -> Result<Song> {
        let response = self
            .send_request("player", song_request_body(video_id)?)
            .await?;
        let song: Song = serde_json::from_value(response)?;
        Ok(song)
    }

    /// Rate a song (like/dislike/indifferent).
    ///
    /// Requires authentication. Returns the raw API response.
    pub async fn rate_song(&self, video_id: &str, rating: LikeStatus) -> Result<Value> {
        self.check_auth()?;
        self.send_request(rating.endpoint(), rating_request_body(video_id)?)
            .await
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
    ///
    /// Requires authentication. When `allow_duplicates` is `false`, the request
    /// includes `DEDUPE_OPTION_SKIP`, which instructs the API to skip videos that
    /// are already present in the playlist.
    pub async fn add_playlist_items(
        &self,
        playlist_id: &str,
        video_ids: &[String],
        allow_duplicates: bool,
    ) -> Result<Value> {
        self.check_auth()?;
        self.send_request(
            "browse/edit_playlist",
            add_playlist_items_body(playlist_id, video_ids, allow_duplicates)?,
        )
        .await
    }

    /// Remove items from a playlist using playlist track metadata.
    ///
    /// Requires authentication. Only items with both `video_id` and `set_video_id`
    /// are removed; if none qualify, this returns [`Error::InvalidInput`].
    pub async fn remove_playlist_items(
        &self,
        playlist_id: &str,
        items: &[PlaylistTrack],
    ) -> Result<Value> {
        self.check_auth()?;
        self.send_request(
            "browse/edit_playlist",
            remove_playlist_items_body(playlist_id, items)?,
        )
        .await
    }

    /// Move items from one playlist to another (add to destination, then remove from source).
    ///
    /// Requires authentication. If the add succeeds but the remove fails, the
    /// destination playlist is not rolled back.
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
                message: format!("Failed to add items to destination playlist: {}", status),
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
    ///
    /// This is a low-level helper that merges a client context into `body`,
    /// performs a `POST`, and returns the raw JSON response.
    ///
    /// Error behavior:
    /// - Surfaces network failures as [`Error::Http`](crate::Error::Http).
    /// - Surfaces non-2xx responses or error payloads as [`Error::Server`](crate::Error::Server).
    /// - Surfaces JSON decode failures as [`Error::Json`](crate::Error::Json).
    ///
    /// This crate does not configure timeouts, retries, or polling; any timeout
    /// behavior comes from the underlying HTTP client defaults.
    pub async fn send_request(&self, endpoint: &str, mut body: Value) -> Result<Value> {
        // Merge context into body
        let context = create_context(
            &self.language,
            self.location.as_deref(),
            self.user.as_deref(),
        );
        if let Value::Object(ref mut map) = body
            && let Value::Object(ctx) = context
        {
            for (k, v) in ctx {
                map.insert(k, v);
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
    /// This maps to the `hl` client parameter (default: `"en"`).
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set the location for results.
    ///
    /// This maps to the `gl` client parameter and expects ISO 3166-1 alpha-2
    /// country codes (e.g., `"US"`, `"GB"`, `"DE"`).
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set a user ID for brand account requests.
    ///
    /// This maps to `onBehalfOfUser` in the request context.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Build the client.
    ///
    /// This does not validate authentication credentials.
    pub fn build(self) -> Result<YTMusicClient> {
        let mut headers = HeaderMap::new();

        for (key, value) in default_headers() {
            if let Ok(header_value) = HeaderValue::from_str(&value)
                && let Ok(header_name) = key.parse::<HeaderName>()
            {
                headers.insert(header_name, header_value);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn track(video_id: Option<&str>, set_video_id: Option<&str>) -> PlaylistTrack {
        PlaylistTrack {
            video_id: video_id.map(String::from),
            set_video_id: set_video_id.map(String::from),
            ..Default::default()
        }
    }

    #[test]
    fn song_body_uses_video_id_key() {
        let body = song_request_body(" abc ").unwrap();
        assert_eq!(body["videoId"], "abc");
        assert!(body.get("video_id").is_none());
        assert!(matches!(
            song_request_body(" "),
            Err(Error::InvalidInput(_))
        ));
    }

    #[test]
    fn rating_body_validates_video_id() {
        let body = rating_request_body("abc").unwrap();
        assert_eq!(body["target"]["videoId"], "abc");
        assert!(matches!(
            rating_request_body(""),
            Err(Error::InvalidInput(_))
        ));
    }

    #[test]
    fn add_playlist_items_honors_allow_duplicates() {
        let video_ids = vec!["abc".to_string()];

        let allow = add_playlist_items_body("VLPL123", &video_ids, true).unwrap();
        assert_eq!(allow["playlistId"], "PL123");
        assert!(allow["actions"][0].get("dedupeOption").is_none());

        let skip = add_playlist_items_body("PL123", &video_ids, false).unwrap();
        assert_eq!(skip["actions"][0]["dedupeOption"], "DEDUPE_OPTION_SKIP");
    }

    #[test]
    fn add_playlist_items_validates_ids() {
        assert!(matches!(
            add_playlist_items_body("", &["abc".to_string()], true),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            add_playlist_items_body("PL123", &[], true),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            add_playlist_items_body("PL123", &[" ".to_string()], true),
            Err(Error::InvalidInput(_))
        ));
    }

    #[test]
    fn remove_playlist_items_ignores_invalid_metadata() {
        let items = vec![
            track(Some(" "), Some("set1")),
            track(Some("vid1"), Some(" set1 ")),
        ];

        let body = remove_playlist_items_body(" VLPL123 ", &items).unwrap();
        assert_eq!(body["playlistId"], "PL123");
        assert_eq!(body["actions"].as_array().unwrap().len(), 1);
        assert_eq!(body["actions"][0]["removedVideoId"], "vid1");
        assert_eq!(body["actions"][0]["setVideoId"], "set1");
    }

    #[test]
    fn remove_playlist_items_requires_one_valid_item() {
        assert!(matches!(
            remove_playlist_items_body("PL123", &[track(Some(" "), Some("set1"))]),
            Err(Error::InvalidInput(_))
        ));
    }
}
