//! Bridge between UI event handlers and presenter logic.

use crate::commands::UiCommand;
use crate::presenter::Presenter;
use crate::state::AppState;
use crate::view_models::{CompareEntryRowViewModel, CompareResultViewModel};
use fc_core::{
    CompareEntry, CompareOptions, CompareReport, CompareRequest, EntryDetail, EntryKind,
    EntryStatus, TextDetailDeferredReason,
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

fn map_entry_row(entry: &CompareEntry) -> CompareEntryRowViewModel {
    CompareEntryRowViewModel {
        relative_path: entry.relative_path.clone(),
        status: status_text(entry.status),
        detail: detail_text(&entry.detail, entry.kind),
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

fn kind_text(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::File => "file",
        EntryKind::Directory => "directory",
        EntryKind::Symlink => "symlink",
        EntryKind::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
