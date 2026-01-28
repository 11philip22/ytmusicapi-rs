//! # ytmusicapi: A Rust client for the YouTube Music API
//!
//! This library provides a Rust interface to the YouTube Music API (YouTubei),
//! ported from the popular Python [ytmusicapi](https://github.com/sigma67/ytmusicapi) library.
//!
//! ## Features
//!
//! - **Browsing**: Fetch playlists, library tracks, and playlist details.
//! - **Authentication**: Browser-cookie based authentication (required for library access).
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
//! This library uses browser cookies for authentication.
//!
//! 1. Open [YouTube Music](https://music.youtube.com).
//! 2. Open Developer Tools (F12) > Network tab.
//! 3. Right-click a request (e.g., `browse`) -> Copy -> Copy Request Headers.
//! 4. Create a `headers.json` file:
//!
//! ```json
//! {
//!     "cookie": "YOUR_COOKIE_STRING",
//!     "x-goog-authuser": "0"
//! }
//! ```
//!
//! ## Example
//!
//! ```no_run
//! use ytmusicapi::{YTMusicClient, BrowserAuth};
//!
//! #[tokio::main]
//! async fn main() -> ytmusicapi::Result<()> {
//!     // Load auth from file
//!     let auth = BrowserAuth::from_file("headers.json")?;
//!     
//!     // Create client
//!     let client = YTMusicClient::builder()
//!         .with_browser_auth(auth)
//!         .build()?;
//!
//!     // List library playlists
//!     let playlists = client.get_library_playlists(None).await?;
//!     for pl in playlists {
//!         println!("{} ({})", pl.title, pl.count.unwrap_or(0));
//!     }
//!
//!     Ok(())
//! }
//! ```
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
