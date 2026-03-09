//! Report-level models.

use crate::domain::entry::{CompareEntry, EntryDetail, EntryStatus, TextDetailDeferredReason};
use serde::{Deserialize, Serialize};

/// Top-level result for one compare request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompareReport {
    /// Summary counters.
    pub summary: CompareSummary,
    /// Entry-level details.
    pub entries: Vec<CompareEntry>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// Indicates output was intentionally truncated.
    pub truncated: bool,
}

impl CompareReport {
    /// Creates an empty report with zero counters.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Builds a report from entry list and optional warnings.
    pub fn from_entries(
        entries: Vec<CompareEntry>,
        warnings: Vec<String>,
        truncated: bool,
    ) -> Self {
        let mut summary = CompareSummary::default();
        for entry in &entries {
            summary.record_status(entry.status);
            summary.record_detail(&entry.detail);
        }

        Self {
            summary,
            entries,
            warnings,
            truncated,
        }
    }

    /// Appends one entry and updates summary counters.
    pub fn push_entry(&mut self, entry: CompareEntry) {
        self.summary.record_status(entry.status);
        self.summary.record_detail(&entry.detail);
        self.entries.push(entry);
    }
}

/// Aggregate statistics for compare output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompareSummary {
    /// Total entries considered.
    pub total_entries: usize,
    /// Count of left-only entries.
    pub left_only: usize,
    /// Count of right-only entries.
    pub right_only: usize,
    /// Count of equal entries.
    pub equal: usize,
    /// Count of different entries.
    pub different: usize,
    /// Count of entries pending content comparison.
    pub pending: usize,
    /// Count of skipped entries.
    pub skipped: usize,
    /// Indicates large-directory protection mode was enabled.
    pub large_mode: bool,
    /// Indicates summary-first protection mode was enabled.
    pub summary_first_mode: bool,
    /// Count of entries whose detail was deferred by protection policy.
    pub deferred_detail_entries: usize,
    /// Count of text entries whose detail was deferred due to file size.
    pub oversized_text_entries: usize,
}

impl CompareSummary {
    /// Returns true when no differences and no one-sided entries are present.
    pub fn is_clean(&self) -> bool {
        self.left_only == 0 && self.right_only == 0 && self.different == 0 && self.pending == 0
    }

    /// Returns true when at least one meaningful difference exists.
    pub fn has_differences(&self) -> bool {
        self.left_only > 0 || self.right_only > 0 || self.different > 0
    }

    /// Returns true when any entry is not fully compared yet.
    pub fn has_pending(&self) -> bool {
        self.pending > 0
    }

    /// Records one entry status into summary counters.
    pub fn record_status(&mut self, status: EntryStatus) {
        self.total_entries += 1;
        match status {
            EntryStatus::LeftOnly => self.left_only += 1,
            EntryStatus::RightOnly => self.right_only += 1,
            EntryStatus::Equal => self.equal += 1,
            EntryStatus::Different => self.different += 1,
            EntryStatus::Pending => self.pending += 1,
            EntryStatus::Skipped => self.skipped += 1,
        }
    }

    /// Records one entry detail into summary counters.
    pub fn record_detail(&mut self, detail: &EntryDetail) {
        match detail {
            EntryDetail::ContentComparisonDeferred => {
                self.deferred_detail_entries += 1;
            }
            EntryDetail::TextDetailDeferred { reason, .. } => {
                self.deferred_detail_entries += 1;
                if matches!(reason, TextDetailDeferredReason::FileTooLarge) {
                    self.oversized_text_entries += 1;
                }
            }
            EntryDetail::None
            | EntryDetail::Message(_)
            | EntryDetail::TypeMismatch { .. }
            | EntryDetail::FileComparison { .. }
            | EntryDetail::TextDiff(_) => {}
        }
    }

    /// Applies compare-level protection mode flags.
    pub fn set_protection_mode(&mut self, large_mode: bool, summary_first_mode: bool) {
        self.large_mode = large_mode;
        self.summary_first_mode = summary_first_mode;
    }
}
