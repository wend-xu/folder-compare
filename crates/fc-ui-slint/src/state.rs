//! App state for compare + detailed diff UI workflow.

use crate::view_models::{CompareEntryRowViewModel, DiffPanelViewModel};

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
    /// Filter text applied to compare rows.
    pub entry_filter: String,
    /// Warning lines from compare report.
    pub warning_lines: Vec<String>,
    /// Top-level compare error message.
    pub error_message: Option<String>,
    /// Whether current report is truncated.
    pub truncated: bool,
    /// Optional selected row index.
    pub selected_row: Option<usize>,
    /// Whether detailed diff loading is running.
    pub diff_loading: bool,
    /// Top-level detailed diff error.
    pub diff_error_message: Option<String>,
    /// Relative path from current selected row.
    pub selected_relative_path: Option<String>,
    /// Structured detailed diff panel payload.
    pub selected_diff: Option<DiffPanelViewModel>,
    /// Optional warning from detailed diff result.
    pub diff_warning: Option<String>,
    /// Whether selected detailed diff is truncated.
    pub diff_truncated: bool,
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
            entry_filter: String::new(),
            warning_lines: Vec::new(),
            error_message: None,
            truncated: false,
            selected_row: None,
            diff_loading: false,
            diff_error_message: None,
            selected_relative_path: None,
            selected_diff: None,
            diff_warning: None,
            diff_truncated: false,
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

    /// Returns filtered entry rows with their source index.
    pub fn filtered_entry_rows_with_index(&self) -> Vec<(usize, CompareEntryRowViewModel)> {
        self.entry_rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.matches_filter(&self.entry_filter))
            .map(|(index, row)| (index, row.clone()))
            .collect()
    }

    /// Returns true when one source row index is currently visible by filter.
    pub fn is_row_visible_in_filter(&self, index: usize) -> bool {
        self.entry_rows
            .get(index)
            .map(|row| row.matches_filter(&self.entry_filter))
            .unwrap_or(false)
    }

    /// Returns filter stats text for UI header.
    pub fn filter_stats_text(&self) -> String {
        let visible = self.filtered_entry_rows_with_index().len();
        let total = self.entry_rows.len();
        if self.entry_filter.trim().is_empty() {
            return format!("Showing all entries: {total}");
        }
        format!(
            "Filtered: {visible}/{total} (query: {})",
            self.entry_filter.trim()
        )
    }

    /// Returns selected relative path text for UI rendering.
    pub fn selected_relative_path_text(&self) -> String {
        self.selected_relative_path.clone().unwrap_or_default()
    }

    /// Returns detailed diff warning text for UI rendering.
    pub fn diff_warning_text(&self) -> String {
        self.diff_warning.clone().unwrap_or_default()
    }

    /// Returns flattened detailed diff rows for viewer rendering.
    pub fn diff_viewer_rows(&self) -> Vec<DiffViewerRow> {
        let mut out = Vec::new();
        let Some(diff) = &self.selected_diff else {
            return out;
        };

        for hunk in &diff.hunks {
            out.push(DiffViewerRow {
                old_line_no: String::new(),
                new_line_no: String::new(),
                marker: "@@".to_string(),
                content: hunk.header(),
                row_kind: "hunk".to_string(),
            });
            for line in &hunk.lines {
                out.push(DiffViewerRow {
                    old_line_no: line
                        .old_line_no
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    new_line_no: line
                        .new_line_no
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    marker: line.marker().to_string(),
                    content: line.content.clone(),
                    row_kind: line.kind_tag().to_string(),
                });
            }
        }

        out
    }

    /// Clears detailed diff panel state without changing compare state.
    pub fn clear_diff_panel(&mut self) {
        self.diff_loading = false;
        self.diff_error_message = None;
        self.selected_diff = None;
        self.diff_warning = None;
        self.diff_truncated = false;
    }
}

/// One flattened row displayed in the unified diff viewer list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffViewerRow {
    /// Old-side line number text.
    pub old_line_no: String,
    /// New-side line number text.
    pub new_line_no: String,
    /// Unified diff marker (`+`, `-`, ` `, `@@`).
    pub marker: String,
    /// Row content text.
    pub content: String,
    /// Row style kind (`hunk`, `added`, `removed`, `context`).
    pub row_kind: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rows() -> Vec<CompareEntryRowViewModel> {
        vec![
            CompareEntryRowViewModel {
                relative_path: "src/main.rs".to_string(),
                status: "different".to_string(),
                detail: "text summary".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "text-diff".to_string(),
                can_load_diff: true,
                diff_blocked_reason: None,
            },
            CompareEntryRowViewModel {
                relative_path: "assets/logo.png".to_string(),
                status: "different".to_string(),
                detail: "file compare: left=10B right=12B".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "file-comparison".to_string(),
                can_load_diff: false,
                diff_blocked_reason: Some("binary candidate".to_string()),
            },
        ]
    }

    #[test]
    fn empty_filter_returns_all_rows() {
        let state = AppState {
            entry_rows: sample_rows(),
            ..AppState::default()
        };
        let filtered = state.filtered_entry_rows_with_index();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].0, 0);
        assert_eq!(filtered[1].0, 1);
    }

    #[test]
    fn non_empty_filter_matches_path_or_detail() {
        let state = AppState {
            entry_rows: sample_rows(),
            entry_filter: "logo".to_string(),
            ..AppState::default()
        };
        let filtered = state.filtered_entry_rows_with_index();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, 1);

        let state = AppState {
            entry_rows: sample_rows(),
            entry_filter: "text summary".to_string(),
            ..AppState::default()
        };
        let filtered = state.filtered_entry_rows_with_index();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, 0);
    }

    #[test]
    fn filtering_does_not_mutate_underlying_rows() {
        let rows = sample_rows();
        let state = AppState {
            entry_rows: rows.clone(),
            entry_filter: "logo".to_string(),
            ..AppState::default()
        };
        let _ = state.filtered_entry_rows_with_index();
        assert_eq!(state.entry_rows, rows);
    }
}
