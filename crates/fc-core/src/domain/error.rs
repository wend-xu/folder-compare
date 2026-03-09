//! Error model for comparison operations.

use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// Error type used by `fc-core` public APIs.
#[derive(Debug, Error)]
pub enum CompareError {
    /// Input payload is invalid.
    #[error("invalid input: {kind}")]
    InvalidInput {
        /// Structured invalid input reason.
        kind: InvalidInputKind,
    },
    /// A root path is invalid.
    #[error("invalid root path on {side}: `{path}`")]
    InvalidRootPath {
        /// Path side in request.
        side: PathSide,
        /// Root path value.
        path: PathBuf,
    },
    /// Root path does not exist.
    #[error("root path does not exist on {side}: `{path}`")]
    RootPathNotFound {
        /// Path side in request.
        side: PathSide,
        /// Missing root path.
        path: PathBuf,
    },
    /// Root path exists but is not a directory.
    #[error("root path is not a directory on {side}: `{path}`")]
    RootPathNotDirectory {
        /// Path side in request.
        side: PathSide,
        /// Non-directory root path.
        path: PathBuf,
    },
    /// Left and right roots resolve to the same location.
    #[error("left and right root paths are identical after normalization: `{path}`")]
    SameRootPathNotAllowed {
        /// Normalized path detected on both sides.
        path: PathBuf,
    },
    /// Path normalization failed before compare.
    #[error("path normalization failed for `{path}`: {reason}")]
    PathNormalizationFailed {
        /// Input path that failed normalization.
        path: PathBuf,
        /// Failure reason.
        reason: String,
    },
    /// Text path cannot be used for current file.
    #[error("text path unavailable for `{path}`: {reason}")]
    TextPathUnavailable {
        /// File path where text processing failed.
        path: PathBuf,
        /// Why text path cannot continue.
        reason: TextPathUnavailableReason,
    },
    /// I/O boundary error.
    #[error("io boundary error during {operation} at `{path}`: {source}")]
    IoBoundary {
        /// I/O operation category.
        operation: IoOperation,
        /// Path involved in the operation.
        path: PathBuf,
        /// Original I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Requested behavior is unsupported.
    #[error("unsupported operation: {operation}")]
    UnsupportedOperation {
        /// Unsupported operation kind.
        operation: UnsupportedOperation,
    },
    /// Operation intentionally deferred to a later phase.
    #[error("deferred in current phase: {operation}")]
    Deferred {
        /// Deferred operation kind.
        operation: DeferredOperation,
    },
    /// Internal unexpected error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Side of a compare request path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSide {
    /// Left-side path.
    Left,
    /// Right-side path.
    Right,
}

impl fmt::Display for PathSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Left => "left",
            Self::Right => "right",
        };
        write!(f, "{text}")
    }
}

/// Structured invalid-input reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidInputKind {
    /// A root path is empty.
    EmptyRootPath {
        /// Which side has an empty path.
        side: PathSide,
    },
    /// A text diff file path is empty.
    EmptyFilePath {
        /// Which side has an empty path.
        side: PathSide,
    },
    /// A numeric option is out of accepted range.
    OptionOutOfRange {
        /// Option field name.
        name: &'static str,
    },
}

impl fmt::Display for InvalidInputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRootPath { side } => write!(f, "empty root path on {side}"),
            Self::EmptyFilePath { side } => write!(f, "empty file path on {side}"),
            Self::OptionOutOfRange { name } => write!(f, "option `{name}` is out of range"),
        }
    }
}

/// I/O operation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation {
    /// Metadata lookup.
    Stat,
    /// Directory listing.
    ReadDir,
    /// File read.
    ReadFile,
}

impl fmt::Display for IoOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Stat => "stat",
            Self::ReadDir => "read_dir",
            Self::ReadFile => "read_file",
        };
        write!(f, "{text}")
    }
}

/// Unsupported operation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedOperation {
    /// Special file compare.
    SpecialFileCompare,
    /// Symlink target compare.
    SymlinkTargetCompare,
}

impl fmt::Display for UnsupportedOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::SpecialFileCompare => "special file compare",
            Self::SymlinkTargetCompare => "symlink target compare",
        };
        write!(f, "{text}")
    }
}

/// Deferred operation kind in current phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredOperation {
    /// Directory scanning stage.
    DirectoryScan,
    /// Path alignment stage.
    PathAlignment,
    /// File-level compare stage.
    FileComparison,
    /// Text diff algorithm stage.
    TextDiffAlgorithm,
    /// Large directory guard execution stage.
    LargeDirectoryGuard,
}

impl fmt::Display for DeferredOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::DirectoryScan => "directory scan",
            Self::PathAlignment => "path alignment",
            Self::FileComparison => "file comparison",
            Self::TextDiffAlgorithm => "text diff algorithm",
            Self::LargeDirectoryGuard => "large directory guard",
        };
        write!(f, "{text}")
    }
}

/// Reason why text loading/diff path is unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextPathUnavailableReason {
    /// Input is not considered a text candidate.
    NotTextCandidate,
    /// Input looked like text but decoding failed.
    DecodeFailed,
}

impl fmt::Display for TextPathUnavailableReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NotTextCandidate => "not a text candidate",
            Self::DecodeFailed => "text decode failed",
        };
        write!(f, "{text}")
    }
}
