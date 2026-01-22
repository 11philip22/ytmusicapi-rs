//! Authentication helpers (browser cookies and OAuth).
//!
//! This module handles authentication using browser cookies or OAuth device flow tokens.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::error::{Error, Result};

/// Browser-based authentication using cookies from a YouTube Music session.
///
/// This authentication method (mimicking a browser) is required for most
/// authenticated operations like accessing your library or managing playlists.
///
/// # Obtaining Credentials
///
/// 1. Open [YouTube Music](https://music.youtube.com) in your browser and log in.
/// 2. Open Developer Tools (F12) and go to the **Network** tab.
/// 3. Filter for `browse` requests.
/// 4. Reload the page if needed, and select a `browse` request (e.g. `browse?...`).
/// 5. In the **Request Headers** section, find:
///    - `Cookie`: The full cookie string.
///    - `x-goog-authuser`: The auth user index (usually `0`).
///
/// # Usage
///
/// You can save these credentials to a JSON file (recommended) or create `BrowserAuth` directly.
///
/// **headers.json example:**
/// ```json
/// {
///     "cookie": "SID=...; __Secure-3PAPISID=...; ...",
///     "x-goog-authuser": "0"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAuth {
    /// The full cookie string from the browser
    pub cookie: String,

    /// The x-goog-authuser header value (usually "0")
    #[serde(alias = "x-goog-authuser")]
    pub x_goog_authuser: String,

    /// Optional origin header
    #[serde(default = "default_origin")]
    pub origin: String,
}

fn default_origin() -> String {
    "https://music.youtube.com".to_string()
}

impl BrowserAuth {
    /// Create BrowserAuth from a headers JSON file.
    ///
    /// The file should contain a JSON object with at least `cookie` and `x-goog-authuser` keys.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Create BrowserAuth from a JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        let parsed: serde_json::Value = serde_json::from_str(json)?;

        // Handle case-insensitive header names
        let cookie = parsed
            .get("cookie")
            .or_else(|| parsed.get("Cookie"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidAuth("missing 'cookie' field".to_string()))?
            .to_string();

        let x_goog_authuser = parsed
            .get("x-goog-authuser")
            .or_else(|| parsed.get("X-Goog-Authuser"))
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();

        let origin = parsed
            .get("origin")
            .or_else(|| parsed.get("Origin"))
            .or_else(|| parsed.get("x-origin"))
            .or_else(|| parsed.get("X-Origin"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://music.youtube.com")
            .to_string();

        Ok(Self {
            cookie,
            x_goog_authuser,
            origin,
        })
    }

    /// Extract the SAPISID from the cookie string.
    pub fn sapisid(&self) -> Result<String> {
        // Parse cookies to find __Secure-3PAPISID
        for part in self.cookie.split(';') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix("__Secure-3PAPISID=") {
                return Ok(value.to_string());
            }
        }
        Err(Error::InvalidAuth(
            "cookie missing __Secure-3PAPISID".to_string(),
        ))
    }

    /// Generate the SAPISIDHASH authorization header.
    ///
    /// This is a time-based hash that YouTube uses for authentication.
    pub fn get_authorization(&self) -> Result<String> {
        let sapisid = self.sapisid()?;
        let timestamp = unix_timestamp();

        let auth_string = format!("{} {} {}", timestamp, sapisid, self.origin);

        let mut hasher = Sha1::new();
        hasher.update(auth_string.as_bytes());
        let hash = hasher.finalize();

        Ok(format!("SAPISIDHASH {}_{:x}", timestamp, hash))
    }
}

/// OAuth scope requested by the YouTube Music device flow.
const OAUTH_SCOPE: &str = "https://www.googleapis.com/auth/youtube";

/// OAuth device code URL.
const OAUTH_CODE_URL: &str = "https://www.youtube.com/o/oauth2/device/code";

/// OAuth token exchange URL.
const OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// OAuth user-agent matching the Python library.
const OAUTH_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0 Cobalt/Version";

/// Authentication strategies supported by the client.
#[derive(Debug)]
pub enum Auth {
    /// Browser cookie authentication (SAPISID hash).
    Browser(BrowserAuth),
    /// OAuth device-flow authentication (Bearer token).
    OAuth(OAuthState),
}

/// Device-code response returned by Google's OAuth device flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    #[serde(default)]
    pub interval: Option<u64>,
}

