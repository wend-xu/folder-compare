//! App state for compare + detailed diff UI workflow.

use crate::view_models::{AnalysisResultViewModel, CompareEntryRowViewModel, DiffPanelViewModel};
use fc_ai::{AiConfig, AiProviderKind};
use std::path::Path;

const WARNING_WRAP_COLUMNS: usize = 96;
const PATH_DISPLAY_MAX_CHARS: usize = 140;
const PATH_DISPLAY_HEAD_CHARS: usize = 90;
const PATH_DISPLAY_TAIL_CHARS: usize = 45;

/// Diff tab shell state for unified status rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffShellState {
    /// No row selected in Results / Navigator.
    NoSelection,
    /// Diff or preview loading is in progress.
    Loading,
    /// Detailed diff payload is ready.
    DetailedReady,
    /// Preview payload is ready.
    PreviewReady,
    /// Selection is valid but this viewer cannot render content.
    Unavailable,
    /// Loading failed due to runtime error.
    Error,
}

/// Analysis tab shell state for unified status rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisPanelState {
    /// No row selected in Results / Navigator.
    NoSelection,
    /// One row is selected, but no analysis result is available yet.
    NotStarted,
    /// AI analysis is currently running.
    Loading,
    /// AI analysis failed in the current session.
    Error,
    /// Structured analysis result is ready.
    Success,
}

/// In-memory UI state for compare workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Status filter scope applied to compare rows (`all`, `different`, ...).
    pub entry_status_filter: String,
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
    /// Whether AI analysis can be triggered for current selection.
    pub analysis_available: bool,
    /// Whether AI analysis loading is running.
    pub analysis_loading: bool,
    /// Optional hint text for AI analysis availability.
    pub analysis_hint: Option<String>,
    /// Top-level AI analysis error.
    pub analysis_error_message: Option<String>,
    /// Structured AI analysis payload for panel rendering.
    pub analysis_result: Option<AnalysisResultViewModel>,
    /// Selected AI provider mode.
    pub analysis_provider_kind: AiProviderKind,
    /// OpenAI-compatible endpoint input.
    pub analysis_openai_endpoint: String,
    /// OpenAI-compatible API key input.
    pub analysis_openai_api_key: String,
    /// OpenAI-compatible model input.
    pub analysis_openai_model: String,
    /// OpenAI-compatible request timeout input in seconds.
    pub analysis_request_timeout_secs: u64,
    /// Provider settings dialog error message.
    pub provider_settings_error_message: Option<String>,
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
            entry_status_filter: "all".to_string(),
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
            analysis_available: false,
            analysis_loading: false,
            analysis_hint: Some("Select one changed text file to analyze.".to_string()),
            analysis_error_message: None,
            analysis_result: None,
            analysis_provider_kind: AiProviderKind::Mock,
            analysis_openai_endpoint: String::new(),
            analysis_openai_api_key: String::new(),
            analysis_openai_model: "gpt-4o-mini".to_string(),
            analysis_request_timeout_secs: 30,
            provider_settings_error_message: None,
        }
    }
}

impl AppState {
    fn selected_entry_row(&self) -> Option<&CompareEntryRowViewModel> {
        self.selected_row.and_then(|idx| self.entry_rows.get(idx))
    }

    fn selected_row_status_token(&self) -> &str {
        self.selected_entry_row()
            .map(|row| row.status.as_str())
            .unwrap_or("")
    }

