//! Text diff models.

use serde::{Deserialize, Serialize};

/// Request output model for a text diff operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TextDiffResult {
    /// Aggregate summary for the diff.
    pub summary: TextDiffSummary,
    /// Ordered list of diff hunks.
    pub hunks: Vec<DiffHunk>,
    /// Whether output was truncated by configured limits.
    pub truncated: bool,
    /// Optional warning message for truncated/limited output.
    pub warning: Option<String>,
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
    /// Starting line number in old text (1-based).
    pub old_start: usize,
    /// Number of old lines covered by this hunk.
    pub old_len: usize,
    /// Starting line number in new text (1-based).
    pub new_start: usize,
    /// Number of new lines covered by this hunk.
    pub new_len: usize,
    /// Diff lines in display order.
    pub lines: Vec<DiffLine>,
}

/// One line in a diff hunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiffLine {
    /// Semantic line kind.
    pub kind: DiffLineKind,
    /// Old-side line number, if applicable.
    pub old_line_no: Option<usize>,
    /// New-side line number, if applicable.
    pub new_line_no: Option<usize>,
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
