//! Common types shared across the API.

use serde::{Deserialize, Serialize};

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
