//! Request and options models.

use crate::domain::error::{CompareError, InvalidInputKind, PathSide};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_CONTEXT_LINES: usize = 10_000;
const DEFAULT_MAX_ENTRIES_SOFT_LIMIT: usize = 10_000;
const DEFAULT_MAX_ENTRIES_HARD_LIMIT: usize = 50_000;
const DEFAULT_MAX_TOTAL_BYTES_SOFT_LIMIT: u64 = 512 * 1024 * 1024;
const DEFAULT_MAX_TOTAL_BYTES_HARD_LIMIT: u64 = 2 * 1024 * 1024 * 1024;
const DEFAULT_MAX_TEXT_FILE_SIZE_BYTES: u64 = 8 * 1024 * 1024;
const DEFAULT_MAX_DIFF_FILE_SIZE_BYTES: u64 = 32 * 1024 * 1024;

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
    /// Soft limit for aligned entry count.
    pub max_entries_soft_limit: usize,
    /// Hard limit for aligned entry count.
    pub max_entries_hard_limit: usize,
    /// Soft limit for total bytes across both trees.
    pub max_total_bytes_soft_limit: u64,
    /// Hard limit for total bytes across both trees.
    pub max_total_bytes_hard_limit: u64,
    /// Maximum file size eligible for summary-level text diff in `compare_dirs`.
    pub max_text_file_size_bytes: u64,
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
            max_entries_soft_limit: DEFAULT_MAX_ENTRIES_SOFT_LIMIT,
            max_entries_hard_limit: DEFAULT_MAX_ENTRIES_HARD_LIMIT,
            max_total_bytes_soft_limit: DEFAULT_MAX_TOTAL_BYTES_SOFT_LIMIT,
            max_total_bytes_hard_limit: DEFAULT_MAX_TOTAL_BYTES_HARD_LIMIT,
            max_text_file_size_bytes: DEFAULT_MAX_TEXT_FILE_SIZE_BYTES,
        }
    }
}

impl CompareOptions {
    /// Validates option bounds.
    pub fn validate(&self) -> Result<(), CompareError> {
        if self.max_entries_soft_limit == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_entries_soft_limit",
                },
            });
        }
        if self.max_entries_hard_limit == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_entries_hard_limit",
                },
            });
        }
        if self.max_entries_soft_limit > self.max_entries_hard_limit {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_entries_soft_limit",
                },
            });
        }
        if self.max_total_bytes_soft_limit == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_total_bytes_soft_limit",
                },
            });
        }
        if self.max_total_bytes_hard_limit == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_total_bytes_hard_limit",
                },
            });
        }
        if self.max_total_bytes_soft_limit > self.max_total_bytes_hard_limit {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_total_bytes_soft_limit",
                },
            });
        }
        if self.max_text_file_size_bytes == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_text_file_size_bytes",
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
    /// Strategy used to decide whether a file can be treated as text.
    pub text_detection: TextDetectionStrategy,
    /// Number of context lines around changes.
    pub context_lines: usize,
    /// Maximum hunks to include in output.
    pub max_hunks: usize,
    /// Maximum diff lines to include in output.
    pub max_lines: usize,
    /// Maximum single input file size allowed for detailed text diff.
    pub max_file_size_bytes: u64,
}

impl Default for TextDiffOptions {
    fn default() -> Self {
        Self {
            ignore_whitespace: IgnoreWhitespaceMode::Preserve,
            ignore_line_endings: false,
            text_detection: TextDetectionStrategy::ExtensionHeuristic,
            context_lines: 3,
            max_hunks: 128,
            max_lines: 20_000,
            max_file_size_bytes: DEFAULT_MAX_DIFF_FILE_SIZE_BYTES,
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
        if self.max_hunks == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange { name: "max_hunks" },
            });
        }
        if self.max_lines == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange { name: "max_lines" },
            });
        }
        if self.max_file_size_bytes == 0 {
            return Err(CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "max_file_size_bytes",
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
    /// Continue full compare flow even when hard limits are reached.
    Normal,
    /// Return summary-level information first when size thresholds are reached.
    #[default]
    SummaryFirst,
    /// Refuse compare when hard limits are exceeded.
    RefuseAboveHardLimit,
}
