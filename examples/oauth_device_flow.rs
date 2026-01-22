//! Example: Authenticate with OAuth device flow and list your playlists.
//!
//! Prerequisites:
//! 1. Create a YouTube Data API OAuth client (type: TVs and Limited Input devices).
//! 2. Export the credentials as environment variables:
//!    - CLIENT_ID
//!    - CLIENT_SECRET
//!
//! Run:
//! ```bash
//! CLIENT_ID=... CLIENT_SECRET=... cargo run --example oauth_device_flow
//! ```

use std::io::{self, Write};
use std::path::Path;

use ytmusicapi::{OAuthCredentials, OAuthToken, YTMusicClient};

#[tokio::main]
async fn main() -> ytmusicapi::Result<()> {
    let client_id = std::env::var("CLIENT_ID").expect("Please set CLIENT_ID env var");
    let client_secret = std::env::var("CLIENT_SECRET").expect("Please set CLIENT_SECRET env var");
    let credentials = OAuthCredentials::new(client_id, client_secret)?;

    // Reuse existing token if present, otherwise run device flow.
    let token_path = "oauth.json";
    let token = if Path::new(token_path).exists() {
        println!("Found existing oauth.json; reusing it.");
        OAuthToken::from_file(token_path)?
    } else {
        let device = credentials.request_device_code().await?;
        println!(
            "Visit {} and enter the code: {}",
            device.verification_url, device.user_code
        );
        print!("Press Enter after you finish the browser flow...");
        io::stdout().flush().ok();
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf);

        let token = credentials
            .exchange_device_code(&device.device_code)
            .await?;
        token.to_file(token_path)?;
        println!("\nSaved token to {}", token_path);
        token
    };

    let client = YTMusicClient::builder()
        .with_oauth_token_and_credentials(token, credentials)
        .build()?;

    println!("\nFetching up to 10 library playlists...\n");
    let playlists = client.get_library_playlists(Some(10)).await?;
    if playlists.is_empty() {
        println!("No playlists found.");
    } else {
        for pl in playlists {
            let count = pl
                .count
                .map(|c| format!("{} tracks", c))
                .unwrap_or_default();
            println!("{} - {} ({})", pl.playlist_id, pl.title, count);
        }
    }

    Ok(())
}
