//! Bridge between UI event handlers and presenter logic.

use crate::commands::UiCommand;
use crate::presenter::Presenter;
use crate::state::AppState;
use crate::view_models::{
    AnalysisResultViewModel, CompareEntryRowViewModel, CompareResultViewModel, DiffHunkViewModel,
    DiffLineViewModel, DiffPanelViewModel,
};
use fc_ai::{
    AiConfig, AnalysisTask, AnalyzeDiffRequest, AnalyzeDiffResponse, RiskLevel as AiRiskLevel,
};
use fc_core::{
    CompareEntry, CompareOptions, CompareReport, CompareRequest, DiffLineKind, EntryDetail,
    EntryKind, EntryStatus, TextDetailDeferredReason, TextDiffOptions, TextDiffRequest,
    TextDiffResult, TextDiffSummary,
};
use std::path::PathBuf;

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

    let entry_rows = report.entries.iter().map(map_entry_row).collect::<Vec<_>>();

    CompareResultViewModel {
        summary_text,
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

fn kind_text(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::File => "file",
        EntryKind::Directory => "directory",
        EntryKind::Symlink => "symlink",
        EntryKind::Other => "other",
    }
}

fn detailed_diff_blocked_reason(entry: &CompareEntry) -> Option<String> {
    if entry.kind != EntryKind::File {
        return Some("detailed text diff is only available for file entries".to_string());
    }

    match entry.status {
        EntryStatus::LeftOnly => {
            return Some("detailed diff requires files that exist on both sides".to_string());
        }
        EntryStatus::RightOnly => {
            return Some("detailed diff requires files that exist on both sides".to_string());
        }
        EntryStatus::Skipped => {
            return Some(
                "entry was skipped during compare and cannot load detailed diff".to_string(),
            );
        }
        EntryStatus::Equal | EntryStatus::Different | EntryStatus::Pending => {}
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
mod tests {
    use super::*;
    use fc_core::{DiffHunk, DiffLine, TextDiffSummary};
    use std::path::PathBuf;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn build_compare_request_validates_required_paths() {
        let left = tempdir().expect("left temp dir should be created");
        let right = tempdir().expect("right temp dir should be created");

        assert!(build_compare_request("", right.path().to_string_lossy().as_ref()).is_err());
        assert!(build_compare_request(left.path().to_string_lossy().as_ref(), "").is_err());
        assert!(build_compare_request(
            left.path().to_string_lossy().as_ref(),
            right.path().to_string_lossy().as_ref()
        )
        .is_ok());
    }

    #[test]
    fn build_compare_request_rejects_missing_or_non_directory_paths() {
        let left_missing = tempdir()
            .expect("left temp dir should be created")
            .path()
            .join("missing");
        let right_dir = tempdir().expect("right temp dir should be created");
        let file = NamedTempFile::new().expect("temp file should be created");

        let missing_err = build_compare_request(
            left_missing.to_string_lossy().as_ref(),
            right_dir.path().to_string_lossy().as_ref(),
        )
        .expect_err("missing folder should return error");
        assert!(missing_err.contains("does not exist"));

        let file_err = build_compare_request(
            file.path().to_string_lossy().as_ref(),
            right_dir.path().to_string_lossy().as_ref(),
        )
        .expect_err("file path should return error");
        assert!(file_err.contains("must be a folder"));
    }

    #[test]
    fn map_compare_report_projects_summary_rows_and_warnings() {
        let mut report = CompareReport::from_entries(
            vec![
                CompareEntry::new("a.txt", EntryKind::File, EntryStatus::Different).with_detail(
                    EntryDetail::TextDiff(fc_core::TextDiffSummary {
                        hunk_count: 1,
                        added_lines: 2,
                        removed_lines: 1,
                        context_lines: 3,
                    }),
                ),
                CompareEntry::new("b.txt", EntryKind::File, EntryStatus::Equal).with_detail(
                    EntryDetail::TextDetailDeferred {
                        reason: TextDetailDeferredReason::FileTooLarge,
                        left_size: 1024,
                        right_size: 1024,
                        max_text_file_size_bytes: 128,
                        content_checked: true,
                    },
                ),
            ],
            vec!["large directory protection enabled".to_string()],
            true,
        );
        report.summary.large_mode = true;
        report.summary.summary_first_mode = true;

        let vm = map_compare_report(report);
        assert!(vm.summary_text.contains("mode=summary-first"));
        assert!(vm.summary_text.contains("truncated=true"));
        assert_eq!(vm.entry_rows.len(), 2);
        assert!(vm.entry_rows[0].detail.contains("text summary"));
        assert!(vm.entry_rows[1].detail.contains("text detail deferred"));
        assert_eq!(vm.warnings.len(), 1);
        assert!(vm.truncated);
    }

    #[test]
    fn build_text_diff_request_uses_selected_row_relative_path() {
        let row = CompareEntryRowViewModel {
            relative_path: "dir/a.txt".to_string(),
            status: "different".to_string(),
            detail: "text summary".to_string(),
            entry_kind: "file".to_string(),
            detail_kind: "text-diff".to_string(),
            can_load_diff: true,
            diff_blocked_reason: None,
            can_load_analysis: true,
            analysis_blocked_reason: None,
        };
        let req = build_text_diff_request("/tmp/left", "/tmp/right", &row)
            .expect("diff request should be built");

        assert_eq!(req.left_path, PathBuf::from("/tmp/left").join("dir/a.txt"));
        assert_eq!(
            req.right_path,
            PathBuf::from("/tmp/right").join("dir/a.txt")
        );
    }

    #[test]
    fn build_text_diff_request_rejects_non_diffable_row() {
        let row = CompareEntryRowViewModel {
            relative_path: "dir/data.bin".to_string(),
            status: "different".to_string(),
            detail: "file compare".to_string(),
            entry_kind: "file".to_string(),
            detail_kind: "file-comparison".to_string(),
            can_load_diff: false,
            diff_blocked_reason: Some("binary candidate".to_string()),
            can_load_analysis: false,
            analysis_blocked_reason: Some("binary candidate".to_string()),
        };
        let err = build_text_diff_request("/tmp/left", "/tmp/right", &row)
            .expect_err("non-diffable row should fail");

        assert!(err.contains("binary candidate"));
    }

    #[test]
    fn map_text_diff_result_projects_warning_truncation_and_lines() {
        let result = TextDiffResult {
            summary: TextDiffSummary {
                hunk_count: 1,
                added_lines: 1,
                removed_lines: 1,
                context_lines: 0,
            },
            hunks: vec![DiffHunk {
                old_start: 2,
                old_len: 1,
                new_start: 2,
                new_len: 1,
                lines: vec![
                    DiffLine {
                        kind: DiffLineKind::Removed,
                        old_line_no: Some(2),
                        new_line_no: None,
                        content: "old".to_string(),
                    },
                    DiffLine {
                        kind: DiffLineKind::Added,
                        old_line_no: None,
                        new_line_no: Some(2),
                        content: "new".to_string(),
                    },
                ],
            }],
            truncated: true,
            warning: Some("line limit reached".to_string()),
        };

        let vm = map_text_diff_result("a.txt", result);
        assert_eq!(vm.relative_path, "a.txt");
        assert!(vm.summary_text.contains("hunks=1"));
        assert_eq!(vm.warning.as_deref(), Some("line limit reached"));
        assert!(vm.truncated);
        assert_eq!(vm.hunks.len(), 1);
        assert_eq!(vm.hunks[0].lines.len(), 2);
        assert_eq!(vm.hunks[0].lines[0].kind, "Removed");
        assert_eq!(vm.hunks[0].lines[1].kind, "Added");
    }

    #[test]
    fn build_analyze_diff_request_uses_selected_diff_payload() {
        let row = CompareEntryRowViewModel {
            relative_path: "src/lib.rs".to_string(),
            status: "different".to_string(),
            detail: "text summary".to_string(),
            entry_kind: "file".to_string(),
            detail_kind: "text-diff".to_string(),
            can_load_diff: true,
            diff_blocked_reason: None,
            can_load_analysis: true,
            analysis_blocked_reason: None,
        };
        let diff = DiffPanelViewModel {
            relative_path: "src/lib.rs".to_string(),
            summary_text: "hunks=1 +1 -1 ctx=0".to_string(),
            hunks: vec![DiffHunkViewModel {
                old_start: 2,
                old_len: 1,
                new_start: 2,
                new_len: 1,
                lines: vec![
                    DiffLineViewModel {
                        old_line_no: Some(2),
                        new_line_no: None,
                        kind: "Removed".to_string(),
                        content: "old".to_string(),
                    },
                    DiffLineViewModel {
                        old_line_no: None,
                        new_line_no: Some(2),
                        kind: "Added".to_string(),
                        content: "new".to_string(),
                    },
                ],
            }],
            warning: Some("line limit reached".to_string()),
            truncated: true,
        };

        let req = build_analyze_diff_request(
            &row,
            &diff,
            diff.warning.as_deref(),
            diff.truncated,
            AiConfig::default(),
        )
        .expect("analysis request should be built");
        assert_eq!(req.task, AnalysisTask::RiskReview);
        assert_eq!(req.relative_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(req.language_hint.as_deref(), Some("rust"));
        assert!(req.diff_excerpt.contains("@@"));
        assert!(req.diff_excerpt.contains("-old"));
        assert!(req.diff_excerpt.contains("+new"));
        assert_eq!(
            req.summary.as_ref().map(|summary| summary.hunk_count),
            Some(1)
        );
        assert!(req
            .truncation_note
            .expect("note should exist")
            .contains("line limit reached"));
    }

    #[test]
    fn build_analyze_diff_request_rejects_non_analyzable_row() {
        let row = CompareEntryRowViewModel {
            relative_path: "left-only.txt".to_string(),
            status: "left-only".to_string(),
            detail: "left only".to_string(),
            entry_kind: "file".to_string(),
            detail_kind: "none".to_string(),
            can_load_diff: false,
            diff_blocked_reason: Some(
                "detailed diff requires files that exist on both sides".into(),
            ),
            can_load_analysis: false,
            analysis_blocked_reason: Some(
                "AI analysis is only available for changed file entries".to_string(),
            ),
        };
        let diff = DiffPanelViewModel::default();
        let err = build_analyze_diff_request(&row, &diff, None, false, AiConfig::default())
            .expect_err("request should reject blocked rows");
        assert!(err.contains("AI analysis") || err.contains("detailed diff"));
    }

    #[test]
    fn build_analyze_diff_request_rejects_incomplete_remote_config() {
        let row = CompareEntryRowViewModel {
            relative_path: "src/lib.rs".to_string(),
            status: "different".to_string(),
            detail: "text summary".to_string(),
            entry_kind: "file".to_string(),
            detail_kind: "text-diff".to_string(),
            can_load_diff: true,
            diff_blocked_reason: None,
            can_load_analysis: true,
            analysis_blocked_reason: None,
        };
        let diff = DiffPanelViewModel {
            relative_path: "src/lib.rs".to_string(),
            summary_text: "hunks=1 +1 -1 ctx=0".to_string(),
            hunks: vec![DiffHunkViewModel {
                old_start: 1,
                old_len: 1,
                new_start: 1,
                new_len: 1,
                lines: vec![DiffLineViewModel {
                    old_line_no: None,
                    new_line_no: Some(1),
                    kind: "Added".to_string(),
                    content: "new".to_string(),
                }],
            }],
            warning: None,
            truncated: false,
        };
        let mut config = AiConfig::default();
        config.provider_kind = fc_ai::AiProviderKind::OpenAiCompatible;
        config.openai_endpoint = Some("http://localhost:11434/v1".to_string());
        config.openai_api_key = None;
        config.openai_model = Some("gpt-4o-mini".to_string());

        let err = build_analyze_diff_request(&row, &diff, None, false, config)
            .expect_err("request should reject incomplete remote config");
        assert!(err.contains("api key"));
    }

    #[test]
    fn map_analyze_diff_response_projects_fields() {
        let response = AnalyzeDiffResponse {
            risk_level: AiRiskLevel::Medium,
            title: "Risk review for src/lib.rs".to_string(),
            rationale: "Some risky operations changed".to_string(),
            key_points: vec!["point a".to_string(), "point b".to_string()],
            review_suggestions: vec!["suggestion a".to_string()],
        };
        let vm = map_analyze_diff_response(response);
        assert_eq!(vm.risk_level, "medium");
        assert!(vm.title.contains("src/lib.rs"));
        assert!(vm.key_points_text().contains("• point a"));
        assert!(vm.review_suggestions_text().contains("• suggestion a"));
    }

    #[test]
    fn map_compare_report_marks_left_only_as_not_diffable() {
        let report = CompareReport::from_entries(
            vec![CompareEntry::new(
                "only-left.txt",
                EntryKind::File,
                EntryStatus::LeftOnly,
            )],
            Vec::new(),
            false,
        );

        let vm = map_compare_report(report);
        assert_eq!(vm.entry_rows.len(), 1);
        assert!(!vm.entry_rows[0].can_load_diff);
        assert!(vm.entry_rows[0].diff_blocked_reason.is_some());
        assert!(!vm.entry_rows[0].can_load_analysis);
        assert!(vm.entry_rows[0].analysis_blocked_reason.is_some());
    }
}
