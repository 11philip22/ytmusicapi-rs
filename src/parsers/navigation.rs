//! Navigation path constants matching the Python library.
//!
//! These paths correspond to the nested JSON structure of YouTube Music API responses.

use crate::nav::PathSegment;

/// Commonly used navigation paths as static slices.
#[allow(dead_code)]
pub mod paths {
    use super::PathSegment;

    pub const CONTENT: &[PathSegment] = &[PathSegment::Key("contents"), PathSegment::Index(0)];

    pub const RUN_TEXT: &[PathSegment] = &[
        PathSegment::Key("runs"),
        PathSegment::Index(0),
        PathSegment::Key("text"),
    ];

    pub const TAB_CONTENT: &[PathSegment] = &[
        PathSegment::Key("tabs"),
        PathSegment::Index(0),
        PathSegment::Key("tabRenderer"),
        PathSegment::Key("content"),
    ];

    pub const TWO_COLUMN_RENDERER: &[PathSegment] = &[
        PathSegment::Key("contents"),
        PathSegment::Key("twoColumnBrowseResultsRenderer"),
    ];

    pub const SINGLE_COLUMN: &[PathSegment] = &[
        PathSegment::Key("contents"),
        PathSegment::Key("singleColumnBrowseResultsRenderer"),
    ];

    pub const SECTION_LIST: &[PathSegment] = &[
        PathSegment::Key("sectionListRenderer"),
        PathSegment::Key("contents"),
    ];

    pub const MUSIC_SHELF: &[PathSegment] = &[PathSegment::Key("musicShelfRenderer")];

    pub const GRID: &[PathSegment] = &[PathSegment::Key("gridRenderer")];

    pub const GRID_ITEMS: &[PathSegment] =
        &[PathSegment::Key("gridRenderer"), PathSegment::Key("items")];

    pub const MENU_ITEMS: &[PathSegment] = &[
        PathSegment::Key("menu"),
        PathSegment::Key("menuRenderer"),
        PathSegment::Key("items"),
    ];

    pub const THUMBNAIL: &[PathSegment] = &[
        PathSegment::Key("thumbnail"),
        PathSegment::Key("thumbnails"),
    ];

    pub const THUMBNAILS: &[PathSegment] = &[
        PathSegment::Key("thumbnail"),
        PathSegment::Key("musicThumbnailRenderer"),
        PathSegment::Key("thumbnail"),
        PathSegment::Key("thumbnails"),
    ];

    pub const TITLE_TEXT: &[PathSegment] = &[
        PathSegment::Key("title"),
        PathSegment::Key("runs"),
        PathSegment::Index(0),
        PathSegment::Key("text"),
    ];

    pub const SUBTITLE_RUNS: &[PathSegment] =
        &[PathSegment::Key("subtitle"), PathSegment::Key("runs")];

    pub const NAVIGATION_BROWSE_ID: &[PathSegment] = &[
        PathSegment::Key("navigationEndpoint"),
        PathSegment::Key("browseEndpoint"),
        PathSegment::Key("browseId"),
    ];

    pub const NAVIGATION_PLAYLIST_ID: &[PathSegment] = &[
        PathSegment::Key("navigationEndpoint"),
        PathSegment::Key("watchEndpoint"),
        PathSegment::Key("playlistId"),
    ];

    pub const MRLIR: &str = "musicResponsiveListItemRenderer";
    pub const MTRIR: &str = "musicTwoRowItemRenderer";

    pub const RESPONSIVE_HEADER: &[PathSegment] =
        &[PathSegment::Key("musicResponsiveHeaderRenderer")];

    pub const EDITABLE_PLAYLIST_DETAIL_HEADER: &[PathSegment] = &[PathSegment::Key(
        "musicEditablePlaylistDetailHeaderRenderer",
    )];

    pub const HEADER: &[PathSegment] = &[PathSegment::Key("header")];

    pub const HEADER_DETAIL: &[PathSegment] = &[
        PathSegment::Key("header"),
        PathSegment::Key("musicDetailHeaderRenderer"),
    ];

    pub const DESCRIPTION_SHELF: &[PathSegment] =
        &[PathSegment::Key("musicDescriptionShelfRenderer")];

    pub const PLAY_BUTTON: &[PathSegment] = &[
        PathSegment::Key("overlay"),
        PathSegment::Key("musicItemThumbnailOverlayRenderer"),
        PathSegment::Key("content"),
        PathSegment::Key("musicPlayButtonRenderer"),
    ];

    pub const BADGE_LABEL: &[PathSegment] = &[
        PathSegment::Key("badges"),
        PathSegment::Index(0),
        PathSegment::Key("musicInlineBadgeRenderer"),
        PathSegment::Key("accessibilityData"),
        PathSegment::Key("accessibilityData"),
        PathSegment::Key("label"),
    ];

    /// Continuation token path in results
    pub const CONTINUATION_TOKEN: &[PathSegment] = &[
        PathSegment::Key("continuationItemRenderer"),
        PathSegment::Key("continuationEndpoint"),
        PathSegment::Key("continuationCommand"),
        PathSegment::Key("token"),
    ];
}
