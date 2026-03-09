//! Entry-level comparison models.

use crate::domain::diff::TextDiffSummary;
use serde::{Deserialize, Serialize};

/// A single compared path in the final report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompareEntry {
    /// Path relative to compare roots.
    pub relative_path: String,
    /// Basic category of the path.
    pub kind: EntryKind,
    /// Comparison status for the path.
    pub status: EntryStatus,
    /// Optional details for the status.
    pub detail: EntryDetail,
}

impl CompareEntry {
    /// Creates an entry with `EntryDetail::None`.
    pub fn new(relative_path: impl Into<String>, kind: EntryKind, status: EntryStatus) -> Self {
        Self {
            relative_path: relative_path.into(),
            kind,
            status,
            detail: EntryDetail::None,
        }
    }

    /// Replaces detail payload and returns the updated entry.
    pub fn with_detail(mut self, detail: EntryDetail) -> Self {
        self.detail = detail;
        self
    }
}

/// High-level status of one compare entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryStatus {
    /// Entry only exists in left side.
    LeftOnly,
    /// Entry only exists in right side.
    RightOnly,
    /// Entry exists on both sides and is considered equal.
    Equal,
    /// Entry exists on both sides and differs.
    Different,
    /// Entry is aligned but content-level comparison is deferred.
    Pending,
    /// Entry was skipped by policy.
    Skipped,
}

/// Kind of file-system entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
    /// Anything else.
    Other,
}

/// Optional detail payload attached to a compare entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryDetail {
    /// No extra details.
    None,
    /// A human-readable detail message.
    Message(String),
    /// Left and right entry kinds do not match.
    TypeMismatch { left: EntryKind, right: EntryKind },
    /// File-level comparison summary.
    FileComparison {
        /// File size from left side.
        left_size: u64,
        /// File size from right side.
        right_size: u64,
        /// Whether byte-level content comparison was executed.
        content_checked: bool,
    },
    /// Content comparison is deferred to later phases.
    ContentComparisonDeferred,
    /// Text detail is intentionally deferred with a structured reason.
    TextDetailDeferred {
        /// Why text detail was deferred.
        reason: TextDetailDeferredReason,
        /// File size from left side.
        left_size: u64,
        /// File size from right side.
        right_size: u64,
        /// Configured text-size limit used by compare.
        max_text_file_size_bytes: u64,
        /// Whether byte-level content comparison was executed.
        content_checked: bool,
    },
    /// Summary from text diff stage.
    TextDiff(TextDiffSummary),
}

/// Reason why text detail was deferred in directory compare output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextDetailDeferredReason {
    /// Deferred because compare is in large-directory summary-first protection mode.
    LargeDirectoryMode,
    /// Deferred because file size exceeds `max_text_file_size_bytes`.
    FileTooLarge,
}
