//! Example: Fetch your liked songs playlist.
//!
//! Usage:
//! 1. Export your browser headers to `headers.json` (see README)
//! 2. Run:
//!    cargo run --example get_liked_songs -- [--limit 50]

use std::env;

use ytmusicapi::{BrowserAuth, YTMusicClient};

#[derive(Default)]
struct Args {
    limit: Option<u32>,
    show_help: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        if let Some(value) = arg.strip_prefix("--limit=") {
            args.limit = value.trim().parse::<u32>().ok();
            continue;
        }

        match arg.as_str() {
            "--help" | "-h" => {
                args.show_help = true;
                return Ok(args);
            }
            "--limit" | "-l" => {
                args.limit = iter.next().and_then(|value| value.trim().parse::<u32>().ok());
            }
            _ => return Err(format!("Unknown argument: {}", arg)),
        }
    }

    Ok(args)
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  cargo run --example get_liked_songs -- [--limit 50]");
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

    let client = YTMusicClient::builder().with_browser_auth(auth).build()?;

    println!("Fetching liked songs...");
    let playlist = client.get_liked_songs(args.limit).await?;

    println!("Title: {}", playlist.title);
    println!("Track count: {:?}", playlist.track_count);
    println!("Returned tracks: {}", playlist.tracks.len());

    for track in &playlist.tracks {
        let title = track.title.as_deref().unwrap_or("Unknown");
        let artists: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
        let duration = track.duration.as_deref().unwrap_or("--:--");
        println!("  [{}] {} - {}", duration, artists.join(", "), title);
    }

    Ok(())
}
