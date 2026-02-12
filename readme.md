# ytmusicapi-rs

[![Crates.io](https://img.shields.io/crates/v/ytmusicapi.svg)](https://crates.io/crates/ytmusicapi)
[![Documentation](https://docs.rs/ytmusicapi/badge.svg)](https://docs.rs/ytmusicapi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/woldp001/guerrillamail-client-rs/pulls)

<img src="https://upload.wikimedia.org/wikipedia/commons/1/1c/YouTube_Music_2024.svg" alt="ytmusicapi" width="300">

A Rust library for the YouTube Music API.

> [!NOTE]
> 🚧 **Work in Progress**: search, uploads, and some library management features are not yet supported.

## Features

- 🔐 **Browser cookie authentication**
- 📋 **Playlist APIs**: List library playlists, get playlist tracks
- ✏️ **Playlist edits**: Add/remove/move tracks between playlists
- ❤️ **Likes**: Like and unlike songs
- 📄 **Pagination**: Automatic handling of large playlists
- 🦀 **Idiomatic Rust**: Builder pattern, strong typing, async/await

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ytmusicapi = { version = "0.3.0" }
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

    // Like a song
    client.like_song("VIDEO_ID").await?;

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
| `create_playlist(title, description, privacy)` | Create a new playlist |
| `delete_playlist(id)` | Delete a playlist |
| `add_playlist_items(id, video_ids, allow_duplicates)` | Add videos to a playlist |
| `remove_playlist_items(id, tracks)` | Remove playlist items (requires `set_video_id`) |
| `move_playlist_items(from, to, tracks, allow_duplicates)` | Move items between playlists |
| `rate_song(video_id, rating)` | Like/dislike/clear rating for a song |
| `like_song(video_id)` | Like a song |
| `unlike_song(video_id)` | Remove like/dislike from a song |

### Types

- `Playlist` - Full playlist with metadata and tracks
- `PlaylistSummary` - Brief playlist info (for library listing)
- `PlaylistTrack` - Track within a playlist
- `LikeStatus` - Rating enum (`LIKE`, `DISLIKE`, `INDIFFERENT`)
- `Artist`, `Album`, `Thumbnail` - Common types

## Examples

Run the examples:

```bash
cargo run --example list_playlists
cargo run --example create_playlist -- \
  --title "My Playlist" \
  --description "Created via ytmusicapi-rs" \
  --privacy private
cargo run --example delete_playlist -- \
  --playlist-id PLAYLIST_ID
cargo run --example playlist_items -- \
  --source PLAYLIST_ID \
  --dest PLAYLIST_ID \
  --video-ids VIDEO_ID_1,VIDEO_ID_2 \
  [--allow-duplicates]
```

## Acknowledgements

This library is a Rust port of [ytmusicapi](https://github.com/sigma67/ytmusicapi).

## License

MIT License - see [license](license) for details.
