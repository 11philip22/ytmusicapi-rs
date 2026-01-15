//! Request context and headers for YouTube Music API.

use serde_json::{json, Value};

/// YouTube Music domain
pub const YTM_DOMAIN: &str = "https://music.youtube.com";

/// YouTube Music API base URL
pub const YTM_BASE_API: &str = "https://music.youtube.com/youtubei/v1/";

/// Default API params
pub const YTM_PARAMS: &str = "?alt=json";

/// API key for browser auth (public, used in web client)
pub const YTM_PARAMS_KEY: &str = "&key=AIzaSyC9XL3ZjWddXya6X74dJoCTL-WEYFDNX30";

/// User agent matching the Python library
pub const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0";

/// Create the request context body that YouTube Music requires.
pub fn create_context(language: &str, location: Option<&str>, user: Option<&str>) -> Value {
    let client_version = format!("1.{}.01.00", chrono::Utc::now().format("%Y%m%d"));

    let mut context = json!({
        "context": {
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": client_version,
                "hl": language,
            },
            "user": {}
        }
    });

    if let Some(loc) = location {
        context["context"]["client"]["gl"] = json!(loc);
    }

    if let Some(u) = user {
        context["context"]["user"]["onBehalfOfUser"] = json!(u);
    }

    context
}

/// Default headers for requests
pub fn default_headers() -> Vec<(&'static str, String)> {
    vec![
        ("user-agent", USER_AGENT.to_string()),
        ("accept", "*/*".to_string()),
        ("accept-encoding", "gzip, deflate".to_string()),
        ("content-type", "application/json".to_string()),
        ("origin", YTM_DOMAIN.to_string()),
    ]
}
