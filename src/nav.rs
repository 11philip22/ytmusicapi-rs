//! JSON navigation helpers.
//!
//! Provides utilities for navigating nested JSON structures using path-like syntax.

use serde_json::Value;

/// A segment in a navigation path.
#[derive(Debug, Clone)]
pub enum PathSegment {
    /// Access an object key
    Key(&'static str),
    /// Access an array index
    Index(usize),
}

impl From<&'static str> for PathSegment {
    fn from(s: &'static str) -> Self {
        PathSegment::Key(s)
    }
}

impl From<usize> for PathSegment {
    fn from(i: usize) -> Self {
        PathSegment::Index(i)
    }
}

/// Navigate a JSON value using a path of segments.
///
/// Returns `None` if any segment in the path is not found.
///
/// # Example
///
/// ```ignore
/// use serde_json::json;
/// use ytmusicapi::nav::{nav, PathSegment};
///
/// let data = json!({"foo": [{"bar": "baz"}]});
/// let result = nav(&data, &["foo".into(), 0.into(), "bar".into()]);
/// assert_eq!(result.and_then(|v| v.as_str()), Some("baz"));
/// ```
pub fn nav<'a>(root: &'a Value, path: &[PathSegment]) -> Option<&'a Value> {
    let mut current = root;

    for segment in path {
        current = match segment {
            PathSegment::Key(key) => current.get(key)?,
            PathSegment::Index(idx) => current.get(idx)?,
        };
    }

    Some(current)
}

/// Navigate and return as a string.
pub fn nav_str<'a>(root: &'a Value, path: &[PathSegment]) -> Option<&'a str> {
    nav(root, path).and_then(|v| v.as_str())
}

/// Navigate and return as i64.
#[allow(dead_code)]
pub fn nav_i64(root: &Value, path: &[PathSegment]) -> Option<i64> {
    nav(root, path).and_then(|v| v.as_i64())
}

/// Navigate and return as u64.
#[allow(dead_code)]
pub fn nav_u64(root: &Value, path: &[PathSegment]) -> Option<u64> {
    nav(root, path).and_then(|v| v.as_u64())
}

/// Navigate and return as array.
pub fn nav_array<'a>(root: &'a Value, path: &[PathSegment]) -> Option<&'a Vec<Value>> {
    nav(root, path).and_then(|v| v.as_array())
}

/// Navigate and return as bool.
#[allow(dead_code)]
pub fn nav_bool(root: &Value, path: &[PathSegment]) -> Option<bool> {
    nav(root, path).and_then(|v| v.as_bool())
}

/// Macro for creating a path from mixed key/index values.
///
/// # Example
///
/// ```ignore
/// use ytmusicapi::path;
/// use ytmusicapi::nav::PathSegment;
///
/// let p: Vec<PathSegment> = path!["contents", 0, "title"];
/// ```
#[macro_export]
macro_rules! path {
    ($($segment:expr),* $(,)?) => {
        vec![$($crate::nav::PathSegment::from($segment)),*]
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_nav_simple() {
        let data = json!({"foo": "bar"});
        assert_eq!(nav_str(&data, &path!["foo"]), Some("bar"));
    }

    #[test]
    fn test_nav_nested() {
        let data = json!({"a": {"b": {"c": 123}}});
        assert_eq!(nav_i64(&data, &path!["a", "b", "c"]), Some(123));
    }

    #[test]
    fn test_nav_array() {
        let data = json!({"items": [{"name": "first"}, {"name": "second"}]});
        assert_eq!(nav_str(&data, &path!["items", 1, "name"]), Some("second"));
    }

    #[test]
    fn test_nav_missing() {
        let data = json!({"foo": "bar"});
        assert_eq!(nav(&data, &path!["missing"]), None);
    }
}
