//! Browser cookie authentication.
//!
//! This module handles authentication using cookies extracted from a browser session.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::error::{Error, Result};

/// Browser-based authentication using cookies from a YouTube Music session.
///
/// This authentication method (mimicking a browser) is required for most
/// authenticated operations like accessing your library or managing playlists.
/// The cookie string must include `__Secure-3PAPISID`, which is used to compute
/// the `SAPISIDHASH` authorization header.
///
/// # Obtaining Credentials
///
/// 1. Open [YouTube Music](https://music.youtube.com) in your browser and sign in.
/// 2. Open Developer Tools (F12) and go to the **Network** tab.
/// 3. Filter for `browse` requests.
/// 4. Reload the page if needed, and select a `browse` request (e.g. `browse?...`).
/// 5. In the **Request Headers** section, copy:
///    - `cookie`: The full cookie string.
///    - `x-goog-authuser`: The auth user index (usually `0`).
///
/// # Usage
///
/// You can save these credentials to a JSON file (recommended) or construct
/// `BrowserAuth` directly.
///
/// **headers.json example:**
/// ```json
/// {
///     "cookie": "SID=...; __Secure-3PAPISID=...; ...",
///     "x-goog-authuser": "0"
/// }
/// ```
///
/// Keep this file private: it grants access to your account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAuth {
    /// The full `cookie` header value from the browser.
    pub cookie: String,

    /// The `x-goog-authuser` header value (usually `"0"`).
    #[serde(alias = "x-goog-authuser")]
    pub x_goog_authuser: String,

    /// Origin used when computing the authorization hash.
    #[serde(default = "default_origin")]
    pub origin: String,
}

fn default_origin() -> String {
    "https://music.youtube.com".to_string()
}

impl BrowserAuth {
    /// Create `BrowserAuth` from a headers JSON file.
    ///
    /// The file should contain a JSON object with at least a `cookie` key.
    /// Header names are matched case-insensitively, and `x-goog-authuser`
    /// defaults to `"0"` if omitted.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Create `BrowserAuth` from a JSON string.
    ///
    /// Accepts `cookie`, `x-goog-authuser`, and `origin` (case-insensitive).
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

    /// Extract `__Secure-3PAPISID` from the cookie string.
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

    /// Generate the `SAPISIDHASH` authorization header.
    ///
    /// This is a time-based hash that YouTube uses for browser authentication.
    pub fn get_authorization(&self) -> Result<String> {
        let sapisid = self.sapisid()?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let auth_string = format!("{} {} {}", timestamp, sapisid, self.origin);

        let mut hasher = Sha1::new();
        hasher.update(auth_string.as_bytes());
        let hash = hasher.finalize();

        Ok(format!("SAPISIDHASH {}_{:x}", timestamp, hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
