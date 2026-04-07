//! Bridge between UI event handlers and presenter logic.

use crate::commands::UiCommand;
use crate::compare_foundation::CompareFoundation;
use crate::presenter::Presenter;
use crate::state::AppState;
use crate::view_models::{
    AnalysisResultViewModel, CompareEntryRowViewModel, CompareResultViewModel, DiffHunkViewModel,
    DiffLineViewModel, DiffPanelViewModel,
};
use fc_ai::{
    AiConfig, AnalysisTask, AnalyzeDiffRequest, AnalyzeDiffResponse, RiskLevel as AiRiskLevel,
};
#[cfg(test)]
use fc_core::{CompareEntry, EntryDetail, EntryKind, EntryStatus, TextDetailDeferredReason};
use fc_core::{
    CompareOptions, CompareReport, CompareRequest, DiffLineKind, TextDiffOptions, TextDiffRequest,
    TextDiffResult, TextDiffSummary,
};
use std::fs;
use std::path::PathBuf;

const SINGLE_SIDE_PREVIEW_MAX_BYTES: usize = 512 * 1024;
const SINGLE_SIDE_PREVIEW_MAX_LINES: usize = 1600;

/// Thin bridge for wiring command dispatch.
#[derive(Clone)]
pub struct UiBridge {
    presenter: Presenter,
}

impl UiBridge {
    /// Creates a new bridge.
    pub fn new(presenter: Presenter) -> Self {
        Self { presenter }
    }

    /// Dispatches a command to the presenter.
    pub fn dispatch(&self, command: UiCommand) {
        self.presenter.handle_command(command);
    }

    /// Returns the latest presenter state snapshot.
    pub fn snapshot(&self) -> AppState {
        self.presenter.state_snapshot()
    }
}

/// Builds a compare request from raw UI path inputs.
pub fn build_compare_request(left_root: &str, right_root: &str) -> Result<CompareRequest, String> {
    let left = validate_compare_root("Left", left_root)?;
    let right = validate_compare_root("Right", right_root)?;

    Ok(CompareRequest::new(left, right, CompareOptions::default()))
}

fn validate_compare_root(label: &str, raw_path: &str) -> Result<PathBuf, String> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} path is empty. Select a folder first."));
    }

    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(format!("{label} path does not exist: {}", path.display()));
    }

    let metadata = std::fs::metadata(&path).map_err(|err| {
        format!(
            "{label} path cannot be accessed: {} ({err})",
            path.display()
        )
    })?;
    if !metadata.is_dir() {
        return Err(format!("{label} path must be a folder: {}", path.display()));
    }

    std::fs::read_dir(&path)
        .map_err(|err| format!("{label} folder cannot be read: {} ({err})", path.display()))?;

    Ok(path)
}

/// Maps core compare report into UI-facing view model.
pub fn map_compare_report(report: CompareReport) -> CompareResultViewModel {
    let summary = &report.summary;
    let mode = if summary.summary_first_mode {
        "summary-first"
    } else if summary.large_mode {
        "large"
    } else {
        "normal"
    };
    let mut summary_text = format!(
        "mode={mode} total={} equal={} different={} left_only={} right_only={} pending={} skipped={} deferred={} oversized_text={}",
        summary.total_entries,
        summary.equal,
        summary.different,
        summary.left_only,
        summary.right_only,
        summary.pending,
        summary.skipped,
        summary.deferred_detail_entries,
        summary.oversized_text_entries
    );
    if report.truncated {
        summary_text.push_str(" | truncated=true");
    }

    let compare_foundation = CompareFoundation::from_compare_entries(&report.entries);
    let entry_rows = compare_foundation.project_legacy_entry_rows();

    CompareResultViewModel {
        summary_text,
        compare_foundation,
        entry_rows,
        warnings: report.warnings,
        truncated: report.truncated,
    }
}

