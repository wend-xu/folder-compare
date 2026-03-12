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
            return Some(format!("Entry: {}", entry.entry_kind));
        }

        let relative_path = self
            .selected_relative_path
            .as_deref()
            .unwrap_or_default()
            .trim();
        if relative_path.is_empty() {
            return Some("Type: file".to_string());
        }
        let ext = Path::new(relative_path)
            .extension()
            .and_then(|value| value.to_str());
        match ext.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => Some(format!("Type: .{}", value.to_ascii_lowercase())),
            None => Some("Type: file".to_string()),
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
            return summary.to_string();
        }

        match self.diff_shell_state() {
            DiffShellState::NoSelection => {
                "Select one row from Results / Navigator to start.".to_string()
            }
            DiffShellState::Loading => {
                if self.diff_is_preview_mode() {
                    "Preparing preview content...".to_string()
                } else {
                    "Preparing hunk-level diff content...".to_string()
                }
            }
            DiffShellState::DetailedReady => "Detailed diff is ready.".to_string(),
            DiffShellState::PreviewReady => "Preview content is ready.".to_string(),
            DiffShellState::Unavailable => {
                if self.diff_is_preview_mode() {
                    "Preview content is unavailable for this selection.".to_string()
                } else {
                    "Detailed diff is unavailable for this selection.".to_string()
                }
            }
            DiffShellState::Error => {
                if self.diff_is_preview_mode() {
                    "Failed to load preview content.".to_string()
                } else {
                    "Failed to load detailed diff content.".to_string()
                }
            }
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
                "left-only" => "Preview source: left side only",
                "right-only" => "Preview source: right side only",
                "equal" => "Preview source: shared equal content",
                _ => "Preview source: selected row",
            };
            parts.push(preview_reason.to_string());
        }

        if self.diff_truncated {
            parts.push("Content is truncated for readability.".to_string());
        }

        if let Some(reason) = self.diff_status_technical_reason() {
            parts.push(abbreviate_middle(reason.trim(), 148, 104, 36));
        }

        parts.join(" | ")
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

    /// Returns filter stats text for UI header.
    pub fn filter_stats_text(&self) -> String {
        let status_filter = normalize_status_filter_token(&self.entry_status_filter);
        let visible = self
            .entry_rows
            .iter()
            .filter(|row| {
                row.matches_filter(&self.entry_filter)
                    && status_filter_matches(&row.status, status_filter.as_str())
            })
            .count();
        let total = self.entry_rows.len();
        let query = self.entry_filter.trim();
        let query_text = if query.is_empty() {
            "—".to_string()
        } else {
            abbreviate_middle(query, 28, 16, 8)
        };
        let status_text = match status_filter.as_str() {
            "all" => "All",
            "different" => "Different",
            "equal" => "Equal",
            "left-only" => "Left-only",
            "right-only" => "Right-only",
            _ => "All",
        };
        format!("Visible {visible}/{total} | Search: {query_text} | Status: {status_text}")
    }

    /// Returns compact compare summary text for sidebar status section.
    pub fn compact_summary_text(&self) -> String {
        if self.summary_text.trim().is_empty() {
            return "No compare summary yet.".to_string();
        }
        let mut parts = Vec::new();
        if let Some(value) = summary_metric(&self.summary_text, "mode=") {
            parts.push(format!("mode {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "total=") {
            parts.push(format!("total {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "different=") {
            parts.push(format!("diff {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "left_only=") {
            parts.push(format!("left {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "right_only=") {
            parts.push(format!("right {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "deferred=") {
            parts.push(format!("deferred {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "oversized_text=") {
            parts.push(format!("oversized {value}"));
        }
        if self.truncated {
            parts.push("truncated".to_string());
        }
        if parts.is_empty() {
            return abbreviate_middle(&self.summary_text, 96, 56, 36);
        }
        parts.join(" | ")
    }

    /// Returns key compare metrics in short desktop-friendly format.
    pub fn compare_metrics_text(&self) -> String {
        if self.summary_text.trim().is_empty() {
            return "total 0 | changed 0 | left 0 | right 0".to_string();
        }
        let total = summary_metric(&self.summary_text, "total=").unwrap_or_else(|| "0".to_string());
        let changed =
            summary_metric(&self.summary_text, "different=").unwrap_or_else(|| "0".to_string());
        let left =
            summary_metric(&self.summary_text, "left_only=").unwrap_or_else(|| "0".to_string());
        let right =
            summary_metric(&self.summary_text, "right_only=").unwrap_or_else(|| "0".to_string());
        format!("total {total} | changed {changed} | left {left} | right {right}")
    }

    /// Returns true when compare summary indicates deferred detail entries.
    pub fn compare_has_deferred(&self) -> bool {
        summary_metric_usize(&self.summary_text, "deferred=").unwrap_or(0) > 0
    }

    /// Returns true when compare summary indicates oversized text entries.
    pub fn compare_has_oversized(&self) -> bool {
        summary_metric_usize(&self.summary_text, "oversized_text=").unwrap_or(0) > 0
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
    fn filter_stats_text_is_consistent_across_scopes() {
        let mut state = AppState {
            entry_rows: sample_rows(),
            ..AppState::default()
        };
        assert_eq!(
            state.filter_stats_text(),
            "Visible 2/2 | Search: — | Status: All"
        );

        state.entry_filter = "logo".to_string();
        state.set_entry_status_filter("different");
        let text = state.filter_stats_text();
        assert!(text.starts_with("Visible 1/2 | Search: logo | Status: Different"));
    }

    #[test]
    fn compact_summary_text_extracts_key_metrics() {
        let state = AppState {
            summary_text: "mode=normal total=120 equal=100 different=8 left_only=7 right_only=5 pending=0 skipped=0 deferred=3 oversized_text=2".to_string(),
            truncated: true,
            ..AppState::default()
        };
        let text = state.compact_summary_text();
        assert!(text.contains("mode normal"));
        assert!(text.contains("total 120"));
        assert!(text.contains("diff 8"));
        assert!(text.contains("left 7"));
        assert!(text.contains("right 5"));
        assert!(text.contains("deferred 3"));
        assert!(text.contains("oversized 2"));
        assert!(text.contains("truncated"));
    }

    #[test]
    fn compare_metrics_text_formats_core_counts() {
        let state = AppState {
            summary_text: "mode=normal total=42 equal=35 different=4 left_only=2 right_only=1 pending=0 skipped=0 deferred=0 oversized_text=0".to_string(),
            ..AppState::default()
        };
        assert_eq!(
            state.compare_metrics_text(),
            "total 42 | changed 4 | left 2 | right 1"
        );
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
        assert!(state
            .diff_context_hint_text()
            .contains("Preview source: left side only"));

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
        assert!(state.diff_context_hint_text().contains("Type: .md"));
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
