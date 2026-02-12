//! # ytmusicapi
//!
//! An async Rust client for the YouTube Music internal API (YouTubei), focused on playlist and
//! library workflows. This is **not** an official Google API and may change as the web client
//! evolves.
//!
//! ## Supported Operations
//!
//! - Read library playlists: [`YTMusicClient::get_library_playlists`]
//! - Fetch playlist metadata and tracks: [`YTMusicClient::get_playlist`]
//! - Fetch your "Liked Songs": [`YTMusicClient::get_liked_songs`]
//! - Create/delete playlists: [`YTMusicClient::create_playlist`], [`YTMusicClient::delete_playlist`]
//! - Add/remove/move playlist items: [`YTMusicClient::add_playlist_items`],
//!   [`YTMusicClient::remove_playlist_items`], [`YTMusicClient::move_playlist_items`]
//! - Rate songs: [`YTMusicClient::rate_song`], [`YTMusicClient::like_song`],
//!   [`YTMusicClient::unlike_song`]
//! - Fetch song metadata (no auth required): [`YTMusicClient::get_song`]
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ytmusicapi = "0.4"
//! ```
//!
//! ## Authentication
//!
//! Authenticated requests use browser cookies. The cookie string **must** include
//! `__Secure-3PAPISID`, which is used to compute the `SAPISIDHASH` authorization header.
//!
//! 1. Open [YouTube Music](https://music.youtube.com) and sign in.
//! 2. Open Developer Tools (F12) and go to the **Network** tab.
//! 3. Filter for a `browse` request and select it.
//! 4. In **Request Headers**, copy:
//!    - `cookie` (the full cookie string)
//!    - `x-goog-authuser` (usually `0`)
//! 5. Save them as `headers.json`:
//!
//! ```json
//! {
//!     "cookie": "SID=...; __Secure-3PAPISID=...; ...",
//!     "x-goog-authuser": "0"
//! }
//! ```
//!
//! ## Quick Start
//!
//! ```no_run
//! use ytmusicapi::{BrowserAuth, YTMusicClient};
//!
//! #[tokio::main]
//! async fn main() -> ytmusicapi::Result<()> {
//!     let auth = BrowserAuth::from_file("headers.json")?;
//!     let client = YTMusicClient::builder()
//!         .with_browser_auth(auth)
//!         .build()?;
//!
//!     let playlists = client.get_library_playlists(Some(10)).await?;
//!     for playlist in playlists {
//!         println!("{} ({})", playlist.title, playlist.count.unwrap_or(0));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Unauthenticated Metadata
//!
//! ```no_run
//! use ytmusicapi::YTMusicClient;
//!
//! #[tokio::main]
//! async fn main() -> ytmusicapi::Result<()> {
//!     let client = YTMusicClient::builder().build()?;
//!     let song = client.get_song("dQw4w9WgXcQ").await?;
//!     println!("{} by {}", song.video_details.title, song.video_details.author);
//!     Ok(())
//! }
//! ```
//!
//! ## Error Behavior
//!
//! All fallible APIs return [`Result`](crate::Result), backed by [`Error`](crate::Error).
//!
//! - Authentication-required methods return [`Error::AuthRequired`](crate::Error::AuthRequired)
//!   when no [`BrowserAuth`](crate::BrowserAuth) is configured.
//! - HTTP and network failures surface as [`Error::Http`](crate::Error::Http).
//! - Non-2xx responses or API error payloads surface as
//!   [`Error::Server`](crate::Error::Server).
//! - Response decode failures surface as [`Error::Json`](crate::Error::Json).
//! - Input validation failures surface as [`Error::InvalidInput`](crate::Error::InvalidInput).
//! - Credential parsing failures surface as [`Error::InvalidAuth`](crate::Error::InvalidAuth).
//!
//! **Timeouts, retries, and polling:** this crate does not configure request
//! timeouts, retry failed requests, or poll for completion. Any timeouts are
//! determined by the underlying HTTP client defaults and the network stack.
//!
//! **External system failures:** because this client depends on the YouTube Music
//! web API, changes or outages on Google's side can cause `Error::Server` or
//! parsing errors. The API is unofficial and may change without notice.
mod auth;
mod client;
mod context;
mod error;
mod nav;
mod parsers;
mod types;

pub use auth::BrowserAuth;
pub use client::{YTMusicClient, YTMusicClientBuilder};
pub use error::{Error, Result};
pub use types::*;
