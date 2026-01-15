//! Error types for the YouTube Music API client.

/// The error type for YouTube Music API operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing failed
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication is required for this operation
    #[error("Authentication required for this operation")]
    AuthRequired,

    /// Server returned an error
    #[error("Server error {status}: {message}")]
    Server {
        /// HTTP status code
        status: u16,
        /// Error message from server
        message: String,
    },

    /// Failed to navigate JSON response
    #[error("Navigation error: could not find path '{path}'")]
    Navigation {
        /// The path that could not be found
        path: String,
    },

    /// Invalid authentication data
    #[error("Invalid auth: {0}")]
    InvalidAuth(String),

    /// I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A specialized Result type for YouTube Music API operations.
pub type Result<T> = std::result::Result<T, Error>;
