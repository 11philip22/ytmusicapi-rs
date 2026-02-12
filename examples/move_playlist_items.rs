//! Example: Move items from one playlist to another.
//!
//! Usage:
//! 1. Export your browser headers to `headers.json` (see README)
//! 2. Run:
//!    cargo run --example move_playlist_items -- \
//!      --source PLAYLIST_ID \
//!      --dest PLAYLIST_ID \
//!      --video-ids VIDEO_ID_1,VIDEO_ID_2 \
//!      [--allow-duplicates]

use std::collections::HashSet;
use std::env;

use ytmusicapi::{BrowserAuth, PlaylistTrack, YTMusicClient};

#[derive(Default)]
struct Args {
    source_playlist_id: Option<String>,
    dest_playlist_id: Option<String>,
    video_ids: Option<String>,
    allow_duplicates: bool,
    show_help: bool,
}

fn parse_video_ids(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(String::from)
        .collect()
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        if let Some(value) = arg.strip_prefix("--source=") {
            args.source_playlist_id = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }
        if let Some(value) = arg.strip_prefix("--dest=") {
            args.dest_playlist_id = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }
        if let Some(value) = arg.strip_prefix("--destination=") {
            args.dest_playlist_id = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }
        if let Some(value) = arg.strip_prefix("--video-ids=") {
            args.video_ids = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }

        match arg.as_str() {
            "--help" | "-h" => {
                args.show_help = true;
                return Ok(args);
            }
            "--source" | "-s" => {
                args.source_playlist_id = iter
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty());
            }
            "--dest" | "--destination" | "-d" => {
                args.dest_playlist_id = iter
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty());
            }
            "--video-ids" | "-v" => {
                args.video_ids = iter
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
    eprintln!("  cargo run --example move_playlist_items -- \\\n    --source PLAYLIST_ID \\\n    --dest PLAYLIST_ID \\\n    --video-ids VIDEO_ID_1,VIDEO_ID_2 \\\n    [--allow-duplicates]");
}

fn collect_items(tracks: &[PlaylistTrack], video_ids: &HashSet<String>) -> Vec<PlaylistTrack> {
    tracks
        .iter()
        .filter(|track| {
            track
                .video_id
                .as_ref()
                .map(|id| video_ids.contains(id))
                .unwrap_or(false)
        })
        .filter(|track| track.set_video_id.is_some())
        .cloned()
        .collect()
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

    let source_playlist_id = match args.source_playlist_id {
        Some(value) => value,
        None => {
            eprintln!("Missing --source PLAYLIST_ID.");
            print_usage();
            return Ok(());
        }
    };

    let dest_playlist_id = match args.dest_playlist_id {
        Some(value) => value,
        None => {
            eprintln!("Missing --dest PLAYLIST_ID.");
            print_usage();
            return Ok(());
        }
    };

    let raw_video_ids = match args.video_ids {
        Some(value) => value,
        None => {
            eprintln!("Missing --video-ids VIDEO_ID_1,VIDEO_ID_2.");
            print_usage();
            return Ok(());
        }
    };

    if source_playlist_id == dest_playlist_id {
        eprintln!("Source and destination playlist IDs must be different.");
        return Ok(());
    }

    let video_ids = parse_video_ids(&raw_video_ids);
    if video_ids.is_empty() {
        eprintln!("Provide at least one video ID.");
        return Ok(());
    }

    let client = YTMusicClient::builder().with_browser_auth(auth).build()?;

    println!("Fetching source playlist to locate items...");
    let playlist = client.get_playlist(&source_playlist_id, None).await?;

    let video_id_set: HashSet<String> = video_ids.into_iter().collect();
    let items = collect_items(&playlist.tracks, &video_id_set);

    if items.is_empty() {
        eprintln!("No matching playlist items found to move.");
        return Ok(());
    }

    println!(
        "Moving {} items to destination playlist {}...",
        items.len(),
        dest_playlist_id
    );
    client
        .move_playlist_items(
            &source_playlist_id,
            &dest_playlist_id,
            &items,
            args.allow_duplicates,
        )
        .await?;
    println!("Moved.");

    Ok(())
}
