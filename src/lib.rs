//! YouTube Music API client for Rust
//!
//! A Rust port of the Python `ytmusicapi` library, focused on playlist reading.
//!
//! # Example
//!
//! ```no_run
//! use ytmusicapi::{YTMusicClient, BrowserAuth};
//!
//! #[tokio::main]
//! async fn main() -> ytmusicapi::Result<()> {
//!     let auth = BrowserAuth::from_file("headers.json")?;
//!     let client = YTMusicClient::builder()
//!         .with_browser_auth(auth)
//!         .build()?;
//!
//!     // Get all playlists
//!     let playlists = client.get_library_playlists(None).await?;
//!     for pl in &playlists {
//!         println!("{}: {}", pl.playlist_id, pl.title);
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
