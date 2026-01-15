//! Playlist types.

use serde::{Deserialize, Serialize};

use super::{Album, Artist, Author, Thumbnail};

/// Privacy status of a playlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Privacy {
    /// Visible to everyone
    #[default]
    Public,
    /// Only visible to owner
    Private,
    /// Visible to anyone with the link
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
    /// Playlist ID
    pub playlist_id: String,
    /// Playlist title
    pub title: String,
    /// Thumbnail images
    pub thumbnails: Vec<Thumbnail>,
    /// Number of tracks (if available)
    pub count: Option<u32>,
}

/// Full playlist with tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Playlist ID
    pub id: String,
    /// Playlist title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Privacy setting
    pub privacy: Privacy,
    /// Thumbnail images
    pub thumbnails: Vec<Thumbnail>,
    /// Author/creator of the playlist
    pub author: Option<Author>,
    /// Year created/updated
    pub year: Option<String>,
    /// Human-readable duration (e.g., "2 hours")
    pub duration: Option<String>,
    /// Total duration in seconds
    pub duration_seconds: Option<u32>,
    /// Number of tracks
    pub track_count: Option<u32>,
    /// Whether the current user owns this playlist
    pub owned: bool,
    /// Playlist tracks
    pub tracks: Vec<PlaylistTrack>,
}

/// A track within a playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    /// Video ID (used for playback)
    pub video_id: Option<String>,
    /// Track title
    pub title: Option<String>,
    /// Artists
    pub artists: Vec<Artist>,
    /// Album info
    pub album: Option<Album>,
    /// Human-readable duration (e.g., "3:42")
    pub duration: Option<String>,
    /// Duration in seconds
    pub duration_seconds: Option<u32>,
    /// Thumbnail images
    pub thumbnails: Vec<Thumbnail>,
    /// Whether the track is available for playback
    pub is_available: bool,
    /// Whether the track has explicit content
    pub is_explicit: bool,
    /// Unique ID of this playlist item (for reordering/removing)
    pub set_video_id: Option<String>,
    /// Type of video (e.g., "MUSIC_VIDEO_TYPE_OMV")
    pub video_type: Option<String>,
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