/// Builds a text diff request from selected compare row and compare roots.
pub fn build_text_diff_request(
    left_root: &str,
    right_root: &str,
    row: &CompareEntryRowViewModel,
) -> Result<TextDiffRequest, String> {
    let left = left_root.trim();
    let right = right_root.trim();
    if left.is_empty() {
        return Err("left root path is required".to_string());
    }
    if right.is_empty() {
        return Err("right root path is required".to_string());
    }
    if !row.can_load_diff {
        return Err(row
            .diff_blocked_reason
            .clone()
            .unwrap_or_else(|| "selected row does not support detailed text diff".to_string()));
    }

    Ok(TextDiffRequest::new(
        PathBuf::from(left).join(&row.relative_path),
        PathBuf::from(right).join(&row.relative_path),
        TextDiffOptions::default(),
    ))
}

/// Maps `TextDiffResult` into panel-ready view model.
pub fn map_text_diff_result(relative_path: &str, result: TextDiffResult) -> DiffPanelViewModel {
    let summary = &result.summary;
    let summary_text = format!(
        "hunks={} +{} -{} ctx={}",
        summary.hunk_count, summary.added_lines, summary.removed_lines, summary.context_lines
    );
    let hunks = result
        .hunks
        .into_iter()
        .map(|hunk| DiffHunkViewModel {
            old_start: hunk.old_start,
            old_len: hunk.old_len,
            new_start: hunk.new_start,
            new_len: hunk.new_len,
            lines: hunk
                .lines
                .into_iter()
                .map(|line| DiffLineViewModel {
                    old_line_no: line.old_line_no,
                    new_line_no: line.new_line_no,
                    kind: diff_line_kind_text(line.kind).to_string(),
                    content: line.content,
                })
                .collect(),
        })
        .collect();

    DiffPanelViewModel {
        relative_path: relative_path.to_string(),
        summary_text,
        hunks,
        warning: result.warning,
        truncated: result.truncated,
    }
}

/// Builds one single-side text preview panel for left-only/right-only file entries.
pub fn map_single_side_file_preview(
    left_root: &str,
    right_root: &str,
    row: &CompareEntryRowViewModel,
) -> DiffPanelViewModel {
    let (side_label, line_no_mode, target_path) = match row.status.as_str() {
        "left-only" => (
            "left-only",
            "left",
            PathBuf::from(left_root.trim()).join(&row.relative_path),
        ),
        "right-only" => (
            "right-only",
            "right",
            PathBuf::from(right_root.trim()).join(&row.relative_path),
        ),
        "equal" => {
            let left_path = PathBuf::from(left_root.trim()).join(&row.relative_path);
            let right_path = PathBuf::from(right_root.trim()).join(&row.relative_path);
            (
                "equal",
                "both",
                if left_path.exists() {
                    left_path
                } else {
                    right_path
                },
            )
        }
        _ => {
            return build_single_side_unavailable_panel(
                row.relative_path.as_str(),
                "preview is only available for left-only/right-only/equal entries",
            );
        }
    };

    let Ok(bytes) = fs::read(&target_path) else {
        return build_single_side_unavailable_panel(
            row.relative_path.as_str(),
            format!(
                "cannot read file for {side_label} preview: {}",
                target_path.display()
            )
            .as_str(),
        );
    };
    if bytes.contains(&0u8) {
        return build_single_side_unavailable_panel(
            row.relative_path.as_str(),
            format!(
                "{side_label} preview unavailable: binary content is not supported in this viewer"
            )
            .as_str(),
        );
    }

    let mut warning = None;
    let mut truncated = false;
    let preview_bytes = if bytes.len() > SINGLE_SIDE_PREVIEW_MAX_BYTES {
        truncated = true;
        warning = Some(format!(
            "{side_label} preview truncated to {} KB",
            SINGLE_SIDE_PREVIEW_MAX_BYTES / 1024
        ));
        &bytes[..SINGLE_SIDE_PREVIEW_MAX_BYTES]
    } else {
        &bytes[..]
    };

    let text = match std::str::from_utf8(preview_bytes) {
        Ok(value) => value.to_string(),
        Err(_) => {
            return build_single_side_unavailable_panel(
                row.relative_path.as_str(),
                format!(
                    "{side_label} preview unavailable: text preview currently supports UTF-8 files only"
                )
                .as_str(),
            );
        }
    };

    let mut lines = text.lines().collect::<Vec<_>>();
    if lines.len() > SINGLE_SIDE_PREVIEW_MAX_LINES {
        lines.truncate(SINGLE_SIDE_PREVIEW_MAX_LINES);
        truncated = true;
        warning = Some(format!(
            "{side_label} preview truncated to {} lines",
            SINGLE_SIDE_PREVIEW_MAX_LINES
        ));
    }

    let line_view_models = lines
        .iter()
        .enumerate()
        .map(|(index, line)| DiffLineViewModel {
            old_line_no: if line_no_mode == "left" || line_no_mode == "both" {
                Some(index + 1)
            } else {
                None
            },
            new_line_no: if line_no_mode == "right" || line_no_mode == "both" {
                Some(index + 1)
            } else {
                None
            },
            kind: "Context".to_string(),
            content: (*line).to_string(),
        })
        .collect::<Vec<_>>();

    let old_len = if line_no_mode == "left" || line_no_mode == "both" {
        line_view_models.len()
    } else {
        0
    };
    let new_len = if line_no_mode == "right" || line_no_mode == "both" {
        line_view_models.len()
    } else {
        0
    };

    DiffPanelViewModel {
        relative_path: row.relative_path.clone(),
        summary_text: format!(
            "{side_label} preview lines={}{}",
            line_view_models.len(),
            if truncated { " (truncated)" } else { "" }
        ),
        hunks: vec![DiffHunkViewModel {
            old_start: 1,
            old_len,
            new_start: 1,
            new_len,
            lines: line_view_models,
        }],
        warning,
        truncated,
    }
}