    fn selected_file_type_hint(&self) -> Option<String> {
        let entry = self.selected_entry_row()?;
        if entry.entry_kind != "file" {
            return Some(format!("entry {}", entry.entry_kind));
        }

        let relative_path = self
            .selected_relative_path
            .as_deref()
            .unwrap_or_default()
            .trim();
        if relative_path.is_empty() {
            return Some("type file".to_string());
        }
        let ext = Path::new(relative_path)
            .extension()
            .and_then(|value| value.to_str());
        match ext.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => Some(format!("type .{}", value.to_ascii_lowercase())),
            None => Some("type file".to_string()),
        }
    }

    fn diff_payload_unavailable_message(&self) -> Option<String> {
        let diff = self.selected_diff.as_ref()?;
        let line_message = diff
            .hunks
            .first()
            .and_then(|hunk| hunk.lines.first())
            .map(|line| line.content.trim().to_string())
            .filter(|content| content.starts_with("[preview unavailable]"));
        if line_message.is_some() {
            return line_message;
        }
        let summary = diff.summary_text.trim();
        if summary.to_ascii_lowercase().contains("unavailable") {
            return Some(summary.to_string());
        }
        None
    }

    fn diff_status_technical_reason(&self) -> Option<String> {
        let warning = self
            .diff_warning
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        if warning.is_some() {
            return warning;
        }

        let payload_message = self.diff_payload_unavailable_message();
        if payload_message.is_some() {
            return payload_message;
        }

        self.diff_error_message
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    }

    /// Returns true when selected row is expected to be rendered in preview mode.
    pub fn diff_is_preview_mode(&self) -> bool {
        matches!(
            self.selected_row_status_token(),
            "left-only" | "right-only" | "equal"
        )
    }

    /// Returns file context mode label for Diff header.
    pub fn diff_mode_label(&self) -> String {
        if self.diff_is_preview_mode() {
            "Preview".to_string()
        } else {
            "Detailed Diff".to_string()
        }
    }

    /// Returns style tone for diff mode pill.
    pub fn diff_mode_tone(&self) -> String {
        if self.diff_is_preview_mode() {
            "info".to_string()
        } else {
            "neutral".to_string()
        }
    }

    /// Returns result status label for selected row.
    pub fn diff_result_status_label(&self) -> String {
        match self.selected_row_status_token() {
            "different" => "Changed".to_string(),
            "left-only" => "Left Only".to_string(),
            "right-only" => "Right Only".to_string(),
            "equal" => "Equal".to_string(),
            "pending" => "Pending".to_string(),
            "skipped" => "Unavailable".to_string(),
            _ => "Unavailable".to_string(),
        }
    }

    /// Returns style tone for selected row status.
    pub fn diff_result_status_tone(&self) -> String {
        match self.selected_row_status_token() {
            "different" => "different".to_string(),
            "left-only" => "left".to_string(),
            "right-only" => "right".to_string(),
            "equal" => "equal".to_string(),
            "pending" => "info".to_string(),
            "skipped" => "warn".to_string(),
            _ => "neutral".to_string(),
        }
    }

    /// Returns normalized Diff shell state for state panel and top header.
    pub fn diff_shell_state(&self) -> DiffShellState {
        if self.selected_row.is_none() {
            return DiffShellState::NoSelection;
        }
        if self.diff_loading {
            return DiffShellState::Loading;
        }
        if self
            .diff_error_message
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return DiffShellState::Error;
        }
        if self.selected_diff.is_none() {
            return DiffShellState::Unavailable;
        }
        if self.diff_payload_unavailable_message().is_some() {
            return DiffShellState::Unavailable;
        }
        if self.diff_is_preview_mode() {
            return DiffShellState::PreviewReady;
        }
        DiffShellState::DetailedReady
    }

    /// Returns short state badge text for Diff shell.
    pub fn diff_shell_state_label(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "No Selection".to_string(),
            DiffShellState::Loading => "Loading".to_string(),
            DiffShellState::DetailedReady => "Detailed Ready".to_string(),
            DiffShellState::PreviewReady => "Preview Ready".to_string(),
            DiffShellState::Unavailable => "Unavailable".to_string(),
            DiffShellState::Error => "Load Failed".to_string(),
        }
    }

    /// Returns stable token for diff shell state branching in UI layer.
    pub fn diff_shell_state_token(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "no-selection".to_string(),
            DiffShellState::Loading => "loading".to_string(),
            DiffShellState::DetailedReady => "detailed-ready".to_string(),
            DiffShellState::PreviewReady => "preview-ready".to_string(),
            DiffShellState::Unavailable => "unavailable".to_string(),
            DiffShellState::Error => "error".to_string(),
        }
    }

    /// Returns state tone for Diff shell badge.
    pub fn diff_shell_state_tone(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "neutral".to_string(),
            DiffShellState::Loading => "info".to_string(),
            DiffShellState::DetailedReady => "success".to_string(),
            DiffShellState::PreviewReady => "info".to_string(),
            DiffShellState::Unavailable => "warn".to_string(),
            DiffShellState::Error => "error".to_string(),
        }
    }

    /// Returns compact second-layer summary text for Diff header.
    pub fn diff_context_summary_text(&self) -> String {
        let diff_summary = self
            .selected_diff
            .as_ref()
            .map(|diff| diff.summary_text.trim())
            .filter(|text| !text.is_empty());
        if let Some(summary) = diff_summary {
            return abbreviate_middle(summary, 96, 64, 24);
        }

        match self.diff_shell_state() {
            DiffShellState::NoSelection => "Choose a row from Results / Navigator.".to_string(),
            DiffShellState::Loading => {
                if self.diff_is_preview_mode() {
                    "Preparing preview lines...".to_string()
                } else {
                    "Preparing detailed diff...".to_string()
                }
            }
            DiffShellState::DetailedReady => "Detailed diff ready.".to_string(),
            DiffShellState::PreviewReady => "Preview ready.".to_string(),
            DiffShellState::Unavailable => {
                if self.diff_is_preview_mode() {
                    "Preview unavailable.".to_string()
                } else {
                    "Detailed diff unavailable.".to_string()
                }
            }
            DiffShellState::Error => "Load failed.".to_string(),
        }
    }

    /// Returns third-layer weak context text for Diff header.
    pub fn diff_context_hint_text(&self) -> String {
        if self.selected_row.is_none() {
            return String::new();
        }

        let mut parts = Vec::new();
        if let Some(type_hint) = self.selected_file_type_hint() {
            parts.push(type_hint);
        }

        if self.diff_is_preview_mode() {
            let preview_reason = match self.selected_row_status_token() {
                "left-only" => "source left-only",
                "right-only" => "source right-only",
                "equal" => "source equal",
                _ => "source selected",
            };
            parts.push(preview_reason.to_string());
        }

        if self.diff_truncated {
            parts.push("truncated".to_string());
        }

        parts.join(" · ")
    }

    /// Returns title text for the unified Diff shell.
    pub fn diff_shell_title_text(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "No file selected".to_string(),
            DiffShellState::Loading => {
                if self.diff_is_preview_mode() {
                    "Loading preview".to_string()
                } else {
                    "Loading detailed diff".to_string()
                }
            }
            DiffShellState::DetailedReady => {
                if self.diff_has_rows() {
                    "Detailed diff ready".to_string()
                } else {
                    "Detailed diff has no lines".to_string()
                }
            }
            DiffShellState::PreviewReady => {
                if self.diff_has_rows() {
                    "Preview ready".to_string()
                } else {
                    "Preview has no lines".to_string()
                }
            }
            DiffShellState::Unavailable => {
                if self.diff_is_preview_mode() {
                    "Preview unavailable".to_string()
                } else {
                    "Detailed diff unavailable".to_string()
                }
            }
            DiffShellState::Error => {
                if self.diff_is_preview_mode() {
                    "Failed to load preview".to_string()
                } else {
                    "Failed to load detailed diff".to_string()
                }
            }
        }
    }

    /// Returns primary body text for the unified Diff shell.
    pub fn diff_shell_body_text(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => {
                "Choose one row from Results / Navigator to open the file-level Diff view."
                    .to_string()
            }
            DiffShellState::Loading => {
                if self.diff_is_preview_mode() {
                    "Preparing selectable preview lines for the selected file.".to_string()
                } else {
                    "Preparing hunks, line numbers, and selectable diff lines.".to_string()
                }
            }
            DiffShellState::DetailedReady => {
                if self.diff_has_rows() {
                    "Detailed diff content is ready.".to_string()
                } else {
                    "This diff has no line-level content to render.".to_string()
                }
            }
            DiffShellState::PreviewReady => {
                if self.diff_has_rows() {
                    "Preview content is ready.".to_string()
                } else {
                    "This file has no text lines to display in preview mode.".to_string()
                }
            }
            DiffShellState::Unavailable => {
                if self.diff_is_preview_mode() {
                    "This selection is valid, but the current viewer has no reviewable preview text."
                        .to_string()
                } else {
                    "This selection was compared successfully, but the current viewer cannot render a detailed text diff."
                        .to_string()
                }
            }
            DiffShellState::Error => {
                if self.diff_is_preview_mode() {
                    "The selected preview could not be loaded in this session.".to_string()
                } else {
                    "The selected detailed diff could not be loaded in this session.".to_string()
                }
            }
        }
    }

    /// Returns optional secondary note text for the unified Diff shell.
    pub fn diff_shell_note_text(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => {
                "Changed files open Detailed Diff. Left Only / Right Only / Equal entries open Preview."
                    .to_string()
            }
            DiffShellState::Loading => self.diff_context_hint_text(),
            DiffShellState::Unavailable | DiffShellState::Error => self
                .diff_status_technical_reason()
                .map(|reason| abbreviate_middle(reason.trim(), 220, 160, 52))
                .unwrap_or_else(|| self.diff_context_hint_text()),
            DiffShellState::DetailedReady | DiffShellState::PreviewReady => String::new(),
        }
    }

    /// Returns left column label for diff table.
    pub fn diff_left_column_label(&self) -> String {
        match self.selected_row_status_token() {
            "right-only" => "-".to_string(),
            "left-only" | "equal" => "left".to_string(),
            _ => "old".to_string(),
        }
    }

    /// Returns right column label for diff table.
    pub fn diff_right_column_label(&self) -> String {
        match self.selected_row_status_token() {
            "left-only" => "-".to_string(),
            "right-only" | "equal" => "right".to_string(),
            _ => "new".to_string(),
        }
    }

    /// Returns approximate character capacity used to size the scrollable diff table.
    pub fn diff_content_char_capacity(&self) -> i32 {
        let max_chars = self
            .diff_viewer_rows()
            .iter()
            .map(|row| row.content.chars().count())
            .max()
            .unwrap_or(80);
        max_chars.clamp(80, 480) as i32
    }

    /// Returns warning lines rendered as a multiline string.
    pub fn warnings_text(&self) -> String {
        if self.warning_lines.is_empty() {
            return String::new();
        }
        let mut out = Vec::new();
        for warning in &self.warning_lines {
            for (idx, part) in wrap_ui_text(warning, WARNING_WRAP_COLUMNS)
                .iter()
                .enumerate()
            {
                if idx == 0 {
                    out.push(format!("• {part}"));
                } else {
                    out.push(format!("  {part}"));
                }
            }
        }
        out.join("\n")
    }

    /// Returns filtered entry rows with their source index.
    pub fn filtered_entry_rows_with_index(&self) -> Vec<(usize, CompareEntryRowViewModel)> {
        let status_filter = normalize_status_filter_token(&self.entry_status_filter);
        self.entry_rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row.matches_filter(&self.entry_filter)
                    && status_filter_matches(&row.status, status_filter.as_str())
            })
            .map(|(index, row)| (index, row.clone()))
            .collect()
    }

    /// Updates status filter scope in canonical form.
    pub fn set_entry_status_filter(&mut self, filter: &str) {
        self.entry_status_filter = normalize_status_filter_token(filter);
    }

    /// Returns true when one source row index is currently visible by filter.
    pub fn is_row_visible_in_filter(&self, index: usize) -> bool {
        let status_filter = normalize_status_filter_token(&self.entry_status_filter);
        self.entry_rows
            .get(index)
            .map(|row| {
                row.matches_filter(&self.entry_filter)
                    && status_filter_matches(&row.status, status_filter.as_str())
            })
            .unwrap_or(false)
    }

    /// Returns collection summary text for Results / Navigator.
    pub fn results_collection_text(&self) -> String {
        let status_filter = normalize_status_filter_token(&self.entry_status_filter);
        let visible = self
            .entry_rows
            .iter()
            .filter(|row| {
                row.matches_filter(&self.entry_filter)
                    && status_filter_matches(&row.status, status_filter.as_str())
            })
            .count();
        let total =
            summary_metric_usize(&self.summary_text, "total=").unwrap_or(self.entry_rows.len());
        let query = self.entry_filter.trim();
        let mut parts = vec![format!("Showing {visible} / {total}")];
        if !query.is_empty() {
            parts.push(format!(
                "Search: \"{}\"",
                abbreviate_middle(&sanitize_inline_query(query), 30, 18, 8)
            ));
        }
        if status_filter == "all" {
            if query.is_empty() {
                parts.push("All results".to_string());
            }
        } else {
            parts.push(status_filter_label(status_filter.as_str()).to_string());
        }
        parts.join(" · ")
    }

    /// Returns compact compare summary text for sidebar status section.
    pub fn compact_summary_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }
        let mut parts = Vec::new();
        if let Some(value) = self.compare_mode_label() {
            parts.push(value);
        }
        if let Some(value) = summary_metric(&self.summary_text, "total=") {
            parts.push(format!("Total {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "different=") {
            parts.push(format!("Changed {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "left_only=") {
            parts.push(format!("Left {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "right_only=") {
            parts.push(format!("Right {value}"));
        }
        if let Some(value) = self.compare_deferred_count().filter(|value| *value > 0) {
            parts.push(format!("{value} deferred"));
        }
        if let Some(value) = self.compare_oversized_count().filter(|value| *value > 0) {
            parts.push(format!("{value} oversized"));
        }
        if self.truncated {
            parts.push("Truncated".to_string());
        }
        if parts.is_empty() {
            if self.error_message.is_some() {
                return "Compare failed".to_string();
            }
            if !self.warning_lines.is_empty() {
                return format_warning_count(self.warning_lines.len());
            }
            return abbreviate_middle(&self.summary_text, 96, 56, 36);
        }
        parts.join(" · ")
    }

    /// Returns key compare metrics in short desktop-friendly format.
    pub fn compare_metrics_text(&self) -> String {
        if self.summary_text.trim().is_empty() {
            return String::new();
        }
        let total = summary_metric(&self.summary_text, "total=").unwrap_or_else(|| "0".to_string());
        let changed =
            summary_metric(&self.summary_text, "different=").unwrap_or_else(|| "0".to_string());
        let left =
            summary_metric(&self.summary_text, "left_only=").unwrap_or_else(|| "0".to_string());
        let right =
            summary_metric(&self.summary_text, "right_only=").unwrap_or_else(|| "0".to_string());
        format!("Total {total} · Changed {changed} · Left {left} · Right {right}")
    }

    /// Returns true when compare summary indicates deferred detail entries.
    pub fn compare_has_deferred(&self) -> bool {
        summary_metric_usize(&self.summary_text, "deferred=").unwrap_or(0) > 0
    }

    /// Returns true when compare summary indicates oversized text entries.
    pub fn compare_has_oversized(&self) -> bool {
        summary_metric_usize(&self.summary_text, "oversized_text=").unwrap_or(0) > 0
    }

    /// Returns true when Compare Status has any compare report detail to expose.
    pub fn compare_status_has_detail(&self) -> bool {
        !self.summary_text.trim().is_empty()
            || !self.warning_lines.is_empty()
            || self
                .error_message
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
    }

    /// Returns one collapsed note line for Compare Status.
    pub fn compare_status_note_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }

        let mut parts = Vec::new();
        if let Some(value) = self.compare_mode_note_text() {
            parts.push(value);
        }
        if let Some(value) = self.compare_deferred_count().filter(|value| *value > 0) {
            parts.push(format!("{value} deferred"));
        }
        if let Some(value) = self.compare_oversized_count().filter(|value| *value > 0) {
            parts.push(format!("{value} oversized"));
        }
        if !self.warning_lines.is_empty() {
            parts.push(format_warning_count(self.warning_lines.len()));
        }
        if self.truncated {
            parts.push("Truncated output".to_string());
        }

        parts.join(" · ")
    }

    /// Returns concise copy-ready text for Compare Status.
    pub fn compare_summary_copy_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }

        let mut lines = vec!["Compare Summary".to_string()];
        if let Some(status) = normalize_optional_text(&self.status_text) {
            lines.push(status);
        }
        if let Some(metrics) = normalize_optional_text(&self.compare_metrics_text()) {
            lines.push(metrics);
        }
        if let Some(note) = normalize_optional_text(&self.compare_status_note_text()) {
            lines.push(note);
        }
        if let Some(error) = self
            .error_message
            .as_deref()
            .and_then(normalize_optional_text)
        {
            lines.push(format!("Error: {error}"));
        }

        lines.join("\n")
    }

    /// Returns structured copy-ready detail text for Compare Status.
    pub fn compare_detail_copy_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }

        let mut blocks = Vec::new();
        if let Some(status) = normalize_optional_text(&self.status_text) {
            blocks.push(format!("Status\n{status}"));
        }
        if let Some(metrics) = normalize_optional_text(&self.compare_metrics_text()) {
            blocks.push(format!("Results\n{metrics}"));
        }

        let mut diagnostics = Vec::new();
        if let Some(mode) = self.compare_mode_label() {
            diagnostics.push(mode);
        }
        if let Some(value) = self.compare_deferred_count().filter(|value| *value > 0) {
            diagnostics.push(format!("{value} deferred detail entries"));
        }
        if let Some(value) = self.compare_oversized_count().filter(|value| *value > 0) {
            diagnostics.push(format!("{value} oversized text entries"));
        }
        if self.truncated {
            diagnostics.push("Truncated compare output".to_string());
        }
        if !diagnostics.is_empty() {
            blocks.push(format!("Detail\n{}", diagnostics.join("\n")));
        }

        if let Some(summary) = normalize_optional_text(&self.compact_summary_text()) {
            blocks.push(format!("Summary\n{summary}"));
        }

        if !self.warning_lines.is_empty() {
            blocks.push(format!(
                "Warnings\n{}",
                self.warning_lines
                    .iter()
                    .map(|warning| format!("• {}", warning.trim()))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if let Some(error) = self
            .error_message
            .as_deref()
            .and_then(normalize_optional_text)
        {
            blocks.push(format!("Error\n{error}"));
        }

        format!("Compare Detail\n\n{}", blocks.join("\n\n"))
    }

    /// Returns selected relative path text for UI rendering.
    pub fn selected_relative_path_text(&self) -> String {
        let raw = self.selected_relative_path.clone().unwrap_or_default();
        abbreviate_middle(
            &raw,
            PATH_DISPLAY_MAX_CHARS,
            PATH_DISPLAY_HEAD_CHARS,
            PATH_DISPLAY_TAIL_CHARS,
        )
    }

    /// Returns selected compare row status token for UI rendering.
    pub fn selected_row_status_text(&self) -> String {
        self.selected_row_status_token().to_string()
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

    /// Returns true when current detailed diff has at least one rendered row.
    pub fn diff_has_rows(&self) -> bool {
        self.selected_diff
            .as_ref()
            .map(|diff| {
                !diff.hunks.is_empty() && diff.hunks.iter().any(|hunk| !hunk.lines.is_empty())
            })
            .unwrap_or(false)
    }

    /// Clears detailed diff panel state without changing compare state.
    pub fn clear_diff_panel(&mut self) {
        self.diff_loading = false;
        self.diff_error_message = None;
        self.selected_diff = None;
        self.diff_warning = None;
        self.diff_truncated = false;
    }

    /// Clears AI analysis panel state without changing compare/diff state.
    pub fn clear_analysis_panel(&mut self) {
        self.analysis_loading = false;
        self.analysis_error_message = None;
        self.analysis_result = None;
    }

    /// Returns normalized Analysis panel state for header and body rendering.
    pub fn analysis_panel_state(&self) -> AnalysisPanelState {
        if self.selected_row.is_none() {
            return AnalysisPanelState::NoSelection;
        }
        if self.analysis_loading {
            return AnalysisPanelState::Loading;
        }
        if self
            .analysis_error_message
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return AnalysisPanelState::Error;
        }
        if self.analysis_result.is_some() {
            return AnalysisPanelState::Success;
        }
        AnalysisPanelState::NotStarted
    }

    /// Returns short state badge text for Analysis shell.
    pub fn analysis_state_label(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "No Selection".to_string(),
            AnalysisPanelState::NotStarted => "Not Started".to_string(),
            AnalysisPanelState::Loading => "Analyzing".to_string(),
            AnalysisPanelState::Error => "Failed".to_string(),
            AnalysisPanelState::Success => "Ready".to_string(),
        }
    }

    /// Returns stable token for Analysis shell state branching in UI layer.
    pub fn analysis_state_token(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "no-selection".to_string(),
            AnalysisPanelState::NotStarted => "not-started".to_string(),
            AnalysisPanelState::Loading => "loading".to_string(),
            AnalysisPanelState::Error => "error".to_string(),
            AnalysisPanelState::Success => "success".to_string(),
        }
    }

    /// Returns tone for Analysis shell state surfaces.
    pub fn analysis_state_tone(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "neutral".to_string(),
            AnalysisPanelState::Loading => "info".to_string(),
            AnalysisPanelState::Error => "error".to_string(),
            AnalysisPanelState::Success => "success".to_string(),
            AnalysisPanelState::NotStarted => {
                if self.analysis_can_start_now() {
                    "neutral".to_string()
                } else {
                    "warn".to_string()
                }
            }
        }
    }

    /// Returns compact header summary text for Analysis shell.
    pub fn analysis_header_summary_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => {
                "Choose one changed text file from Results / Navigator.".to_string()
            }
            AnalysisPanelState::NotStarted => {
                if self.analysis_can_start_now() {
                    "Diff context is ready. Run Analyze to generate a review conclusion."
                        .to_string()
                } else if !self.analysis_hint_text().trim().is_empty() {
                    abbreviate_middle(&self.analysis_hint_text(), 116, 88, 22)
                } else {
                    "Analysis is not ready for this selection yet.".to_string()
                }
            }
            AnalysisPanelState::Loading => {
                "Building a structured review conclusion for the selected diff.".to_string()
            }
            AnalysisPanelState::Error => {
                "The last analysis did not complete for the current file.".to_string()
            }
            AnalysisPanelState::Success => self.analysis_summary_text(),
        }
    }

    /// Returns weak technical context text for Analysis header/helper strip.
    pub fn analysis_technical_context_text(&self) -> String {
        let mut parts = vec![
            format!("Provider {}", self.analysis_provider_mode_text()),
            if self.analysis_remote_mode() {
                if self.analysis_remote_config_ready() {
                    "remote ready".to_string()
                } else {
                    "remote config incomplete".to_string()
                }
            } else {
                "local deterministic".to_string()
            },
            format!("timeout {}s", self.analysis_timeout_text()),
        ];

        if self.diff_truncated {
            parts.push("diff context truncated".to_string());
        }

        if let Some(warning) = self
            .diff_warning
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            parts.push(abbreviate_middle(warning, 96, 72, 18));
        }

        parts.join(" · ")
    }

    /// Returns provider readiness badge label for Analysis header.
    pub fn analysis_provider_status_label(&self) -> String {
        if self.analysis_remote_mode() {
            if self.analysis_remote_config_ready() {
                "remote ready".to_string()
            } else {
                "remote config".to_string()
            }
        } else {
            "local mock".to_string()
        }
    }

    /// Returns provider readiness badge tone for Analysis header.
    pub fn analysis_provider_status_tone(&self) -> String {
        if self.analysis_remote_mode() {
            if self.analysis_remote_config_ready() {
                "info".to_string()
            } else {
                "warn".to_string()
            }
        } else {
            "neutral".to_string()
        }
    }

    /// Returns state surface title for Analysis mode.
    pub fn analysis_state_title_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "No file selected".to_string(),
            AnalysisPanelState::NotStarted => {
                if self.analysis_can_start_now() {
                    "Analysis ready to start".to_string()
                } else if self.analysis_available {
                    "Analysis is waiting for diff context".to_string()
                } else {
                    "Analysis is not ready yet".to_string()
                }
            }
            AnalysisPanelState::Loading => "Analysis in progress".to_string(),
            AnalysisPanelState::Error => "Analysis failed".to_string(),
            AnalysisPanelState::Success => "Review conclusion ready".to_string(),
        }
    }

    /// Returns state surface body text for Analysis mode.
    pub fn analysis_state_body_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => {
                "Select one row in Results / Navigator to open the file-level Analysis view."
                    .to_string()
            }
            AnalysisPanelState::NotStarted => {
                if self.analysis_can_start_now() {
                    "The selected file already has reviewable diff context. Run Analyze when you want a structured risk review."
                        .to_string()
                } else if !self.analysis_hint_text().trim().is_empty() {
                    self.analysis_hint_text()
                } else {
                    "Prepare detailed diff context before requesting analysis.".to_string()
                }
            }
            AnalysisPanelState::Loading => {
                "The provider is reviewing the current diff context and assembling summary, risk, and next-step guidance."
                    .to_string()
            }
            AnalysisPanelState::Error => {
                "A review conclusion could not be generated for the current diff context in this session."
                    .to_string()
            }
            AnalysisPanelState::Success => {
                "Summary, risk level, key points, and review suggestions are ready below."
                    .to_string()
            }
        }
    }

    /// Returns optional secondary note text for Analysis state surface.
    pub fn analysis_state_note_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => {
                "Analysis only runs for changed text files with loadable diff context.".to_string()
            }
            AnalysisPanelState::NotStarted => {
                if self.analysis_can_start_now() {
                    self.analysis_technical_context_text()
                } else if self.analysis_remote_mode() && !self.analysis_remote_config_ready() {
                    "Complete endpoint, API key, and model in Provider Settings before using the remote provider."
                        .to_string()
                } else {
                    self.analysis_technical_context_text()
                }
            }
            AnalysisPanelState::Loading => self.analysis_technical_context_text(),
            AnalysisPanelState::Error => self
                .analysis_error_message
                .as_ref()
                .map(|value| abbreviate_middle(value.trim(), 220, 168, 40))
                .unwrap_or_else(|| self.analysis_technical_context_text()),
            AnalysisPanelState::Success => self.analysis_result_notes_text(),
        }
    }

    /// Returns AI analysis hint text for UI rendering.
    pub fn analysis_hint_text(&self) -> String {
        self.analysis_hint.clone().unwrap_or_default()
    }

    /// Returns AI analysis title text for UI rendering.
    pub fn analysis_title_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.title.clone())
            .unwrap_or_default()
    }

    /// Returns AI risk level text for UI rendering.
    pub fn analysis_risk_level_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.risk_level.clone())
            .unwrap_or_default()
    }

    /// Returns AI rationale text for UI rendering.
    pub fn analysis_rationale_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.rationale.clone())
            .unwrap_or_default()
    }

    /// Returns AI key points text for UI rendering.
    pub fn analysis_key_points_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.key_points_text())
            .unwrap_or_default()
    }

    /// Returns AI review suggestions text for UI rendering.
    pub fn analysis_review_suggestions_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.review_suggestions_text())
            .unwrap_or_default()
    }

    /// Returns summary excerpt for Analysis success content.
    pub fn analysis_summary_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| {
                let title = result.title.trim();
                let rationale = result.rationale.trim();
                if !rationale.is_empty() {
                    sentence_excerpt(rationale, 168)
                } else if !title.is_empty() {
                    title.to_string()
                } else {
                    "Review summary unavailable.".to_string()
                }
            })
            .unwrap_or_default()
    }

    /// Returns primary assessment text for Analysis success content.
    pub fn analysis_core_judgment_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| {
                let rationale = result.rationale.trim();
                if !rationale.is_empty() {
                    rationale.to_string()
                } else {
                    "No core judgment was returned for this analysis.".to_string()
                }
            })
            .unwrap_or_default()
    }

    /// Returns risk badge label for Analysis success content.
    pub fn analysis_risk_label_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| format!("{} risk", title_case_token(&result.risk_level)))
            .unwrap_or_default()
    }

    /// Returns risk badge tone for Analysis success content.
    pub fn analysis_risk_tone(&self) -> String {
        match self.analysis_risk_level_text().as_str() {
            "high" => "error".to_string(),
            "medium" => "warn".to_string(),
            "low" => "success".to_string(),
            _ => "neutral".to_string(),
        }
    }

    /// Returns short risk guidance text for Analysis success content.
    pub fn analysis_risk_guidance_text(&self) -> String {
        match self.analysis_risk_level_text().as_str() {
            "high" => "Prioritize a careful review before merging.".to_string(),
            "medium" => "Review the changed logic paths and edge cases closely.".to_string(),
            "low" => {
                "No immediate high-risk signal surfaced from the current diff context.".to_string()
            }
            _ => "Risk signal unavailable for this analysis.".to_string(),
        }
    }

    /// Returns notes/annotations block for Analysis success content.
    pub fn analysis_result_notes_text(&self) -> String {
        let mut notes = Vec::new();
        if self.diff_truncated {
            notes.push("The analysis was generated from truncated diff context.".to_string());
        }
        if let Some(warning) = self
            .diff_warning
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            notes.push(warning.to_string());
        }
        if notes.is_empty() && !self.analysis_remote_mode() && self.analysis_result.is_some() {
            notes.push(
                "This result came from the deterministic mock provider for local review."
                    .to_string(),
            );
        }
        notes.join("\n")
    }

    /// Returns copy-ready text for the Summary section.
    pub fn analysis_summary_copy_text(&self) -> String {
        compose_section_copy_text(
            "Summary",
            normalize_optional_text(&self.analysis_title_text()),
            normalize_optional_text(&self.analysis_summary_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Risk Level section.
    pub fn analysis_risk_copy_text(&self) -> String {
        compose_section_copy_text(
            "Risk Level",
            normalize_optional_text(&self.analysis_risk_label_text()),
            normalize_optional_text(&self.analysis_risk_guidance_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Core Judgment section.
    pub fn analysis_core_judgment_copy_text(&self) -> String {
        compose_section_copy_text(
            "Core Judgment",
            None,
            normalize_optional_text(&self.analysis_core_judgment_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Key Points section.
    pub fn analysis_key_points_copy_text(&self) -> String {
        compose_section_copy_text(
            "Key Points",
            None,
            normalize_optional_text(&self.analysis_key_points_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Review Suggestions section.
    pub fn analysis_review_suggestions_copy_text(&self) -> String {
        compose_section_copy_text(
            "Review Suggestions",
            None,
            normalize_optional_text(&self.analysis_review_suggestions_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Notes section.
    pub fn analysis_notes_copy_text(&self) -> String {
        compose_section_copy_text(
            "Notes",
            None,
            normalize_optional_text(&self.analysis_result_notes_text()),
        )
        .unwrap_or_default()
    }

    /// Returns one copy-ready export for the current Analysis conclusion.
    pub fn analysis_full_copy_text(&self) -> String {
        let mut blocks = Vec::new();
        if let Some(path) = self
            .selected_relative_path
            .as_deref()
            .and_then(normalize_optional_text)
        {
            blocks.push(format!("File\n{path}"));
        }

        for section in [
            self.analysis_summary_copy_text(),
            self.analysis_risk_copy_text(),
            self.analysis_core_judgment_copy_text(),
            self.analysis_key_points_copy_text(),
            self.analysis_review_suggestions_copy_text(),
            self.analysis_notes_copy_text(),
        ] {
            if !section.trim().is_empty() {
                blocks.push(section);
            }
        }

        blocks.join("\n\n")
    }

    /// Returns human-readable AI provider mode.
    pub fn analysis_provider_mode_text(&self) -> String {
        match self.analysis_provider_kind {
            AiProviderKind::Mock => "Mock".to_string(),
            AiProviderKind::OpenAiCompatible => "OpenAI-compatible".to_string(),
        }
    }

    /// Returns true when remote provider mode is selected.
    pub fn analysis_remote_mode(&self) -> bool {
        self.analysis_provider_kind == AiProviderKind::OpenAiCompatible
    }

    /// Returns true when remote provider required config is complete.
    pub fn analysis_remote_config_ready(&self) -> bool {
        !self.analysis_openai_endpoint.trim().is_empty()
            && !self.analysis_openai_api_key.trim().is_empty()
            && !self.analysis_openai_model.trim().is_empty()
    }

    /// Returns request timeout text for UI rendering.
    pub fn analysis_timeout_text(&self) -> String {
        self.analysis_request_timeout_secs.to_string()
    }

    /// Returns provider settings error text for UI rendering.
    pub fn provider_settings_error_text(&self) -> String {
        self.provider_settings_error_message
            .clone()
            .unwrap_or_default()
    }

    /// Builds one AI config snapshot from current UI state.
    pub fn analysis_ai_config(&self) -> AiConfig {
        let mut config = AiConfig::default();
        config.provider_kind = self.analysis_provider_kind;
        config.openai_endpoint = normalize_optional_text(&self.analysis_openai_endpoint);
        config.openai_api_key = normalize_optional_text(&self.analysis_openai_api_key);
        config.openai_model = normalize_optional_text(&self.analysis_openai_model);
        config.request_timeout_secs = self.analysis_request_timeout_secs.max(1);
        config
    }

    fn analysis_can_start_now(&self) -> bool {
        self.selected_row.is_some()
            && self.analysis_available
            && !self.diff_loading
            && self.selected_diff.is_some()
            && (!self.analysis_remote_mode() || self.analysis_remote_config_ready())
    }

    fn compare_mode_token(&self) -> Option<String> {
        summary_metric(&self.summary_text, "mode=")
            .and_then(|value| normalize_optional_text(&value))
    }

    fn compare_mode_label(&self) -> Option<String> {
        match self.compare_mode_token()?.as_str() {
            "summary-first" => Some("Summary-first mode".to_string()),
            "large" => Some("Large mode".to_string()),
            "normal" => None,
            other => Some(title_case_token(other)),
        }
    }

    fn compare_mode_note_text(&self) -> Option<String> {
        match self.compare_mode_token()?.as_str() {
            "summary-first" => Some("Summary-first mode".to_string()),
            "large" => Some("Large-directory protection".to_string()),
            "normal" => None,
            other => Some(title_case_token(other)),
        }
    }

    fn compare_deferred_count(&self) -> Option<usize> {
        summary_metric_usize(&self.summary_text, "deferred=")
    }

    fn compare_oversized_count(&self) -> Option<usize> {
        summary_metric_usize(&self.summary_text, "oversized_text=")
    }
}

fn wrap_ui_text(text: &str, max_columns: usize) -> Vec<String> {
    if text.trim().is_empty() || max_columns == 0 {
        return vec![text.to_string()];
    }

    let mut remaining = text.trim().to_string();
    let mut out = Vec::new();
    while remaining.chars().count() > max_columns {
        let mut split_byte = None;
        let mut chars_seen = 0usize;
        for (idx, ch) in remaining.char_indices() {
            chars_seen += 1;
            if chars_seen > max_columns {
                break;
            }
            if ch.is_whitespace() || ch == '/' || ch == '\\' || ch == ',' || ch == ';' {
                split_byte = Some(idx + ch.len_utf8());
            }
        }
        let split_at = split_byte.unwrap_or_else(|| {
            remaining
                .char_indices()
                .nth(max_columns)
                .map(|(idx, _)| idx)
                .unwrap_or(remaining.len())
        });
        let (head, tail) = remaining.split_at(split_at);
        out.push(head.trim_end().to_string());
        remaining = tail.trim_start().to_string();
    }
    if !remaining.is_empty() {
        out.push(remaining);
    }
    out
}

fn abbreviate_middle(text: &str, max_chars: usize, head_chars: usize, tail_chars: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars || chars.len() <= head_chars + tail_chars + 1 {
        return text.to_string();
    }
    let head = chars[..head_chars].iter().collect::<String>();
    let tail = chars[chars.len() - tail_chars..].iter().collect::<String>();
    format!("{head}…{tail}")
}

fn normalize_optional_text(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn compose_section_copy_text(
    section_label: &str,
    title: Option<String>,
    body: Option<String>,
) -> Option<String> {
    let mut lines = vec![section_label.to_string()];
    if let Some(value) = title.and_then(|value| normalize_optional_text(&value)) {
        lines.push(value);
    }
    if let Some(value) = body.and_then(|value| normalize_optional_text(&value)) {
        lines.push(value);
    }
    (lines.len() > 1).then(|| lines.join("\n"))
}

fn sentence_excerpt(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut boundary = None;
    for (idx, ch) in trimmed.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            boundary = Some(idx + ch.len_utf8());
            break;
        }
    }

    let excerpt = boundary
        .map(|idx| trimmed[..idx].trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(trimmed);

    if excerpt.chars().count() <= max_chars {
        excerpt.to_string()
    } else {
        abbreviate_middle(excerpt, max_chars, max_chars.saturating_sub(20), 18)
    }
}

fn title_case_token(token: &str) -> String {
    let mut chars = token.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn normalize_status_filter_token(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "all" => "all".to_string(),
        "different" => "different".to_string(),
        "equal" => "equal".to_string(),
        "left-only" => "left-only".to_string(),
        "right-only" => "right-only".to_string(),
        _ => "all".to_string(),
    }
}

fn status_filter_matches(status: &str, filter: &str) -> bool {
    filter == "all" || status.eq_ignore_ascii_case(filter)
}

fn status_filter_label(filter: &str) -> &'static str {
    match filter {
        "different" => "Diff",
        "equal" => "Equal",
        "left-only" => "Left only",
        "right-only" => "Right only",
        _ => "All results",
    }
}

fn sanitize_inline_query(query: &str) -> String {
    query
        .trim()
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('"', "'")
}

fn format_warning_count(count: usize) -> String {
    match count {
        0 => String::new(),
        1 => "1 warning".to_string(),
        value => format!("{value} warnings"),
    }
}

fn summary_metric(summary_text: &str, key: &str) -> Option<String> {
    summary_text
        .split_whitespace()
        .find_map(|part| part.trim_matches('|').strip_prefix(key))
        .map(|value| value.trim_matches('|').to_string())
}

fn summary_metric_usize(summary_text: &str, key: &str) -> Option<usize> {
    summary_metric(summary_text, key).and_then(|value| value.parse::<usize>().ok())
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
                can_load_analysis: true,
                analysis_blocked_reason: None,
            },
            CompareEntryRowViewModel {
                relative_path: "assets/logo.png".to_string(),
                status: "different".to_string(),
                detail: "file compare: left=10B right=12B".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "file-comparison".to_string(),
                can_load_diff: false,
                diff_blocked_reason: Some("binary candidate".to_string()),
                can_load_analysis: false,
                analysis_blocked_reason: Some("binary candidate".to_string()),
            },
        ]
    }

    fn sample_preview_panel(summary: &str, content: &str) -> DiffPanelViewModel {
        DiffPanelViewModel {
            relative_path: "assets/preview.js".to_string(),
            summary_text: summary.to_string(),
            hunks: vec![crate::view_models::DiffHunkViewModel {
                old_start: 1,
                old_len: 1,
                new_start: 1,
                new_len: 0,
                lines: vec![crate::view_models::DiffLineViewModel {
                    old_line_no: Some(1),
                    new_line_no: None,
                    kind: "Context".to_string(),
                    content: content.to_string(),
                }],
            }],
            warning: None,
            truncated: false,
        }
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
    fn status_filter_reduces_visible_rows() {
        let mut rows = sample_rows();
        rows.push(CompareEntryRowViewModel {
            relative_path: "docs/guide.md".to_string(),
            status: "equal".to_string(),
            detail: "metadata equal".to_string(),
            entry_kind: "file".to_string(),
            detail_kind: "file-comparison".to_string(),
            can_load_diff: false,
            diff_blocked_reason: Some("not changed".to_string()),
            can_load_analysis: false,
            analysis_blocked_reason: Some("not changed".to_string()),
        });
        let state = AppState {
            entry_rows: rows,
            entry_status_filter: "equal".to_string(),
            ..AppState::default()
        };
        let filtered = state.filtered_entry_rows_with_index();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.status, "equal");
    }

    #[test]
    fn invalid_status_filter_falls_back_to_all() {
        let mut state = AppState::default();
        state.set_entry_status_filter("unexpected-status");
        assert_eq!(state.entry_status_filter, "all");
    }

    #[test]
    fn results_collection_text_tracks_search_and_scope() {
        let mut state = AppState {
            entry_rows: sample_rows(),
            ..AppState::default()
        };
        assert_eq!(
            state.results_collection_text(),
            "Showing 2 / 2 · All results"
        );

        state.entry_filter = "logo".to_string();
        state.set_entry_status_filter("different");
        let text = state.results_collection_text();
        assert_eq!(text, "Showing 1 / 2 · Search: \"logo\" · Diff");
    }

    #[test]
    fn compact_summary_text_extracts_key_metrics() {
        let state = AppState {
            summary_text: "mode=normal total=120 equal=100 different=8 left_only=7 right_only=5 pending=0 skipped=0 deferred=3 oversized_text=2".to_string(),
            truncated: true,
            ..AppState::default()
        };
        let text = state.compact_summary_text();
        assert!(text.contains("Total 120"));
        assert!(text.contains("Changed 8"));
        assert!(text.contains("Left 7"));
        assert!(text.contains("Right 5"));
        assert!(text.contains("3 deferred"));
        assert!(text.contains("2 oversized"));
        assert!(text.contains("Truncated"));
    }

    #[test]
    fn compare_metrics_text_formats_core_counts() {
        let state = AppState {
            summary_text: "mode=normal total=42 equal=35 different=4 left_only=2 right_only=1 pending=0 skipped=0 deferred=0 oversized_text=0".to_string(),
            ..AppState::default()
        };
        assert_eq!(
            state.compare_metrics_text(),
            "Total 42 · Changed 4 · Left 2 · Right 1"
        );
    }

    #[test]
    fn compare_copy_texts_are_summary_first_and_structured() {
        let state = AppState {
            status_text: "Compare finished: 42 entries".to_string(),
            summary_text: "mode=summary-first total=42 equal=35 different=4 left_only=2 right_only=1 pending=0 skipped=0 deferred=3 oversized_text=1".to_string(),
            warning_lines: vec!["large-directory guard applied".to_string()],
            truncated: true,
            ..AppState::default()
        };

        let summary = state.compare_summary_copy_text();
        assert!(summary.contains("Compare Summary"));
        assert!(summary.contains("Compare finished: 42 entries"));
        assert!(summary.contains("Total 42 · Changed 4 · Left 2 · Right 1"));
        assert!(summary.contains(
            "Summary-first mode · 3 deferred · 1 oversized · 1 warning · Truncated output"
        ));

        let detail = state.compare_detail_copy_text();
        assert!(detail.contains("Compare Detail"));
        assert!(detail.contains("Status\nCompare finished: 42 entries"));
        assert!(detail.contains("Results\nTotal 42 · Changed 4 · Left 2 · Right 1"));
        assert!(detail.contains("Detail\nSummary-first mode"));
        assert!(detail.contains("3 deferred detail entries"));
        assert!(detail.contains("Warnings\n• large-directory guard applied"));
    }

    #[test]
    fn compare_flags_reflect_summary_metrics() {
        let state = AppState {
            summary_text: "mode=normal total=6 equal=2 different=1 left_only=1 right_only=2 pending=0 skipped=0 deferred=2 oversized_text=1".to_string(),
            ..AppState::default()
        };
        assert!(state.compare_has_deferred());
        assert!(state.compare_has_oversized());
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

    #[test]
    fn warnings_text_wraps_long_lines_for_ui() {
        let state = AppState {
            warning_lines: vec![
                "large directory guard: entries=20000 total_bytes=3221225472 hard_entries=50000 hard_total_bytes=2147483648".to_string(),
            ],
            ..AppState::default()
        };
        let text = state.warnings_text();
        assert!(text.contains("• "));
        assert!(text.contains('\n'));
        assert!(text.contains("entries=20000"));
    }

    #[test]
    fn selected_relative_path_is_abbreviated_when_too_long() {
        let long_path = format!("{}/{}", "a".repeat(120), "b".repeat(120));
        let state = AppState {
            selected_relative_path: Some(long_path),
            ..AppState::default()
        };
        let display = state.selected_relative_path_text();
        assert!(display.contains('…'));
        assert!(display.len() < 200);
    }

    #[test]
    fn diff_shell_state_tracks_no_selection_loading_and_ready() {
        let mut state = AppState::default();
        assert_eq!(state.diff_shell_state(), DiffShellState::NoSelection);

        state.selected_row = Some(0);
        state.entry_rows = sample_rows();
        state.diff_loading = true;
        assert_eq!(state.diff_shell_state(), DiffShellState::Loading);

        state.diff_loading = false;
        state.selected_diff = Some(DiffPanelViewModel {
            relative_path: "src/main.rs".to_string(),
            summary_text: "hunks=1 +2 -1 ctx=3".to_string(),
            hunks: vec![crate::view_models::DiffHunkViewModel {
                old_start: 1,
                old_len: 1,
                new_start: 1,
                new_len: 1,
                lines: vec![crate::view_models::DiffLineViewModel {
                    old_line_no: Some(1),
                    new_line_no: Some(1),
                    kind: "Added".to_string(),
                    content: "line".to_string(),
                }],
            }],
            warning: None,
            truncated: false,
        });
        assert_eq!(state.diff_shell_state(), DiffShellState::DetailedReady);
    }

    #[test]
    fn diff_shell_state_marks_preview_and_unavailable() {
        let mut state = AppState {
            selected_row: Some(0),
            entry_rows: vec![CompareEntryRowViewModel {
                relative_path: "assets/p.js".to_string(),
                status: "left-only".to_string(),
                detail: "only on left".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "none".to_string(),
                can_load_diff: true,
                diff_blocked_reason: None,
                can_load_analysis: false,
                analysis_blocked_reason: Some("not changed".to_string()),
            }],
            selected_relative_path: Some("assets/p.js".to_string()),
            ..AppState::default()
        };

        state.selected_diff = Some(sample_preview_panel("left-only preview lines=4", "line"));
        assert_eq!(state.diff_shell_state(), DiffShellState::PreviewReady);
        assert!(state.diff_context_hint_text().contains("source left-only"));

        state.selected_diff = Some(sample_preview_panel(
            "single-side preview unavailable",
            "[preview unavailable] binary content is not supported",
        ));
        assert_eq!(state.diff_shell_state(), DiffShellState::Unavailable);
    }

    #[test]
    fn diff_context_header_fields_use_status_specific_labels() {
        let state = AppState {
            selected_row: Some(0),
            entry_rows: vec![CompareEntryRowViewModel {
                relative_path: "docs/readme.md".to_string(),
                status: "equal".to_string(),
                detail: "equal".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "none".to_string(),
                can_load_diff: true,
                diff_blocked_reason: None,
                can_load_analysis: false,
                analysis_blocked_reason: Some("not changed".to_string()),
            }],
            selected_relative_path: Some("docs/readme.md".to_string()),
            selected_diff: Some(sample_preview_panel("equal preview lines=10", "line")),
            ..AppState::default()
        };

        assert_eq!(state.diff_mode_label(), "Preview");
        assert_eq!(state.diff_result_status_label(), "Equal");
        assert_eq!(state.diff_left_column_label(), "left");
        assert_eq!(state.diff_right_column_label(), "right");
        assert!(state.diff_context_hint_text().contains("type .md"));
    }

    #[test]
    fn clear_analysis_panel_resets_loading_error_and_result() {
        let mut state = AppState {
            analysis_loading: true,
            analysis_error_message: Some("error".to_string()),
            analysis_result: Some(AnalysisResultViewModel {
                title: "title".to_string(),
                risk_level: "low".to_string(),
                rationale: "ok".to_string(),
                key_points: vec!["k".to_string()],
                review_suggestions: vec!["s".to_string()],
            }),
            ..AppState::default()
        };
        state.clear_analysis_panel();
        assert!(!state.analysis_loading);
        assert!(state.analysis_error_message.is_none());
        assert!(state.analysis_result.is_none());
    }

    #[test]
    fn analysis_panel_state_distinguishes_no_selection_not_started_and_success() {
        let mut state = AppState::default();
        assert_eq!(
            state.analysis_panel_state(),
            AnalysisPanelState::NoSelection
        );

        state.selected_row = Some(0);
        state.entry_rows = sample_rows();
        assert_eq!(state.analysis_panel_state(), AnalysisPanelState::NotStarted);

        state.analysis_available = true;
        state.selected_diff = Some(sample_preview_panel("preview", "line"));
        assert_eq!(state.analysis_state_title_text(), "Analysis ready to start");

        state.analysis_result = Some(AnalysisResultViewModel {
            title: "Risk review for src/main.rs".to_string(),
            risk_level: "medium".to_string(),
            rationale: "The change touches branching logic and should be reviewed carefully."
                .to_string(),
            key_points: vec!["Branching changed".to_string()],
            review_suggestions: vec!["Add coverage".to_string()],
        });
        assert_eq!(state.analysis_panel_state(), AnalysisPanelState::Success);
    }

    #[test]
    fn analysis_result_notes_include_truncation_and_warning() {
        let state = AppState {
            selected_row: Some(0),
            entry_rows: sample_rows(),
            analysis_result: Some(AnalysisResultViewModel {
                title: "Risk review".to_string(),
                risk_level: "high".to_string(),
                rationale: "This change updates error handling. It also introduces unwrap."
                    .to_string(),
                key_points: vec!["unwrap added".to_string()],
                review_suggestions: vec!["Check panic paths".to_string()],
            }),
            diff_truncated: true,
            diff_warning: Some("input excerpt trimmed to fit provider limit".to_string()),
            ..AppState::default()
        };

        assert_eq!(state.analysis_risk_tone(), "error");
        assert!(
            state
                .analysis_summary_text()
                .starts_with("This change updates")
        );
        assert!(
            state
                .analysis_result_notes_text()
                .contains("truncated diff context")
        );
        assert!(
            state
                .analysis_result_notes_text()
                .contains("input excerpt trimmed")
        );
    }

    #[test]
    fn analysis_copy_text_exports_structured_sections() {
        let state = AppState {
            selected_row: Some(0),
            entry_rows: sample_rows(),
            selected_relative_path: Some("src/main.rs".to_string()),
            analysis_result: Some(AnalysisResultViewModel {
                title: "Regression risk in startup path".to_string(),
                risk_level: "high".to_string(),
                rationale: "The patch removes validation and shifts initialization order."
                    .to_string(),
                key_points: vec![
                    "Validation branch deleted".to_string(),
                    "Startup sequencing changed".to_string(),
                ],
                review_suggestions: vec!["Re-run startup coverage".to_string()],
            }),
            diff_warning: Some("context excerpt trimmed".to_string()),
            ..AppState::default()
        };

        assert!(
            state
                .analysis_summary_copy_text()
                .starts_with("Summary\nRegression risk")
        );
        assert!(state.analysis_risk_copy_text().contains("High risk"));
        assert!(
            state
                .analysis_full_copy_text()
                .contains("File\nsrc/main.rs")
        );
        assert!(
            state
                .analysis_full_copy_text()
                .contains("Review Suggestions")
        );
        assert!(
            state
                .analysis_full_copy_text()
                .contains("context excerpt trimmed")
        );
    }

    #[test]
    fn remote_config_ready_requires_endpoint_key_and_model() {
        let mut state = AppState {
            analysis_provider_kind: AiProviderKind::OpenAiCompatible,
            ..AppState::default()
        };
        assert!(!state.analysis_remote_config_ready());

        state.analysis_openai_endpoint = "http://localhost:11434/v1".to_string();
        assert!(!state.analysis_remote_config_ready());
        state.analysis_openai_api_key = "token".to_string();
        assert!(state.analysis_remote_config_ready());
    }

    #[test]
    fn analysis_ai_config_reflects_provider_fields() {
        let state = AppState {
            analysis_provider_kind: AiProviderKind::OpenAiCompatible,
            analysis_openai_endpoint: " http://localhost:11434/v1 ".to_string(),
            analysis_openai_api_key: " sk-test ".to_string(),
            analysis_openai_model: " qwen2.5-coder ".to_string(),
            analysis_request_timeout_secs: 42,
            ..AppState::default()
        };
        let config = state.analysis_ai_config();
        assert_eq!(config.provider_kind, AiProviderKind::OpenAiCompatible);
        assert_eq!(
            config.openai_endpoint.as_deref(),
            Some("http://localhost:11434/v1")
        );
        assert_eq!(config.openai_api_key.as_deref(), Some("sk-test"));
        assert_eq!(config.openai_model.as_deref(), Some("qwen2.5-coder"));
        assert_eq!(config.request_timeout_secs, 42);
    }
}
