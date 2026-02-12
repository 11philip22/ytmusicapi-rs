//! Common types shared across the API.

use serde::{Deserialize, Serialize};

/// Like/Dislike status for rating a song or playlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LikeStatus {
    /// Thumbs up / like
    Like,
    /// Thumbs down / dislike
    Dislike,
    /// Remove any existing rating
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
    /// URL of the thumbnail
    pub url: String,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
}

/// An artist reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// Artist name
    pub name: String,
    /// Artist ID (browse ID), may be None for some artists
    pub id: Option<String>,
}

/// An album reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// Album name
    pub name: String,
    /// Album ID (browse ID)
    pub id: Option<String>,
}

/// Author of a playlist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Author name
    pub name: String,
    /// Author channel ID
    pub id: Option<String>,
}
