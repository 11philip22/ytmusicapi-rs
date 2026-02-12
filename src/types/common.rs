//! Common types shared across the API.

use serde::{Deserialize, Serialize};

/// Rating status for a song.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LikeStatus {
    /// Thumbs up / like.
    Like,
    /// Thumbs down / dislike.
    Dislike,
    /// Remove any existing rating.
    Indifferent,
}

impl LikeStatus {
    pub(crate) fn endpoint(self) -> &'static str {
        match self {
            LikeStatus::Like => "like/like",
            LikeStatus::Dislike => "like/dislike",
            LikeStatus::Indifferent => "like/removelike",
        }
    }
}

/// A thumbnail image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thumbnail {
    /// URL of the thumbnail.
    pub url: String,
    /// Width in pixels, if provided by the API.
    pub width: Option<u32>,
    /// Height in pixels, if provided by the API.
    pub height: Option<u32>,
}

/// An artist reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// Artist name.
    pub name: String,
    /// Artist browse ID, if available.
    pub id: Option<String>,
}

/// An album reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// Album name.
    pub name: String,
    /// Album browse ID, if available.
    pub id: Option<String>,
}

/// Author of a playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Author name.
    pub name: String,
    /// Author channel browse ID, if available.
    pub id: Option<String>,
}
