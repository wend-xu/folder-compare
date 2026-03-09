//! App state for the Phase 9 MVP UI.

use crate::view_models::CompareEntryRowViewModel;

/// In-memory UI state for compare workflow.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Left compare root path.
    pub left_root: String,
    /// Right compare root path.
    pub right_root: String,
    /// Whether compare command is currently running.
    pub running: bool,
    /// Plain status text for rendering.
    pub status_text: String,
    /// Summary text derived from compare result.
    pub summary_text: String,
    /// Result rows for list rendering.
    pub entry_rows: Vec<CompareEntryRowViewModel>,
    /// Warning lines from compare report.
    pub warning_lines: Vec<String>,
    /// Top-level compare error message.
    pub error_message: Option<String>,
    /// Whether current report is truncated.
    pub truncated: bool,
    /// Optional selected row index.
    pub selected_row: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            left_root: String::new(),
            right_root: String::new(),
            running: false,
            status_text: "Ready".to_string(),
            summary_text: String::new(),
            entry_rows: Vec::new(),
            warning_lines: Vec::new(),
            error_message: None,
            truncated: false,
            selected_row: None,
        }
    }
}

impl AppState {
    /// Returns warning lines rendered as a multiline string.
    pub fn warnings_text(&self) -> String {
        if self.warning_lines.is_empty() {
            return String::new();
        }
        self.warning_lines
            .iter()
            .map(|line| format!("• {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns entry rows rendered as display lines.
    pub fn entry_display_lines(&self) -> Vec<String> {
        self.entry_rows
            .iter()
            .map(CompareEntryRowViewModel::display_line)
            .collect()
    }
}
