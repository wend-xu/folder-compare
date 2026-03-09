//! Error model for AI analysis layer.

use thiserror::Error;

/// Error type for `fc-ai` APIs.
#[derive(Debug, Error)]
pub enum AiError {
    /// Request is invalid.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// Provider returned an execution failure.
    #[error("provider error: {0}")]
    Provider(String),
    /// Feature is intentionally not implemented yet.
    #[error("not implemented: {0}")]
    NotImplemented(String),
}
