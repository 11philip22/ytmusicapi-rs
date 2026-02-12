//! Playlist types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Album, Artist, Author, Thumbnail};

/// Privacy status of a playlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Privacy {
    /// Visible to everyone.
    #[default]
    Public,
    /// Only visible to the owner.
    Private,
    /// Visible to anyone with the link.
    Unlisted,
}

impl From<&str> for Privacy {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "PUBLIC" => Privacy::Public,
            "PRIVATE" => Privacy::Private,
            "UNLISTED" => Privacy::Unlisted,
            _ => Privacy::Public,
        }
    }
}

/// Summary info for a playlist in a library listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistSummary {
    /// Playlist ID without the `VL` prefix.
    pub playlist_id: String,
    /// Playlist title.
    pub title: String,
    /// Thumbnail images.
    pub thumbnails: Vec<Thumbnail>,
    /// Number of tracks, if provided by the API.
    pub count: Option<u32>,
}

/// Full playlist with tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Playlist ID without the `VL` prefix.
    pub id: String,
    /// Playlist title.
    pub title: String,
    /// Description.
    pub description: Option<String>,
    /// Privacy setting.
    pub privacy: Privacy,
    /// Thumbnail images.
    pub thumbnails: Vec<Thumbnail>,
    /// Author/creator of the playlist, if available.
    pub author: Option<Author>,
    /// Year created/updated, if present in the response.
    pub year: Option<String>,
    /// Human-readable duration (e.g., `"2 hours"`), if present.
    pub duration: Option<String>,
    /// Total duration in seconds, computed from parsed tracks.
    pub duration_seconds: Option<u32>,
    /// Number of tracks, if provided by the API.
    pub track_count: Option<u32>,
    /// Whether the current user owns this playlist.
    pub owned: bool,
    /// Playlist tracks.
    pub tracks: Vec<PlaylistTrack>,
}

/// A track within a playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    /// Video ID (used for playback), if available.
    pub video_id: Option<String>,
    /// Track title, if available.
    pub title: Option<String>,
    /// Artists.
    pub artists: Vec<Artist>,
    /// Album info, if available.
    pub album: Option<Album>,
    /// Human-readable duration (e.g., `"3:42"`), if available.
    pub duration: Option<String>,
    /// Duration in seconds, if parsed successfully.
    pub duration_seconds: Option<u32>,
    /// Thumbnail images.
    pub thumbnails: Vec<Thumbnail>,
    /// Whether the track is available for playback.
    pub is_available: bool,
    /// Whether the track has explicit content.
    pub is_explicit: bool,
    /// Unique playlist item ID used for removing/reordering.
    pub set_video_id: Option<String>,
    /// Type of video (e.g., `"MUSIC_VIDEO_TYPE_OMV"`), if available.
    pub video_type: Option<String>,
}

/// Result of moving items between playlists.
#[derive(Debug, Clone)]
pub struct MovePlaylistItemsResult {
    /// Response from adding items to the destination playlist.
    pub add_response: Value,
    /// Response from removing items from the source playlist.
    pub remove_response: Value,
}

/// Response from creating a playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlaylistResponse {
    /// The newly created playlist ID.
    #[serde(rename = "playlistId")]
    pub playlist_id: String,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            description: None,
            privacy: Privacy::Public,
            thumbnails: Vec::new(),
            author: None,
            year: None,
            duration: None,
            duration_seconds: None,
            track_count: None,
            owned: false,
            tracks: Vec::new(),
        }
    }
}

impl Default for PlaylistTrack {
    fn default() -> Self {
        Self {
            video_id: None,
            title: None,
            artists: Vec::new(),
            album: None,
            duration: None,
            duration_seconds: None,
            thumbnails: Vec::new(),
            is_available: true,
            is_explicit: false,
            set_video_id: None,
            video_type: None,
        }
    }
}
