<p align="center">
  <img src="assets/hero-banner.png" alt="hero pane" width="980">
</p>

<p align="center">
  <a href="https://crates.io/crates/ytmusicapi"><img src="https://img.shields.io/badge/crates.io-ytmusicapi-F59E0B?style=for-the-badge&logo=rust&logoColor=white" alt="Crates.io"></a>
  <a href="https://docs.rs/ytmusicapi"><img src="https://img.shields.io/badge/docs.rs-ytmusicapi-3B82F6?style=for-the-badge&logo=readthedocs&logoColor=white" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-8B5CF6?style=for-the-badge" alt="MIT License"></a>
  <a href="https://github.com/woldp001/ytmusicapi-rs/pulls"><img src="https://img.shields.io/badge/PRs-Welcome-22C55E?style=for-the-badge" alt="PRs Welcome"></a>
</p>

<p align="center">
  <a href="#features">Features</a> · <a href="#installation">Installation</a> · <a href="#quick-start">Quick Start</a> · <a href="#api-reference">API Reference</a> · <a href="#examples">Examples</a> · <a href="#contributing">Contributing</a> · <a href="#support">Support</a> · <a href="#license">License</a>
</p>

A Rust library for the YouTube Music API.

---

> [!NOTE]
> 🚧 **Work in Progress**: search, uploads, and some library management features are not yet supported.

## Features

- **Browser cookie authentication**
- **Playlist APIs**: List library playlists, get playlist tracks
- **Playlist edits**: Add/remove/move tracks between playlists
- **Likes**: Like and unlike songs
- **Pagination**: Automatic handling of large playlists
- **Idiomatic Rust**: Builder pattern, strong typing, async/await

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
| `builder()` | Create a `YTMusicClientBuilder` with defaults |
| `is_authenticated()` | Check whether browser auth is configured |
| `get_library_playlists(limit)` | Get all playlists from your library |
| `get_playlist(id, limit)` | Get a playlist with its tracks |
| `get_liked_songs(limit)` | Get your liked songs playlist |
| `create_playlist(title, description, privacy)` | Create a new playlist |
| `delete_playlist(id)` | Delete a playlist |
| `get_song(video_id)` | Get song metadata from the `player` endpoint |
| `add_playlist_items(id, video_ids, allow_duplicates)` | Add videos to a playlist |
| `remove_playlist_items(id, tracks)` | Remove playlist items (requires `set_video_id`) |
| `move_playlist_items(from, to, tracks, allow_duplicates)` | Move items between playlists |
| `rate_song(video_id, rating)` | Like/dislike/clear rating for a song |
| `like_song(video_id)` | Like a song |
| `unlike_song(video_id)` | Remove like/dislike from a song |
| `send_request(endpoint, body)` | Low-level API helper that sends a raw request and returns JSON |

### `YTMusicClientBuilder`

| Method | Description |
|--------|-------------|
| `with_browser_auth(auth)` | Configure browser-cookie authentication |
| `with_language(language)` | Set request language (default: `en`) |
| `with_location(location)` | Set location hint |
| `with_user(user)` | Set user profile index |
| `build()` | Build and validate a `YTMusicClient` instance |

### Types

- `Playlist` - Full playlist with metadata and tracks
- `PlaylistSummary` - Brief playlist info (for library listing)
- `PlaylistTrack` - Track within a playlist
- `CreatePlaylistResponse` - Response with newly created playlist ID
- `MovePlaylistItemsResult` - Responses from moving items between playlists
- `Privacy` - Playlist privacy enum (`PUBLIC`, `PRIVATE`, `UNLISTED`)
- `LikeStatus` - Rating enum (`LIKE`, `DISLIKE`, `INDIFFERENT`)
- `Song` - Song metadata from `get_song`
- `VideoDetails`, `Microformat`, `MicroformatDataRenderer` - Song metadata subtypes
- `Artist`, `Album`, `Author`, `Thumbnail` - Common types

## Examples

Run the examples:

```bash
cargo run --example list_playlists
cargo run --example create_playlist -- \
  --title "My Playlist" \
  --description "Created via ytmusicapi-rs" \
  --privacy private
cargo run --example add_song_to_playlist -- \
  --playlist-id PLAYLIST_ID \
  --video-id VIDEO_ID \
  [--allow-duplicates]
cargo run --example get_liked_songs -- [--limit 50]
cargo run --example like_song -- \
  --video-id VIDEO_ID
cargo run --example unlike_song -- \
  --video-id VIDEO_ID
cargo run --example delete_playlist -- \
  --playlist-id PLAYLIST_ID
cargo run --example remove_playlist_items -- \
  --playlist-id PLAYLIST_ID \
  --video-ids VIDEO_ID_1,VIDEO_ID_2
cargo run --example move_playlist_items -- \
  --source PLAYLIST_ID \
  --dest PLAYLIST_ID \
  --video-ids VIDEO_ID_1,VIDEO_ID_2 \
  [--allow-duplicates]
cargo run --example playlist_items -- \
  --source PLAYLIST_ID \
  --dest PLAYLIST_ID \
  --video-ids VIDEO_ID_1,VIDEO_ID_2 \
  [--allow-duplicates]
```

## Acknowledgements

This library is a Rust port of [ytmusicapi](https://github.com/sigma67/ytmusicapi).

## Contributing

PRs are welcome!  
Please run `cargo fmt` and `cargo clippy` before submitting.

If you’re changing behavior (e.g. stricter parsing), document it in the PR.

## Support

If this crate saves you time or helps your work, support is appreciated:

[![Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/11philip22)

## License

This project is licensed under the MIT License; see the [license](https://opensource.org/licenses/MIT) for details.
