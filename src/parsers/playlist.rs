//! Playlist response parsing.

use serde_json::Value;

use crate::nav::{nav, nav_array, nav_str};
use crate::parsers::navigation::paths;
use crate::parsers::track::{
    get_fixed_column_item, get_item_text, parse_duration, parse_song_album, parse_song_artists,
};
use crate::path;
use crate::types::{Author, Playlist, PlaylistSummary, PlaylistTrack, Privacy, Thumbnail};

/// Parse library playlists from browse response.
pub fn parse_library_playlists(response: &Value) -> Vec<PlaylistSummary> {
    // Navigate to grid items
    // Path: contents.singleColumnBrowseResultsRenderer.tabs[0].tabRenderer.content
    //       .sectionListRenderer.contents[0].gridRenderer.items
    let single_column = nav(response, paths::SINGLE_COLUMN);
    let single_column = match single_column {
        Some(v) => v,
        None => return Vec::new(),
    };

    let tab_content = nav(single_column, paths::TAB_CONTENT);
    let tab_content = match tab_content {
        Some(v) => v,
        None => return Vec::new(),
    };

    let section_list = nav(tab_content, paths::SECTION_LIST);
    let section_list = match section_list {
        Some(Value::Array(arr)) => arr,
        _ => return Vec::new(),
    };

    // Find the grid in section list
    let grid_items = section_list.iter().find_map(|item| {
        let grid = nav(item, paths::GRID_ITEMS)?;
        grid.as_array()
    });

    let items = match grid_items {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    // Skip the first item (usually "New playlist" button)
    items
        .iter()
        .skip(1)
        .filter_map(|item| parse_playlist_item(item))
        .collect()
}

/// Parse a single playlist item from library listing.
fn parse_playlist_item(item: &Value) -> Option<PlaylistSummary> {
    let renderer = item.get(paths::MTRIR)?;

    let title = nav_str(renderer, paths::TITLE_TEXT)?.to_string();

    let playlist_id = nav_str(renderer, paths::NAVIGATION_PLAYLIST_ID)
        .or_else(|| nav_str(renderer, paths::NAVIGATION_BROWSE_ID))
        .map(|s| s.trim_start_matches("VL").to_string())?;

    let thumbnails = parse_thumbnails(renderer);

    // Count is in subtitle
    let count = nav_str(renderer, &path!["subtitle", "runs", 0, "text"]).and_then(|s| {
        // Parse "123 songs" or similar
        s.split_whitespace().next()?.parse().ok()
    });

    Some(PlaylistSummary {
        playlist_id,
        title,
        thumbnails,
        count,
    })
}

/// Parse thumbnails from a renderer.
pub fn parse_thumbnails(data: &Value) -> Vec<Thumbnail> {
    let thumbs = nav_array(data, paths::THUMBNAILS).or_else(|| nav_array(data, paths::THUMBNAIL));

    let thumbs = match thumbs {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    thumbs
        .iter()
        .filter_map(|t| {
            let url = t.get("url")?.as_str()?.to_string();
            let width = t.get("width").and_then(|v| v.as_u64()).map(|v| v as u32);
            let height = t.get("height").and_then(|v| v.as_u64()).map(|v| v as u32);
            Some(Thumbnail { url, width, height })
        })
        .collect()
}

/// Parse full playlist response.
pub fn parse_playlist_response(response: &Value, playlist_id: &str) -> Playlist {
    let mut playlist = Playlist {
        id: playlist_id.trim_start_matches("VL").to_string(),
        ..Default::default()
    };

    // Determine if owned playlist
    let two_col = nav(response, paths::TWO_COLUMN_RENDERER);
    let two_col = match two_col {
        Some(v) => v,
        None => return playlist,
    };

    let tab_content = nav(two_col, paths::TAB_CONTENT);
    let tab_content = match tab_content {
        Some(v) => v,
        None => return playlist,
    };

    let section_list_item = nav(tab_content, &path!["sectionListRenderer", "contents", 0]);
    let section_list_item = match section_list_item {
        Some(v) => v,
        None => return playlist,
    };

    // Check if editable (owned) playlist
    let editable_header = nav(section_list_item, paths::EDITABLE_PLAYLIST_DETAIL_HEADER);
    playlist.owned = editable_header.is_some();

    // Get header based on whether playlist is owned
    let header = if playlist.owned {
        let editable = editable_header.unwrap();
        playlist.privacy = nav_str(
            editable,
            &path!["editHeader", "musicPlaylistEditHeaderRenderer", "privacy"],
        )
        .map(Privacy::from)
        .unwrap_or(Privacy::Private);
        nav(editable, &path!["header", "musicResponsiveHeaderRenderer"])
    } else {
        playlist.privacy = Privacy::Public;
        nav(section_list_item, paths::RESPONSIVE_HEADER)
    };

    if let Some(header) = header {
        // Title
        playlist.title = nav_str(header, paths::TITLE_TEXT).unwrap_or("").to_string();

        // Thumbnails
        playlist.thumbnails = parse_thumbnails(header);

        // Description
        playlist.description = nav_str(
            header,
            &path![
                "description",
                "musicDescriptionShelfRenderer",
                "description",
                "runs",
                0,
                "text"
            ],
        )
        .map(|s| s.to_string());

        // Author from facepile or subtitle
        if let Some(author_name) = nav_str(
            header,
            &path!["facepile", "avatarStackViewModel", "text", "content"],
        ) {
            let author_id = nav_str(
                header,
                &path![
                    "facepile",
                    "avatarStackViewModel",
                    "rendererContext",
                    "commandContext",
                    "onTap",
                    "innertubeCommand",
                    "browseEndpoint",
                    "browseId"
                ],
            );
            playlist.author = Some(Author {
                name: author_name.to_string(),
                id: author_id.map(|s| s.to_string()),
            });
        }

        // Parse second subtitle for metadata
        if let Some(second_subtitle) = nav(header, &path!["secondSubtitle", "runs"]) {
            if let Some(runs) = second_subtitle.as_array() {
                parse_playlist_meta_from_runs(runs, &mut playlist);
            }
        }
    }

    // Parse tracks from secondary contents
    let secondary = nav(
        two_col,
        &path!["secondaryContents", "sectionListRenderer", "contents", 0],
    );
    if let Some(secondary) = secondary {
        let shelf = nav(secondary, &path!["musicPlaylistShelfRenderer", "contents"]);
        if let Some(Value::Array(contents)) = shelf {
            playlist.tracks = parse_playlist_tracks(contents);
        }
    }

    // Calculate total duration
    playlist.duration_seconds = Some(
        playlist
            .tracks
            .iter()
            .filter_map(|t| t.duration_seconds)
            .sum(),
    );

    playlist
}

/// Parse metadata from second subtitle runs.
fn parse_playlist_meta_from_runs(runs: &[Value], playlist: &mut Playlist) {
    // Format varies: could be "123 songs", "X songs • Y hours", "X views • Y songs • Z hours"
    for run in runs {
        if let Some(text) = run.get("text").and_then(|v| v.as_str()) {
            let text_lower = text.to_lowercase();

            if text_lower.contains("song") || text_lower.contains("track") {
                // Extract track count
                if let Some(count_str) = text.split_whitespace().next() {
                    if let Ok(count) = count_str.replace(',', "").parse::<u32>() {
                        playlist.track_count = Some(count);
                    }
                }
            } else if text_lower.contains("hour") || text_lower.contains("minute") {
                playlist.duration = Some(text.to_string());
            }
        }
    }
}

/// Parse playlist tracks from contents array.
pub fn parse_playlist_tracks(contents: &[Value]) -> Vec<PlaylistTrack> {
    contents
        .iter()
        .filter_map(|item| parse_playlist_track(item))
        .collect()
}

/// Parse a single playlist track.
pub fn parse_playlist_track(item: &Value) -> Option<PlaylistTrack> {
    let data = item.get(paths::MRLIR)?;

    let mut track = PlaylistTrack::default();

    // Video ID from play button
    track.video_id = nav_str(
        data,
        &path![
            "overlay",
            "musicItemThumbnailOverlayRenderer",
            "content",
            "musicPlayButtonRenderer",
            "playNavigationEndpoint",
            "watchEndpoint",
            "videoId"
        ],
    )
    .map(|s| s.to_string());

    // Set video ID from menu (for removing from playlist)
    if let Some(menu_items) = nav_array(data, paths::MENU_ITEMS) {
        for menu_item in menu_items {
            if let Some(service) = nav(
                menu_item,
                &path!["menuServiceItemRenderer", "serviceEndpoint"],
            ) {
                if let Some(set_video_id) = nav_str(
                    service,
                    &path!["playlistEditEndpoint", "actions", 0, "setVideoId"],
                ) {
                    track.set_video_id = Some(set_video_id.to_string());
                }
                // Also get video ID if we didn't get it from play button
                if track.video_id.is_none() {
                    track.video_id = nav_str(
                        service,
                        &path!["playlistEditEndpoint", "actions", 0, "removedVideoId"],
                    )
                    .map(|s| s.to_string());
                }
            }
        }
    }

    // Determine flex column indexes by analyzing content
    let flex_columns = data.get("flexColumns")?.as_array()?;

    // Title is usually first column
    track.title = get_item_text(data, 0).map(|s| s.to_string());

    // Skip deleted songs
    if track.title.as_deref() == Some("Song deleted") {
        return None;
    }

    // Artists usually second column
    track.artists = parse_song_artists(data, 1);

    // Try to find album (usually third column, but could vary)
    for i in 2..flex_columns.len() {
        if let Some(album) = parse_song_album(data, i) {
            track.album = Some(album);
            break;
        }
    }

    // Duration from fixed columns if available
    if let Some(fixed) = get_fixed_column_item(data, 0) {
        let duration = nav_str(fixed, &path!["text", "simpleText"])
            .or_else(|| nav_str(fixed, &path!["text", "runs", 0, "text"]));

        if let Some(dur) = duration {
            track.duration = Some(dur.to_string());
            track.duration_seconds = parse_duration(dur);
        }
    }

    // Thumbnails
    track.thumbnails = parse_thumbnails(data);

    // Availability
    if let Some(policy) = data
        .get("musicItemRendererDisplayPolicy")
        .and_then(|v| v.as_str())
    {
        track.is_available = policy != "MUSIC_ITEM_RENDERER_DISPLAY_POLICY_GREY_OUT";
    }

    // Explicit badge
    track.is_explicit = nav(data, paths::BADGE_LABEL).is_some();

    // Video type
    track.video_type = nav_str(
        data,
        &path![
            "menu",
            "menuRenderer",
            "items",
            0,
            "menuNavigationItemRenderer",
            "navigationEndpoint",
            "watchEndpoint",
            "watchEndpointMusicSupportedConfigs",
            "watchEndpointMusicConfig",
            "musicVideoType"
        ],
    )
    .map(|s| s.to_string());

    Some(track)
}

/// Get continuation token from results.
pub fn get_continuation_token(results: &Value) -> Option<String> {
    let contents = results.get("contents")?.as_array()?;
    let last = contents.last()?;
    nav_str(last, paths::CONTINUATION_TOKEN).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_thumbnails() {
        let data = json!({
            "thumbnail": {
                "thumbnails": [
                    {"url": "https://example.com/1.jpg", "width": 100, "height": 100},
                    {"url": "https://example.com/2.jpg", "width": 200, "height": 200}
                ]
            }
        });

        let thumbs = parse_thumbnails(&data);
        assert_eq!(thumbs.len(), 2);
        assert_eq!(thumbs[0].url, "https://example.com/1.jpg");
        assert_eq!(thumbs[0].width, Some(100));
    }
}