fn build_single_side_unavailable_panel(relative_path: &str, reason: &str) -> DiffPanelViewModel {
    DiffPanelViewModel {
        relative_path: relative_path.to_string(),
        summary_text: "single-side preview unavailable".to_string(),
        hunks: vec![DiffHunkViewModel {
            old_start: 1,
            old_len: 1,
            new_start: 1,
            new_len: 1,
            lines: vec![DiffLineViewModel {
                old_line_no: None,
                new_line_no: None,
                kind: "Context".to_string(),
                content: format!("[preview unavailable] {reason}"),
            }],
        }],
        warning: Some(reason.to_string()),
        truncated: false,
    }
}

/// Builds an AI analysis request from selected row + detailed diff panel payload.
pub fn build_analyze_diff_request(
    row: &CompareEntryRowViewModel,
    diff: &DiffPanelViewModel,
    diff_warning: Option<&str>,
    diff_truncated: bool,
    mut config: AiConfig,
) -> Result<AnalyzeDiffRequest, String> {
    if !row.can_load_analysis {
        return Err(row
            .analysis_blocked_reason
            .clone()
            .unwrap_or_else(|| "selected row does not support AI analysis".to_string()));
    }
    if diff.hunks.is_empty() {
        return Err("load detailed diff before running AI analysis".to_string());
    }

    let diff_excerpt = build_diff_excerpt(diff);
    if diff_excerpt.trim().is_empty() {
        return Err("detailed diff excerpt is empty, cannot run AI analysis".to_string());
    }

    if config.provider_kind == fc_ai::AiProviderKind::OpenAiCompatible {
        if config
            .openai_endpoint
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
        {
            return Err(
                "remote provider endpoint is required for OpenAI-compatible mode".to_string(),
            );
        }
        if config
            .openai_api_key
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
        {
            return Err(
                "remote provider api key is required for OpenAI-compatible mode".to_string(),
            );
        }
        if config
            .openai_model
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
        {
            return Err("remote provider model is required for OpenAI-compatible mode".to_string());
        }
    }
    config.openai_endpoint = normalize_optional_text(config.openai_endpoint);
    config.openai_api_key = normalize_optional_text(config.openai_api_key);
    config.openai_model = normalize_optional_text(config.openai_model);

    let mut notes = Vec::new();
    if diff_truncated {
        notes.push("detailed diff output is truncated".to_string());
    }
    if let Some(warning) = diff_warning.map(str::trim).filter(|item| !item.is_empty()) {
        notes.push(warning.to_string());
    }

    Ok(AnalyzeDiffRequest {
        task: AnalysisTask::RiskReview,
        relative_path: Some(row.relative_path.clone()),
        language_hint: infer_language_hint(&row.relative_path),
        diff_excerpt,
        summary: Some(build_diff_summary(diff)),
        truncation_note: if notes.is_empty() {
            None
        } else {
            Some(notes.join("; "))
        },
        config,
    })
}