/// OAuth token representation (access + optional refresh).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OAuthToken {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
    /// Optional path to write refreshed tokens back to disk.
    #[serde(skip)]
    pub cache_path: Option<PathBuf>,
}

impl OAuthToken {
    /// Create an OAuth token from a JSON file (usually `oauth.json`).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let mut token = Self::from_json(&content)?;
        token.cache_path = Some(path.as_ref().to_path_buf());
        Ok(token)
    }

    /// Create an OAuth token from a JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        let mut token: OAuthToken = serde_json::from_str(json)?;
        token.ensure_expires_at();
        Ok(token)
    }

    /// Persist the token to a file (without the cache path metadata).
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut copy = self.clone();
        copy.cache_path = None;
        let content = serde_json::to_string_pretty(&copy)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Compute expires_at when only expires_in is present.
    fn ensure_expires_at(&mut self) {
        if self.expires_at.is_none() {
            if let Some(expires_in) = self.expires_in {
                self.expires_at = Some(unix_timestamp() + expires_in);
            }
        }
    }

    /// Whether the token is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            unix_timestamp() >= expires_at
        } else {
            false
        }
    }

    /// Whether the token should be refreshed soon (<= 60 seconds).
    pub fn is_expiring(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            unix_timestamp() + 60 >= expires_at
        } else {
            false
        }
    }

    /// Update token fields from a freshly fetched token.
    pub fn update_from(&mut self, mut fresh: OAuthToken) {
        // Preserve existing refresh token if the server does not return one on refresh.
        if fresh.refresh_token.is_none() {
            fresh.refresh_token = self.refresh_token.clone();
        }

        // Always keep cache path to allow persistence.
        fresh.cache_path = self.cache_path.clone();
        fresh.ensure_expires_at();
        *self = fresh;
    }
}

/// OAuth client credentials.
#[derive(Debug, Clone)]
pub struct OAuthCredentials {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
}

impl OAuthCredentials {
    /// Create new OAuth credentials using the YouTube Data API client id/secret.
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(OAUTH_USER_AGENT)
            .build()?;

        Ok(Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            http,
        })
    }

    /// Request a device/user code pair for the OAuth device flow.
    pub async fn request_device_code(&self) -> Result<DeviceCode> {
        let response = self
            .http
            .post(OAUTH_CODE_URL)
            .form(&[
                ("client_id", self.client_id.clone()),
                ("scope", OAUTH_SCOPE.to_string()),
            ])
            .send()
            .await?;

        parse_oauth_response(response).await
    }

    /// Exchange the device code for a full OAuth token.
    pub async fn exchange_device_code(&self, device_code: &str) -> Result<OAuthToken> {
        let response = self
            .http
            .post(OAUTH_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.clone()),
                ("client_secret", self.client_secret.clone()),
                (
                    "grant_type",
                    "http://oauth.net/grant_type/device/1.0".to_string(),
                ),
                ("code", device_code.to_string()),
            ])
            .send()
            .await?;

        parse_oauth_response(response).await
    }

    /// Refresh an existing OAuth token.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken> {
        let response = self
            .http
            .post(OAUTH_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.clone()),
                ("client_secret", self.client_secret.clone()),
                ("grant_type", "refresh_token".to_string()),
                ("refresh_token", refresh_token.to_string()),
            ])
            .send()
            .await?;

        parse_oauth_response(response).await
    }
}

/// Shared OAuth state (token + optional refresh credentials).
#[derive(Debug)]
pub struct OAuthState {
    inner: Mutex<OAuthInner>,
}

#[derive(Debug)]
struct OAuthInner {
    token: OAuthToken,
    credentials: Option<OAuthCredentials>,
}

impl OAuthState {
    /// Create a new OAuth state holder.
    pub fn new(token: OAuthToken, credentials: Option<OAuthCredentials>) -> Self {
        Self {
            inner: Mutex::new(OAuthInner { token, credentials }),
        }
    }

