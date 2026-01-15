# ytmusicapi-rs

[![Crates.io](https://img.shields.io/crates/v/ytmusicapi.svg)](https://crates.io/crates/ytmusicapi)
[![Documentation](https://docs.rs/ytmusicapi/badge.svg)](https://docs.rs/ytmusicapi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

<img src="https://upload.wikimedia.org/wikipedia/commons/1/1c/YouTube_Music_2024.svg" alt="ytmusicapi" width="300">

A Rust library for the YouTube Music API.

> [!NOTE]
> 🚧 **Work in Progress**: currently implementing only **playlist reading** features. Search, library management, and uploads are not yet supported.

## Features

- 🔐 **Browser cookie authentication**
- 📋 **Playlist APIs**: List library playlists, get playlist tracks
- ❤️ **Liked Songs**: Access your liked songs playlist
- 📄 **Pagination**: Automatic handling of large playlists
- 🦀 **Idiomatic Rust**: Builder pattern, strong typing, async/await

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ytmusicapi = { path = "path/to/ytmusicapi-rs" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

### 1. Get Your Browser Headers

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

### 2. Use the Library

```rust
use ytmusicapi::{BrowserAuth, YTMusicClient};

#[tokio::main]
async fn main() -> ytmusicapi::Result<()> {
    let auth = BrowserAuth::from_file("headers.json")?;
    
    let client = YTMusicClient::builder()
        .with_browser_auth(auth)
        .build()?;

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

Run the example:

```bash
cargo run --example list_playlists
```

## Acknowledgements

This library is a Rust port of [ytmusicapi](https://github.com/sigma67/ytmusicapi).

## License

MIT License - see [license](license) for details.