/// Maps AI analysis response into panel-ready view model.
pub fn map_analyze_diff_response(response: AnalyzeDiffResponse) -> AnalysisResultViewModel {
    AnalysisResultViewModel {
        title: response.title,
        risk_level: ai_risk_level_text(response.risk_level).to_string(),
        rationale: response.rationale,
        key_points: response.key_points,
        review_suggestions: response.review_suggestions,
    }
}

#[cfg(test)]
#[cfg(test)]
#[allow(dead_code)]
fn map_entry_row(entry: &CompareEntry) -> CompareEntryRowViewModel {
    let diff_blocked_reason = detailed_diff_blocked_reason(entry);
    let analysis_blocked_reason = detailed_analysis_blocked_reason(entry, &diff_blocked_reason);
    CompareEntryRowViewModel {
        relative_path: entry.relative_path.clone(),
        status: status_text(entry.status),
        detail: detail_text(&entry.detail, entry.kind),
        entry_kind: kind_text(entry.kind).to_string(),
        detail_kind: detail_kind_text(&entry.detail).to_string(),
        can_load_diff: diff_blocked_reason.is_none(),
        diff_blocked_reason,
        can_load_analysis: analysis_blocked_reason.is_none(),
        analysis_blocked_reason,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn status_text(status: EntryStatus) -> String {
    match status {
        EntryStatus::LeftOnly => "left-only".to_string(),
        EntryStatus::RightOnly => "right-only".to_string(),
        EntryStatus::Equal => "equal".to_string(),
        EntryStatus::Different => "different".to_string(),
        EntryStatus::Pending => "pending".to_string(),
        EntryStatus::Skipped => "skipped".to_string(),
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn detail_text(detail: &EntryDetail, kind: EntryKind) -> String {
    match detail {
        EntryDetail::None => format!("kind={}", kind_text(kind)),
        EntryDetail::Message(msg) => msg.clone(),
        EntryDetail::TypeMismatch { left, right } => format!(
            "type mismatch: left={} right={}",
            kind_text(*left),
            kind_text(*right)
        ),
        EntryDetail::FileComparison {
            left_size,
            right_size,
            content_checked,
        } => format!(
            "file compare: left={}B right={}B content_checked={}",
            left_size, right_size, content_checked
        ),
        EntryDetail::ContentComparisonDeferred => "content comparison deferred".to_string(),
        EntryDetail::TextDetailDeferred {
            reason,
            left_size,
            right_size,
            max_text_file_size_bytes,
            content_checked,
        } => {
            let reason_text = match reason {
                TextDetailDeferredReason::LargeDirectoryMode => "large-directory mode",
                TextDetailDeferredReason::FileTooLarge => "file too large",
            };
            format!(
                "text detail deferred ({reason_text}): left={}B right={}B limit={}B content_checked={}",
                left_size, right_size, max_text_file_size_bytes, content_checked
            )
        }
        EntryDetail::TextDiff(summary) => format!(
            "text summary: hunks={} +{} -{} ctx={}",
            summary.hunk_count, summary.added_lines, summary.removed_lines, summary.context_lines
        ),
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn detail_kind_text(detail: &EntryDetail) -> &'static str {
    match detail {
        EntryDetail::None => "none",
        EntryDetail::Message(_) => "message",
        EntryDetail::TypeMismatch { .. } => "type-mismatch",
        EntryDetail::FileComparison { .. } => "file-comparison",
        EntryDetail::ContentComparisonDeferred => "content-comparison-deferred",
        EntryDetail::TextDetailDeferred { .. } => "text-detail-deferred",
        EntryDetail::TextDiff(_) => "text-diff",
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn kind_text(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::File => "file",
        EntryKind::Directory => "directory",
        EntryKind::Symlink => "symlink",
        EntryKind::Other => "other",
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn detailed_diff_blocked_reason(entry: &CompareEntry) -> Option<String> {
    if entry.kind != EntryKind::File {
        return Some("detailed text diff is only available for file entries".to_string());
    }

    match entry.status {
        EntryStatus::LeftOnly | EntryStatus::RightOnly => {}
        EntryStatus::Skipped => {
            return Some(
                "entry was skipped during compare and cannot load detailed diff".to_string(),
            );
        }
        EntryStatus::Equal | EntryStatus::Different | EntryStatus::Pending => {}
    }
    if matches!(
        entry.status,
        EntryStatus::LeftOnly | EntryStatus::RightOnly | EntryStatus::Equal
    ) {
        return None;
    }

    match &entry.detail {
        EntryDetail::TypeMismatch { .. } => {
            Some("type mismatch entries cannot load detailed text diff".to_string())
        }
        EntryDetail::FileComparison { .. } => Some(
            "entry was compared as non-text/binary candidate, detailed text diff unavailable"
                .to_string(),
        ),
        EntryDetail::Message(message) => Some(format!(
            "entry detail indicates detailed text diff is unavailable: {message}"
        )),
        EntryDetail::None
        | EntryDetail::ContentComparisonDeferred
        | EntryDetail::TextDetailDeferred { .. }
        | EntryDetail::TextDiff(_) => None,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn detailed_analysis_blocked_reason(
    entry: &CompareEntry,
    diff_blocked_reason: &Option<String>,
) -> Option<String> {
    if let Some(reason) = diff_blocked_reason {
        return Some(reason.clone());
    }

    if entry.status != EntryStatus::Different {
        return Some("AI analysis is only available for changed file entries".to_string());
    }

    None
}

fn build_diff_excerpt(diff: &DiffPanelViewModel) -> String {
    let mut out = Vec::new();
    for hunk in &diff.hunks {
        out.push(hunk.header());
        for line in &hunk.lines {
            out.push(format!("{}{}", line.marker(), line.content));
        }
    }
    out.join("\n")
}

fn build_diff_summary(diff: &DiffPanelViewModel) -> TextDiffSummary {
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut context = 0usize;
    for hunk in &diff.hunks {
        for line in &hunk.lines {
            match line.kind_tag() {
                "added" => added += 1,
                "removed" => removed += 1,
                _ => context += 1,
            }
        }
    }

    TextDiffSummary {
        hunk_count: diff.hunks.len(),
        added_lines: added,
        removed_lines: removed,
        context_lines: context,
    }
}

fn infer_language_hint(relative_path: &str) -> Option<String> {
    let extension = PathBuf::from(relative_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())?;

    let language = match extension.as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "go" => "go",
        "java" => "java",
        "kt" => "kotlin",
        "swift" => "swift",
        "rb" => "ruby",
        "php" => "php",
        "c" => "c",
        "h" => "c-header",
        "cc" | "cpp" | "cxx" | "hpp" => "cpp",
        "cs" => "csharp",
        "lua" => "lua",
        "sh" => "shell",
        "ps1" => "powershell",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" => "markdown",
        "xml" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "sql" => "sql",
        _ => return None,
    };
    Some(language.to_string())
}

fn normalize_optional_text(raw: Option<String>) -> Option<String> {
    raw.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn ai_risk_level_text(level: AiRiskLevel) -> &'static str {
    match level {
        AiRiskLevel::Low => "low",
        AiRiskLevel::Medium => "medium",
        AiRiskLevel::High => "high",
    }
}

fn diff_line_kind_text(kind: DiffLineKind) -> &'static str {
    match kind {
        DiffLineKind::Added => "Added",
        DiffLineKind::Removed => "Removed",
        DiffLineKind::Context => "Context",
    }
}

#[cfg(test)]
#[path = "tests/bridge_tests.rs"]
mod tests;
