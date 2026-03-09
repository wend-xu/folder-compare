//! Bridge between UI event handlers and presenter logic.

use crate::commands::UiCommand;
use crate::presenter::Presenter;
use crate::state::AppState;
use crate::view_models::{
    CompareEntryRowViewModel, CompareResultViewModel, DiffHunkViewModel, DiffLineViewModel,
    DiffPanelViewModel,
};
use fc_core::{
    CompareEntry, CompareOptions, CompareReport, CompareRequest, DiffLineKind, EntryDetail,
    EntryKind, EntryStatus, TextDetailDeferredReason, TextDiffOptions, TextDiffRequest,
    TextDiffResult,
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
    let left = left_root.trim();
    let right = right_root.trim();
    if left.is_empty() {
        return Err("left root path is required".to_string());
    }
    if right.is_empty() {
        return Err("right root path is required".to_string());
    }

    Ok(CompareRequest::new(
        PathBuf::from(left),
        PathBuf::from(right),
        CompareOptions::default(),
    ))
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

fn map_entry_row(entry: &CompareEntry) -> CompareEntryRowViewModel {
    let diff_blocked_reason = detailed_diff_blocked_reason(entry);
    CompareEntryRowViewModel {
        relative_path: entry.relative_path.clone(),
        status: status_text(entry.status),
        detail: detail_text(&entry.detail, entry.kind),
        entry_kind: kind_text(entry.kind).to_string(),
        detail_kind: detail_kind_text(&entry.detail).to_string(),
        can_load_diff: diff_blocked_reason.is_none(),
        diff_blocked_reason,
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

    #[test]
    fn build_compare_request_validates_required_paths() {
        assert!(build_compare_request("", "/tmp").is_err());
        assert!(build_compare_request("/tmp", "").is_err());
        assert!(build_compare_request("/tmp/a", "/tmp/b").is_ok());
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
    }
}
