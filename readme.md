# ytmusicapi-rs

[![Crates.io](https://img.shields.io/crates/v/ytmusicapi.svg)](https://crates.io/crates/ytmusicapi)
[![Documentation](https://docs.rs/ytmusicapi/badge.svg)](https://docs.rs/ytmusicapi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

<img src="https://upload.wikimedia.org/wikipedia/commons/1/1c/YouTube_Music_2024.svg" alt="ytmusicapi" width="300">

A Rust library for the YouTube Music API.

> [!NOTE]
> 🚧 **Work in Progress**: currently implementing only **playlist reading** features. Search, library management, and uploads are not yet supported.

## Features

- 🔐 **Authentication**: Browser cookies or OAuth device flow
- 📋 **Playlist APIs**: List library playlists, get playlist tracks
- ❤️ **Liked Songs**: Access your liked songs playlist
- 📄 **Pagination**: Automatic handling of large playlists
- 🦀 **Idiomatic Rust**: Builder pattern, strong typing, async/await

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ytmusicapi = { version = "0.1.0" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

### Option A: Browser headers

1. Open [YouTube Music](https://music.youtube.com) in your browser and log in
2. Open Developer Tools (F12) → Network tab
3. Find any request to `music.youtube.com`
4. Copy the `cookie` and `x-goog-authuser` headers
5. Save as `headers.json`:

```json
{
  "cookie": "__Secure-3PAPISID=...; other_cookies...",
  "x-goog-authuser": "0"
}
```

### Option B: OAuth device flow

1. Create a YouTube Data API OAuth client (type: **TVs and Limited Input devices**)
2. Run the device flow in code to obtain an `oauth.json` token file:

```rust
use ytmusicapi::{OAuthCredentials, OAuthToken};

# async fn setup_oauth() -> ytmusicapi::Result<()> {
let credentials = OAuthCredentials::new("CLIENT_ID", "CLIENT_SECRET")?;
let device = credentials.request_device_code().await?;
println!("Visit {} and enter {}", device.verification_url, device.user_code);
// Wait for the user to finish in the browser...
let token = credentials.exchange_device_code(&device.device_code).await?;
token.to_file("oauth.json")?;
# Ok(())
# }
```

You can reuse `oauth.json` later without repeating the flow.

### Use the Library

```rust
use ytmusicapi::{BrowserAuth, OAuthCredentials, OAuthToken, YTMusicClient};

#[tokio::main]
async fn main() -> ytmusicapi::Result<()> {
    // Browser cookies
    let auth = BrowserAuth::from_file("headers.json")?;
    
    let client = YTMusicClient::builder()
        .with_browser_auth(auth)
        .build()?;

    // Or OAuth (uncomment to use)
    // let token = OAuthToken::from_file("oauth.json")?;
    // let creds = OAuthCredentials::new("CLIENT_ID", "CLIENT_SECRET")?;
    // let client = YTMusicClient::builder()
    //     .with_oauth_token_and_credentials(token, creds)
    //     .build()?;

    // List all playlists
    let playlists = client.get_library_playlists(None).await?;
    for pl in &playlists {
        println!("{}: {}", pl.playlist_id, pl.title);
    }

    // Get a specific playlist with tracks
    let playlist = client.get_playlist("PLxxxxxx", None).await?;
    for track in &playlist.tracks {
        println!("{} - {}", 
            track.artists.first().map(|a| a.name.as_str()).unwrap_or("Unknown"),
            track.title.as_deref().unwrap_or("Unknown"));
    }

    // Get liked songs
    let liked = client.get_liked_songs(Some(50)).await?;
    println!("You have {} liked songs", liked.tracks.len());

    Ok(())
}
```

## API Reference

### `YTMusicClient`

| Method | Description |
|--------|-------------|
| `get_library_playlists(limit)` | Get all playlists from your library |
| `get_playlist(id, limit)` | Get a playlist with its tracks |
| `get_liked_songs(limit)` | Get your liked songs playlist |

### Types

- `Playlist` - Full playlist with metadata and tracks
- `PlaylistSummary` - Brief playlist info (for library listing)
- `PlaylistTrack` - Track within a playlist
- `Artist`, `Album`, `Thumbnail` - Common types

## Examples

- List playlists with browser auth:
  ```bash
  cargo run --example list_playlists
  ```
- OAuth device flow (requires `CLIENT_ID`/`CLIENT_SECRET` env vars):
  ```bash
  CLIENT_ID=... CLIENT_SECRET=... cargo run --example oauth_device_flow
  ```

## TODO / Missing Parity

- Auth parity: accept fully formed OAuth headers/custom clients; visitor-id bootstrap; mobile context toggle
- Pagination/continuations: finish library playlist paging and general continuation handling for new endpoints
- Search + suggestions: search API plus suggestion fetch/remove
- Library reads/actions: library songs/albums/artists, subscriptions/channels, history add/remove, ratings, account info
- Playlists: create/edit/delete, add/remove items, rate playlists
- Browse metadata: home feed, artist/album/user/channel/podcast info, related songs, lyrics (incl. timed), taste profile get/set
- Explore/Charts and watch/queue (`get_watch_playlist`)
- Uploads: list upload songs/albums/artists, upload song, delete upload entity
- Podcasts: channel/podcast/episode APIs, saved episodes

## Acknowledgements

This library is a Rust port of [ytmusicapi](https://github.com/sigma67/ytmusicapi).

## License

MIT License - see [license](license) for details.
