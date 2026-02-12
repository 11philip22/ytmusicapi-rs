//! Example: Add a song to a playlist.
//!
//! Usage:
//! 1. Export your browser headers to `headers.json` (see README)
//! 2. Run:
//!    cargo run --example add_song_to_playlist -- \
//!      --playlist-id PLAYLIST_ID \
//!      --video-id VIDEO_ID \
//!      [--allow-duplicates]

use std::env;

use ytmusicapi::{BrowserAuth, YTMusicClient};

#[derive(Default)]
struct Args {
    playlist_id: Option<String>,
    video_id: Option<String>,
    allow_duplicates: bool,
    show_help: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        if let Some(value) = arg.strip_prefix("--playlist-id=") {
            args.playlist_id = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }
        if let Some(value) = arg.strip_prefix("--video-id=") {
            args.video_id = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }

        match arg.as_str() {
            "--help" | "-h" => {
                args.show_help = true;
                return Ok(args);
            }
            "--playlist-id" | "-p" => {
                args.playlist_id = iter
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty());
            }
            "--video-id" | "-v" => {
                args.video_id = iter
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty());
            }
            "--allow-duplicates" | "-a" => {
                args.allow_duplicates = true;
            }
            _ => return Err(format!("Unknown argument: {}", arg)),
        }
    }

    Ok(args)
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  cargo run --example add_song_to_playlist -- \\\n    --playlist-id PLAYLIST_ID \\\n    --video-id VIDEO_ID \\\n    [--allow-duplicates]");
}

#[tokio::main]
async fn main() -> ytmusicapi::Result<()> {
    // Load auth from headers.json file
    let auth = match BrowserAuth::from_file("headers.json") {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error loading headers.json: {}", e);
            eprintln!("\nTo create headers.json:");
            eprintln!("1. Open YouTube Music in your browser and log in");
            eprintln!("2. Open Developer Tools (F12) -> Network tab");
            eprintln!("3. Find any request to music.youtube.com");
            eprintln!("4. Copy the request headers and save as JSON");
            eprintln!("\nExample headers.json:");
            eprintln!("{}", r#"{"cookie": "...", "x-goog-authuser": "0"}"#);
            return Ok(());
        }
    };

    let args = match parse_args() {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("{}", err);
            print_usage();
            return Ok(());
        }
    };

    if args.show_help {
        print_usage();
        return Ok(());
    }

    let playlist_id = match args.playlist_id {
        Some(value) => value,
        None => {
            eprintln!("Missing --playlist-id.");
            print_usage();
            return Ok(());
        }
    };

    let video_id = match args.video_id {
        Some(value) => value,
        None => {
            eprintln!("Missing --video-id.");
            print_usage();
            return Ok(());
        }
    };

    let client = YTMusicClient::builder().with_browser_auth(auth).build()?;

    println!("Adding video '{}' to playlist '{}'...", video_id, playlist_id);
    client
        .add_playlist_items(&playlist_id, &[video_id], args.allow_duplicates)
        .await?;
    println!("Added.");

    Ok(())
}
