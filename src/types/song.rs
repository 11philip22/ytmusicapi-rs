use serde::{Deserialize, Serialize};

/// Metadata returned by the `player` endpoint.
///
/// This is a partial view of the YouTube Music response and may omit fields
/// depending on availability.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    /// Core video metadata.
    pub video_details: VideoDetails,
    /// Optional microformat metadata.
    pub microformat: Option<Microformat>,
}

/// Core video metadata.
///
/// Note that numeric values like `length_seconds` and `view_count` are returned
/// as strings by the API.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    /// Video ID (11-character YouTube ID).
    pub video_id: String,
    /// Title of the song/video.
    pub title: String,
    /// Author/artist as presented by the API.
    pub author: String,
    /// Length in seconds, represented as a string.
    pub length_seconds: String,
    /// View count, represented as a string.
    pub view_count: String,
    /// Keyword tags, if present.
    pub keywords: Option<Vec<String>>,
}

/// Microformat wrapper.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Microformat {
    /// Microformat data renderer payload.
    pub microformat_data_renderer: MicroformatDataRenderer,
}

/// Microformat metadata values.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicroformatDataRenderer {
    /// Category label, if provided (for example, "Music").
    pub category: Option<String>,
    /// Upload date as provided by the API.
    pub upload_date: String,
    /// View count, represented as a string.
    pub view_count: String,
    /// Tags, if present.
    pub tags: Option<Vec<String>>,
}
