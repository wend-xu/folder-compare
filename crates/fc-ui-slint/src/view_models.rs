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
