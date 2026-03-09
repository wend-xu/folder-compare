//! View model definitions for Slint binding.

/// One compare entry row rendered by the MVP list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareEntryRowViewModel {
    /// Relative path from compare roots.
    pub relative_path: String,
    /// Human-readable entry status.
    pub status: String,
    /// Human-readable detail summary.
    pub detail: String,
    /// Entry kind from compare result (`file`, `directory`, ...).
    pub entry_kind: String,
    /// Detail payload kind (`text-diff`, `type-mismatch`, ...).
    pub detail_kind: String,
    /// Whether this row can trigger detailed text diff.
    pub can_load_diff: bool,
    /// Why this row cannot trigger detailed diff (if any).
    pub diff_blocked_reason: Option<String>,
}

impl CompareEntryRowViewModel {
    /// Renders one compact row line for list display.
    pub fn display_line(&self) -> String {
        format!("[{}] {} — {}", self.status, self.relative_path, self.detail)
    }
}

/// Compare result view model projected from `fc-core::CompareReport`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareResultViewModel {
    /// Summary text rendered in the header area.
    pub summary_text: String,
    /// Flat result rows for list rendering.
    pub entry_rows: Vec<CompareEntryRowViewModel>,
    /// Warning lines to show in warning area.
    pub warnings: Vec<String>,
    /// Indicates core report is truncated.
    pub truncated: bool,
}

/// Detailed diff panel payload for one selected row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffPanelViewModel {
    /// Relative path currently shown in the panel.
    pub relative_path: String,
    /// Human-readable summary for this detailed diff.
    pub summary_text: String,
    /// Detailed hunk list for line-level rendering.
    pub hunks: Vec<DiffHunkViewModel>,
    /// Optional warning emitted by `diff_text_file`.
    pub warning: Option<String>,
    /// Whether detailed diff output was truncated.
    pub truncated: bool,
}

impl DiffPanelViewModel {
    /// Returns a flat display list for basic panel rendering.
    pub fn display_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        for hunk in &self.hunks {
            out.push(hunk.header());
            out.extend(hunk.lines.iter().map(DiffLineViewModel::display_line));
        }
        out
    }
}

/// One hunk in detailed diff panel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffHunkViewModel {
    /// Starting line number in old text (1-based).
    pub old_start: usize,
    /// Number of old lines covered by this hunk.
    pub old_len: usize,
    /// Starting line number in new text (1-based).
    pub new_start: usize,
    /// Number of new lines covered by this hunk.
    pub new_len: usize,
    /// Ordered lines within this hunk.
    pub lines: Vec<DiffLineViewModel>,
}

impl DiffHunkViewModel {
    /// Returns one unified-style hunk header.
    pub fn header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_len, self.new_start, self.new_len
        )
    }
}

/// One detailed diff line in panel output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffLineViewModel {
    /// Old-side line number if present.
    pub old_line_no: Option<usize>,
    /// New-side line number if present.
    pub new_line_no: Option<usize>,
    /// Kind label (`Context`, `Added`, `Removed`).
    pub kind: String,
    /// Line content.
    pub content: String,
}

impl DiffLineViewModel {
    /// Returns one compact line for list display.
    pub fn display_line(&self) -> String {
        let old_no = self
            .old_line_no
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        let new_no = self
            .new_line_no
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        format!("{old_no:>6} {new_no:>6} {:<7} {}", self.kind, self.content)
    }
}
