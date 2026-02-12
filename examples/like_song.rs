//! Example: Like a song.
//!
//! Usage:
//! 1. Export your browser headers to `headers.json` (see README)
//! 2. Run:
//!    cargo run --example like_song -- \
//!      --video-id VIDEO_ID

use std::env;

use ytmusicapi::{BrowserAuth, YTMusicClient};

#[derive(Default)]
struct Args {
    video_id: Option<String>,
    show_help: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        if let Some(value) = arg.strip_prefix("--video-id=") {
            args.video_id = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }

        match arg.as_str() {
            "--help" | "-h" => {
                args.show_help = true;
                return Ok(args);
            }
            "--video-id" | "-v" => {
                args.video_id = iter
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty());
            }
            _ => return Err(format!("Unknown argument: {}", arg)),
        }
    }

    Ok(args)
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  cargo run --example like_song -- \\\n    --video-id VIDEO_ID");
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

    let video_id = match args.video_id {
        Some(value) => value,
        None => {
            eprintln!("Missing --video-id.");
            print_usage();
            return Ok(());
        }
    };

    let client = YTMusicClient::builder().with_browser_auth(auth).build()?;

    println!("Liking video '{}'...", video_id);
    client.like_song(&video_id).await?;
    println!("Liked.");

    Ok(())
}
