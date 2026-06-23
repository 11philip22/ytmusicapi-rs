<h1 align="center">ytmusicapi</h1>

<p align="center">
  <strong>Unofficial async Rust client for YouTube Music playlist and library workflows.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/ytmusicapi"><img src="https://img.shields.io/crates/v/ytmusicapi?style=flat-square&logo=rust" alt="Crates.io"></a>
  <a href="https://docs.rs/ytmusicapi"><img src="https://img.shields.io/docsrs/ytmusicapi?style=flat-square&logo=readthedocs" alt="Documentation"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License"></a>
</p>

<p align="center">
  <a href="#overview">Overview</a> &middot;
  <a href="#getting-started">Getting Started</a> &middot;
  <a href="#examples">Examples</a> &middot;
  <a href="#api-surface">API Surface</a> &middot;
  <a href="#development">Development</a>
</p>

## Overview

`ytmusicapi` wraps the YouTube Music web client's YouTubei endpoints with typed Rust models, browser-cookie authentication, and async `reqwest` requests. It is useful for tools that need to inspect a YouTube Music library, read playlist contents, create playlists, move tracks, or update likes from Rust.

> [!NOTE]
> This crate uses an unofficial internal API. YouTube Music can change these endpoints without notice, so callers should handle server and parsing errors as part of normal operation.

## Features

- Browser-cookie authentication with generated `SAPISIDHASH` authorization headers.
- Library playlist listing and playlist metadata fetching.
- Playlist track pagination, capped at 5,000 tracks when no explicit limit is supplied.
- Playlist creation, deletion, item add, item removal, and item moves.
- Liked songs access and song rating helpers.
- Unauthenticated song metadata lookup through the `player` endpoint.
- Typed playlist, track, artist, album, thumbnail, song, and error models.

## Getting Started

Add the crate and Tokio runtime to your project:

```toml
[dependencies]
ytmusicapi = "0.4"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Authentication

Most library and playlist management operations require browser headers from a signed-in YouTube Music session.

1. Open [YouTube Music](https://music.youtube.com) and sign in.
2. Open Developer Tools and select the Network tab.
3. Filter for a `browse` request to `music.youtube.com`.
4. Copy the full `cookie` request header and the `x-goog-authuser` header.
5. Save them as `headers.json`.

```json
{
  "cookie": "SID=...; __Secure-3PAPISID=...; ...",
  "x-goog-authuser": "0"
}
```

> [!IMPORTANT]
> Keep `headers.json` private. The cookie grants access to your account, and it must include `__Secure-3PAPISID` for authenticated requests.

### Quick Start

```rust
use ytmusicapi::{BrowserAuth, YTMusicClient};

#[tokio::main]
async fn main() -> ytmusicapi::Result<()> {
    let auth = BrowserAuth::from_file("headers.json")?;
    let client = YTMusicClient::builder()
        .with_browser_auth(auth)
        .build()?;

    let playlists = client.get_library_playlists(Some(10)).await?;
    for playlist in playlists {
        println!("{}: {}", playlist.playlist_id, playlist.title);
    }

    Ok(())
}
```

Song metadata can be fetched without browser authentication:

```rust
use ytmusicapi::YTMusicClient;

#[tokio::main]
async fn main() -> ytmusicapi::Result<()> {
    let client = YTMusicClient::builder().build()?;
    let song = client.get_song("dQw4w9WgXcQ").await?;

    println!("{} by {}", song.video_details.title, song.video_details.author);
    Ok(())
}
```

## Examples

The checked-in examples expect `headers.json` in the repository root for authenticated operations.

```bash
cargo run --example list_playlists
cargo run --example get_liked_songs -- --limit 50
cargo run --example create_playlist -- --title "My Playlist" --privacy private
cargo run --example add_song_to_playlist -- --playlist-id PLAYLIST_ID --video-id VIDEO_ID
cargo run --example remove_playlist_items -- --playlist-id PLAYLIST_ID --video-ids VIDEO_ID_1,VIDEO_ID_2
cargo run --example move_playlist_items -- --source PLAYLIST_ID --dest PLAYLIST_ID --video-ids VIDEO_ID_1,VIDEO_ID_2
cargo run --example like_song -- --video-id VIDEO_ID
cargo run --example unlike_song -- --video-id VIDEO_ID
cargo run --example delete_playlist -- --playlist-id PLAYLIST_ID
```

## API Surface

| Area | Methods |
| --- | --- |
| Client setup | `YTMusicClient::builder`, `is_authenticated` |
| Playlists | `get_library_playlists`, `get_playlist`, `create_playlist`, `delete_playlist` |
| Playlist items | `add_playlist_items`, `remove_playlist_items`, `move_playlist_items` |
| Songs | `get_song`, `get_liked_songs`, `rate_song`, `like_song`, `unlike_song` |
| Configuration | `with_browser_auth`, `with_language`, `with_location`, `with_user` |
| Low-level access | `send_request` |

Common exported types include `BrowserAuth`, `Playlist`, `PlaylistSummary`, `PlaylistTrack`, `Privacy`, `LikeStatus`, `Song`, `Artist`, `Album`, `Thumbnail`, `Error`, and `Result`.

## Caveats

- Authenticated methods return `Error::AuthRequired` when no `BrowserAuth` is configured.
- `get_song` returns metadata only, not stream URLs.
- `get_library_playlists` currently reads the first library page and applies the requested limit locally.
- The client does not add automatic retries or custom request timeouts.
- Private or account-specific data depends on the validity of the supplied browser cookies.

## Development

```bash
cargo fmt
cargo clippy --all-targets
cargo test
```

## Acknowledgement

This project is inspired by the Python [ytmusicapi](https://github.com/sigma67/ytmusicapi) project.