    /// Ensure the access token is fresh (refreshes if needed) and return a clone.
    pub async fn ensure_access_token(&self) -> Result<OAuthToken> {
        // First, decide whether a refresh is needed without holding the lock across await.
        let refresh_inputs = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| Error::InvalidAuth("OAuth state poisoned".to_string()))?;

            guard.token.ensure_expires_at();

            let should_refresh = guard.credentials.is_some()
                && guard.token.refresh_token.is_some()
                && (guard.token.is_expiring() || guard.token.is_expired());

            if should_refresh {
                Some((
                    guard.credentials.clone().unwrap(),
                    guard.token.refresh_token.clone().unwrap(),
                    guard.token.cache_path.clone(),
                ))
            } else {
                None
            }
        };

        if let Some((credentials, refresh_token, cache_path)) = refresh_inputs {
            let mut fresh = credentials.refresh_token(&refresh_token).await?;
            fresh.cache_path = cache_path;

            let mut guard = self
                .inner
                .lock()
                .map_err(|_| Error::InvalidAuth("OAuth state poisoned".to_string()))?;
            guard.token.update_from(fresh);
            guard.persist_to_disk()?;
            return Ok(guard.token.clone());
        }

        // No refresh needed.
        let guard = self
            .inner
            .lock()
            .map_err(|_| Error::InvalidAuth("OAuth state poisoned".to_string()))?;
        if guard.token.is_expired() {
            return Err(Error::InvalidAuth(
                "OAuth token expired and no refresh credentials were provided".to_string(),
            ));
        }
        Ok(guard.token.clone())
    }
}

impl OAuthInner {
    fn persist_to_disk(&self) -> Result<()> {
        if let Some(path) = &self.token.cache_path {
            self.token.to_file(path)?;
        }
        Ok(())
    }
}

/// Parse an OAuth response (token/device code).
async fn parse_oauth_response<T>(response: reqwest::Response) -> Result<T>
where
    T: DeserializeOwned + 'static,
{
    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(Error::InvalidAuth(format!(
            "OAuth request failed ({}): {}",
            status.as_u16(),
            text
        )));
    }

    let mut parsed: T = serde_json::from_str(&text)?;

    if let Some(token) = downcast_token_mut(&mut parsed) {
        token.ensure_expires_at();
    }

    Ok(parsed)
}

/// Downcast helper to call ensure_expires_at on OAuthToken if the parsed type matches.
fn downcast_token_mut<T: 'static>(value: &mut T) -> Option<&mut OAuthToken> {
    use std::any::Any;
    (value as &mut dyn Any).downcast_mut::<OAuthToken>()
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_sapisid() {
        let auth = BrowserAuth {
            cookie: "other=value; __Secure-3PAPISID=abc123def; more=stuff".to_string(),
            x_goog_authuser: "0".to_string(),
            origin: "https://music.youtube.com".to_string(),
        };

        assert_eq!(auth.sapisid().unwrap(), "abc123def");
    }

    #[test]
    fn test_from_json() {
        let json = r#"{"cookie": "test=1; __Secure-3PAPISID=xyz", "x-goog-authuser": "0"}"#;
        let auth = BrowserAuth::from_json(json).unwrap();
        assert_eq!(auth.sapisid().unwrap(), "xyz");
    }

    #[test]
    fn test_oauth_token_from_file_roundtrip() {
        let temp_dir =
            std::env::temp_dir().join(format!("ytmusicapi_oauth_test_{}", unix_timestamp()));
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("oauth.json");

        let json = r#"{
            "access_token": "abc",
            "refresh_token": "def",
            "expires_in": 100,
            "token_type": "Bearer"
        }"#;
        fs::write(&path, json).unwrap();

        let token = OAuthToken::from_file(&path).unwrap();
        assert_eq!(token.access_token, "abc");
        assert_eq!(token.refresh_token.as_deref(), Some("def"));
        assert!(token.expires_at.is_some());
        assert_eq!(token.cache_path.as_deref(), Some(path.as_path()));

        token.to_file(&path).unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("access_token"));
        assert!(!saved.contains("cache_path"));

        fs::remove_file(&path).ok();
        fs::remove_dir_all(&temp_dir).ok();
    }
}
