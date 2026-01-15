//! Example: List all playlists from your YouTube Music library.
//!
//! Usage:
//! 1. Export your browser headers to `headers.json` (see README)
//! 2. Run: cargo run --example list_playlists

use ytmusicapi::{BrowserAuth, YTMusicClient};

#[tokio::main]
async fn main() -> ytmusicapi::Result<()> {
    // Load auth from headers.json file
    let auth = match BrowserAuth::from_file("headers.json") {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error loading headers.json: {}", e);
            eprintln!("\nTo create headers.json:");
            eprintln!("1. Open YouTube Music in your browser and log in");
            eprintln!("2. Open Developer Tools (F12) → Network tab");
            eprintln!("3. Find any request to music.youtube.com");
            eprintln!("4. Copy the request headers and save as JSON");
            eprintln!("\nExample headers.json:");
            eprintln!(r#"{{"cookie": "...", "x-goog-authuser": "0"}}"#);
            return Ok(());
        }
    };

    let client = YTMusicClient::builder().with_browser_auth(auth).build()?;

    println!("Fetching your playlists...\n");

    let playlists = client.get_library_playlists(None).await?;

    if playlists.is_empty() {
        println!("No playlists found.");
        return Ok(());
    }

    println!("Found {} playlists:\n", playlists.len());

    for pl in &playlists {
        let count = pl
            .count
            .map(|c| format!("{} tracks", c))
            .unwrap_or_default();
        println!("  {} - {} ({})", pl.playlist_id, pl.title, count);
    }

    // Optionally fetch first playlist details
    if let Some(first) = playlists.first() {
        println!("\n---\nFetching details for: {}\n", first.title);

        let playlist = client.get_playlist(&first.playlist_id, Some(10)).await?;

        println!("Title: {}", playlist.title);
        if let Some(desc) = &playlist.description {
            println!("Description: {}", desc);
        }
        println!("Privacy: {:?}", playlist.privacy);
        println!("Track count: {:?}", playlist.track_count);
        println!("\nFirst {} tracks:", playlist.tracks.len());

        for track in &playlist.tracks {
            let title = track.title.as_deref().unwrap_or("Unknown");
            let artists: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
            let duration = track.duration.as_deref().unwrap_or("--:--");
            println!("  [{}] {} - {}", duration, artists.join(", "), title);
        }
    }

    Ok(())
}
