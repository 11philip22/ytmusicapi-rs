//! Track/song parsing utilities.

use serde_json::Value;

use crate::nav::{nav, nav_str};
use crate::path;
use crate::types::{Album, Artist};

/// Parse duration string to seconds.
///
/// # Example
///
/// ```
/// use ytmusicapi::parsers::track::parse_duration;
///
/// assert_eq!(parse_duration("3:42"), Some(222));
/// assert_eq!(parse_duration("1:23:45"), Some(5025));
/// assert_eq!(parse_duration(""), None);
/// ```
pub fn parse_duration(duration: &str) -> Option<u32> {
    let duration = duration.trim();
    if duration.is_empty() {
        return None;
    }

    let parts: Vec<&str> = duration.split(':').collect();
    let mut seconds = 0u32;

    for (i, part) in parts.iter().rev().enumerate() {
        let value: u32 = part.parse().ok()?;
        let multiplier = match i {
            0 => 1,    // seconds
            1 => 60,   // minutes
            2 => 3600, // hours
            _ => return None,
        };
        seconds += value * multiplier;
    }

    Some(seconds)
}

/// Parse artists from flex column runs.
pub fn parse_song_artists(data: &Value, index: usize) -> Vec<Artist> {
    let flex_item = get_flex_column_item(data, index);
    let flex_item = match flex_item {
        Some(v) => v,
        None => return Vec::new(),
    };

    let runs = match nav(flex_item, &path!["text", "runs"]) {
        Some(Value::Array(arr)) => arr,
        _ => return Vec::new(),
    };

    parse_artist_runs(runs)
}

/// Parse artist runs into Artist structs.
pub fn parse_artist_runs(runs: &[Value]) -> Vec<Artist> {
    let mut artists = Vec::new();

    for run in runs.iter().step_by(2) {
        // Skip separators (every other item)
        let name = match run.get("text").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        let id = nav_str(
            run,
            &path!["navigationEndpoint", "browseEndpoint", "browseId"],
        )
        .map(|s| s.to_string());

        artists.push(Artist { name, id });
    }

    artists
}

/// Parse album info from a flex column.
pub fn parse_song_album(data: &Value, index: usize) -> Option<Album> {
    let flex_item = get_flex_column_item(data, index)?;

    let name = nav_str(flex_item, &path!["text", "runs", 0, "text"])?.to_string();

    let id = nav_str(
        flex_item,
        &path![
            "text",
            "runs",
            0,
            "navigationEndpoint",
            "browseEndpoint",
            "browseId"
        ],
    )
    .map(|s| s.to_string());

    Some(Album { name, id })
}

/// Get a flex column item from a music responsive list item.
pub fn get_flex_column_item(data: &Value, index: usize) -> Option<&Value> {
    let columns = data.get("flexColumns")?.as_array()?;
    let column = columns.get(index)?;
    let renderer = column.get("musicResponsiveListItemFlexColumnRenderer")?;

    // Check that text and runs exist
    if renderer.get("text")?.get("runs").is_none() {
        return None;
    }

    Some(renderer)
}

/// Get a fixed column item from a music responsive list item.
pub fn get_fixed_column_item(data: &Value, index: usize) -> Option<&Value> {
    let columns = data.get("fixedColumns")?.as_array()?;
    let column = columns.get(index)?;
    column.get("musicResponsiveListItemFixedColumnRenderer")
}

/// Get text from an item at a specific flex column index.
pub fn get_item_text(item: &Value, index: usize) -> Option<&str> {
    let column = get_flex_column_item(item, index)?;
    nav_str(column, &path!["text", "runs", 0, "text"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("3:42"), Some(222));
        assert_eq!(parse_duration("0:30"), Some(30));
        assert_eq!(parse_duration("1:00:00"), Some(3600));
        assert_eq!(parse_duration("1:23:45"), Some(5025));
        assert_eq!(parse_duration(""), None);
        assert_eq!(parse_duration("  "), None);
    }

    #[test]
    fn test_parse_artist_runs() {
        let runs = serde_json::json!([
            {"text": "Artist 1", "navigationEndpoint": {"browseEndpoint": {"browseId": "UC123"}}},
            {"text": " & "},
            {"text": "Artist 2"}
        ]);

        let artists = parse_artist_runs(runs.as_array().unwrap());
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].name, "Artist 1");
        assert_eq!(artists[0].id, Some("UC123".to_string()));
        assert_eq!(artists[1].name, "Artist 2");
        assert_eq!(artists[1].id, None);
    }
}
