//! # ytmusicapi: A Rust client for the YouTube Music API
//!
//! This library provides a Rust interface to the YouTube Music API (YouTubei),
//! ported from the popular Python [ytmusicapi](https://github.com/sigma67/ytmusicapi) library.
//!
//! ## Features
//!
//! - **Browsing**: Fetch playlists, library tracks, and playlist details.
//! - **Authentication**: Browser-cookie or OAuth-based authentication (required for library access).
//! - **Robust**: Handles various API response formats (premium vs free, grid vs list).
//!
//! *Note: Currently focused on **read-only** operations (reading playlists/library).*
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ytmusicapi = "0.1.0"
//! ```
//!
//! ## Authentication
//!
//! Two options are supported:
//! - **Browser cookies** (matches the Python library default):
//!   1. Open [YouTube Music](https://music.youtube.com).
//!   2. Open Developer Tools (F12) > Network tab.
//!   3. Copy request headers from a `browse` call and save as `headers.json`:
//!      ```json
//!      {"cookie": "YOUR_COOKIE_STRING", "x-goog-authuser": "0"}
//!      ```
//! - **OAuth device flow**:
//!   1. Create a YouTube Data API OAuth client (TVs and Limited Input devices).
//!   2. Use `OAuthCredentials::request_device_code` to get a `user_code` and `verification_url`.
//!   3. After completing the browser flow, call `exchange_device_code` and save the token JSON.
//!
//! ## Example (browser cookies)
//!
//! ```no_run
//! use ytmusicapi::{YTMusicClient, BrowserAuth};
//!
//! # #[tokio::main]
//! # async fn main() -> ytmusicapi::Result<()> {
//! let auth = BrowserAuth::from_file("headers.json")?;
//! let client = YTMusicClient::builder()
//!     .with_browser_auth(auth)
//!     .build()?;
//!
//! let playlists = client.get_library_playlists(None).await?;
//! for pl in playlists {
//!     println!("{} ({})", pl.title, pl.count.unwrap_or(0));
//! }
//! # Ok(())
//! # }
//! ```
mod auth;
mod client;
mod context;
mod error;
mod nav;
mod parsers;
mod types;

pub use auth::{Auth, BrowserAuth, DeviceCode, OAuthCredentials, OAuthToken};
pub use client::{YTMusicClient, YTMusicClientBuilder};
pub use error::{Error, Result};
pub use types::*;
