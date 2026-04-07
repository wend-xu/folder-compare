use crate::{
    compare_foundation::CompareFoundation, state::AppState, view_models::CompareEntryRowViewModel,
};
use fc_core::{CompareEntry, EntryDetail, EntryKind, EntryStatus, TextDiffSummary};

#[derive(Debug, Clone)]
pub(crate) struct CompareFixture {
    pub foundation: CompareFoundation,
    pub rows: Vec<CompareEntryRowViewModel>,
}

impl CompareFixture {
    pub fn app_state(&self) -> AppState {
        AppState {
            compare_foundation: self.foundation.clone(),
            entry_rows: self.rows.clone(),
            ..AppState::default()
        }
    }
}

pub(crate) fn compare_fixture(entries: Vec<CompareEntry>) -> CompareFixture {
    let foundation = CompareFoundation::from_compare_entries(&entries);
    let rows = foundation.project_legacy_entry_rows();
    CompareFixture { foundation, rows }
}

pub(crate) fn state_from_entries(entries: Vec<CompareEntry>) -> AppState {
    compare_fixture(entries).app_state()
}

pub(crate) fn file_entry(path: &str, status: EntryStatus) -> CompareEntry {
    CompareEntry::new(path, EntryKind::File, status)
}

pub(crate) fn directory_entry(path: &str, status: EntryStatus) -> CompareEntry {
    CompareEntry::new(path, EntryKind::Directory, status)
}

pub(crate) fn file_comparison_entry(
    path: &str,
    status: EntryStatus,
    left_size: u64,
    right_size: u64,
    content_checked: bool,
) -> CompareEntry {
    CompareEntry::new(path, EntryKind::File, status).with_detail(EntryDetail::FileComparison {
        left_size,
        right_size,
        content_checked,
    })
}

pub(crate) fn text_diff_entry(path: &str, status: EntryStatus) -> CompareEntry {
    text_diff_entry_with_summary(path, status, 2, 4, 1, 8)
}

pub(crate) fn text_diff_entry_with_summary(
    path: &str,
    status: EntryStatus,
    hunk_count: usize,
    added_lines: usize,
    removed_lines: usize,
    context_lines: usize,
) -> CompareEntry {
    CompareEntry::new(path, EntryKind::File, status).with_detail(EntryDetail::TextDiff(
        TextDiffSummary {
            hunk_count,
            added_lines,
            removed_lines,
            context_lines,
        },
    ))
}
