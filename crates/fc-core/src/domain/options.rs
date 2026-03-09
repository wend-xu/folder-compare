//! Request and options models.

use crate::domain::error::{CompareError, InvalidInputKind, PathSide};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_CONTEXT_LINES: usize = 10_000;

/// Compare directories request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompareRequest {
    /// Left root directory.
    pub left_root: PathBuf,
    /// Right root directory.
    pub right_root: PathBuf,
    /// Compare behavior options.
    pub options: CompareOptions,
}

impl CompareRequest {
    /// Creates a compare request from roots and options.
    pub fn new(left_root: PathBuf, right_root: PathBuf, options: CompareOptions) -> Self {
        Self {
            left_root,
            right_root,
            options,
        }
    }

    /// Validates request shape and option bounds.
    pub fn validate(&self) -> Result<(), CompareError> {
        if self.left_root.as_os_str().is_empty() {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::EmptyRootPath {
                    side: PathSide::Left,
                },
            });
        }
        if self.right_root.as_os_str().is_empty() {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::EmptyRootPath {
                    side: PathSide::Right,
                },
            });
        }

        self.options.validate()
    }
}

/// Options for directory compare behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompareOptions {
    /// Hash algorithm used for file fingerprinting.
    pub hash_algorithm: HashAlgorithm,
    /// Strategy used to detect text files.
    pub text_detection: TextDetectionStrategy,
    /// Whitespace handling mode for text compare.
    pub ignore_whitespace: IgnoreWhitespaceMode,
    /// Whether symlink traversal is enabled.
    pub follow_symlinks: bool,
    /// Whether line ending differences are ignored.
    pub ignore_line_endings: bool,
    /// Policy to apply for large directories.
    pub large_dir_policy: LargeDirPolicy,
    /// Maximum entries allowed before policy may trigger.
    pub max_entries: usize,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            hash_algorithm: HashAlgorithm::Sha256,
            text_detection: TextDetectionStrategy::ExtensionHeuristic,
            ignore_whitespace: IgnoreWhitespaceMode::Preserve,
            follow_symlinks: false,
            ignore_line_endings: false,
            large_dir_policy: LargeDirPolicy::SummaryFirst,
            max_entries: 10_000,
        }
    }
}

impl CompareOptions {
    /// Validates option bounds.
    pub fn validate(&self) -> Result<(), CompareError> {
        if self.max_entries == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_entries",
                },
            });
        }

        Ok(())
    }
}

/// Request for diffing two text files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextDiffRequest {
    /// Left-side text file path.
    pub left_path: PathBuf,
    /// Right-side text file path.
    pub right_path: PathBuf,
    /// Diff behavior options.
    pub options: TextDiffOptions,
}

impl TextDiffRequest {
    /// Creates a text diff request from file paths and options.
    pub fn new(left_path: PathBuf, right_path: PathBuf, options: TextDiffOptions) -> Self {
        Self {
            left_path,
            right_path,
            options,
        }
    }

    /// Validates request shape and option bounds.
    pub fn validate(&self) -> Result<(), CompareError> {
        if self.left_path.as_os_str().is_empty() {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::EmptyFilePath {
                    side: PathSide::Left,
                },
            });
        }
        if self.right_path.as_os_str().is_empty() {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::EmptyFilePath {
                    side: PathSide::Right,
                },
            });
        }

        self.options.validate()
    }
}

/// Options for text diff behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextDiffOptions {
    /// Whitespace handling mode.
    pub ignore_whitespace: IgnoreWhitespaceMode,
    /// Whether line ending differences are ignored.
    pub ignore_line_endings: bool,
    /// Number of context lines around changes.
    pub context_lines: usize,
}

impl Default for TextDiffOptions {
    fn default() -> Self {
        Self {
            ignore_whitespace: IgnoreWhitespaceMode::Preserve,
            ignore_line_endings: false,
            context_lines: 3,
        }
    }
}

impl TextDiffOptions {
    /// Validates option bounds.
    pub fn validate(&self) -> Result<(), CompareError> {
        if self.context_lines > MAX_CONTEXT_LINES {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "context_lines",
                },
            });
        }

        Ok(())
    }
}

/// Hashing algorithm options for file content identity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum HashAlgorithm {
    /// Disable hashing.
    None,
    /// SHA-256 digest.
    #[default]
    Sha256,
}

/// Strategy for deciding whether a file should be treated as text.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TextDetectionStrategy {
    /// Use file extension heuristics.
    #[default]
    ExtensionHeuristic,
    /// Use simple byte sampling heuristics.
    ContentHeuristic,
}

/// Whitespace handling behavior for text comparison.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum IgnoreWhitespaceMode {
    /// Keep whitespace as-is.
    #[default]
    Preserve,
    /// Trim leading and trailing whitespace.
    TrimEdges,
    /// Normalize all consecutive spaces and tabs.
    NormalizeRuns,
}

/// Policy for oversized directory trees.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LargeDirPolicy {
    /// Return summary-level information first when size thresholds are reached.
    #[default]
    SummaryFirst,
    /// Continue processing.
    Allow,
    /// Abort with an error.
    Error,
}
