//! Example: Create a playlist.
//!
//! Usage:
//! 1. Export your browser headers to `headers.json` (see README)
//! 2. Run:
//!    cargo run --example create_playlist -- \
//!      --title "My Playlist" \
//!      [--description "Created via ytmusicapi-rs"] \
//!      [--privacy private]

use std::env;

use ytmusicapi::{BrowserAuth, Privacy, YTMusicClient};

#[derive(Default)]
struct Args {
    title: Option<String>,
    description: Option<String>,
    privacy: Option<String>,
    show_help: bool,
}

fn parse_privacy(value: &str) -> Option<Privacy> {
    match value.trim().to_lowercase().as_str() {
        "public" => Some(Privacy::Public),
        "private" => Some(Privacy::Private),
        "unlisted" => Some(Privacy::Unlisted),
        _ => None,
    }
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();
    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        if let Some(value) = arg.strip_prefix("--title=") {
            args.title = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }
        if let Some(value) = arg.strip_prefix("--description=") {
            args.description = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }
        if let Some(value) = arg.strip_prefix("--privacy=") {
            args.privacy = Some(value.trim().to_string()).filter(|v| !v.is_empty());
            continue;
        }

        match arg.as_str() {
            "--help" | "-h" => {
                args.show_help = true;
                return Ok(args);
            }
            "--title" | "-t" => {
                args.title = iter
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty());
            }
            "--description" | "-d" => {
                args.description = iter
                    .next()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty());
            }
            "--privacy" | "-p" => {
                args.privacy = iter
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
    eprintln!("  cargo run --example create_playlist -- \\\n    --title \"My Playlist\" \\\n    [--description \"Created via ytmusicapi-rs\"] \\\n    [--privacy private]");
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

    let title = match args.title {
        Some(value) => value,
        None => {
            eprintln!("Missing --title.");
            print_usage();
            return Ok(());
        }
    };

    let privacy = match args.privacy {
        Some(value) => match parse_privacy(&value) {
            Some(parsed) => parsed,
            None => {
                eprintln!("Unknown privacy value: {}", value);
                print_usage();
                return Ok(());
            }
        },
        None => Privacy::Private,
    };

    let client = YTMusicClient::builder().with_browser_auth(auth).build()?;

    println!("Creating playlist '{}'...", title);
    let response = client
        .create_playlist(&title, args.description.as_deref(), privacy)
        .await?;
    println!("Playlist ID: {}", response.playlist_id);

    Ok(())
}
