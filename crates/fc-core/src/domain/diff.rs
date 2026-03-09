//! Text diff models.

use serde::{Deserialize, Serialize};

/// Request output model for a text diff operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TextDiffResult {
    /// Aggregate summary for the diff.
    pub summary: TextDiffSummary,
    /// Ordered list of diff hunks.
    pub hunks: Vec<DiffHunk>,
}

impl TextDiffResult {
    /// Creates an empty diff result.
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Statistics extracted from a text diff result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TextDiffSummary {
    /// Number of hunks produced.
    pub hunk_count: usize,
    /// Number of added lines.
    pub added_lines: usize,
    /// Number of removed lines.
    pub removed_lines: usize,
    /// Number of context lines.
    pub context_lines: usize,
}

impl TextDiffSummary {
    /// Returns an empty diff summary.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns true when no additions or removals are present.
    pub fn is_equal(&self) -> bool {
        self.added_lines == 0 && self.removed_lines == 0
    }
}

/// One contiguous change block in a diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiffHunk {
    /// Header text such as range metadata.
    pub header: String,
    /// Diff lines in display order.
    pub lines: Vec<DiffLine>,
}

/// One line in a diff hunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiffLine {
    /// Semantic line kind.
    pub kind: DiffLineKind,
    /// Original line content.
    pub content: String,
}

/// Kind of a line in unified diff format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DiffLineKind {
    /// Added line.
    Added,
    /// Removed line.
    Removed,
    /// Unchanged context line.
    #[default]
    Context,
}
