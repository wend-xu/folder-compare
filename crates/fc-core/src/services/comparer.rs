//! Entry comparison pipeline skeleton.

use crate::domain::entry::{CompareEntry, EntryDetail, EntryKind, EntryStatus};
use crate::domain::error::{CompareError, PathSide};
use crate::domain::options::{CompareOptions, CompareRequest};
use crate::domain::report::CompareReport;
use crate::infra::path_norm;
use crate::services::scanner;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Internal compare context after request validation and normalization.
#[derive(Debug, Clone)]
pub(crate) struct CompareContext {
    /// Normalized left root.
    pub left_root: PathBuf,
    /// Normalized right root.
    pub right_root: PathBuf,
    /// Effective options.
    pub options: CompareOptions,
}

/// Runs the directory compare pipeline skeleton.
pub(crate) fn run_compare(req: CompareRequest) -> Result<CompareReport, CompareError> {
    req.validate()?;

    let left_root = path_norm::normalize_root_path(&req.left_root, PathSide::Left)?;
    let right_root = path_norm::normalize_root_path(&req.right_root, PathSide::Right)?;

    if left_root == right_root {
        return Err(CompareError::SameRootPathNotAllowed { path: left_root });
    }

    let context = CompareContext {
        left_root,
        right_root,
        options: req.options,
    };

    let left_tree = scanner::scan_tree(
        &context.left_root,
        PathSide::Left,
        context.options.follow_symlinks,
    )?;
    let right_tree = scanner::scan_tree(
        &context.right_root,
        PathSide::Right,
        context.options.follow_symlinks,
    )?;
    let entries = align_scanned_trees(&left_tree, &right_tree);

    Ok(CompareReport::from_entries(entries, Vec::new(), false))
}

fn align_scanned_trees(
    left_tree: &scanner::ScannedTree,
    right_tree: &scanner::ScannedTree,
) -> Vec<CompareEntry> {
    debug_assert_eq!(left_tree.side, PathSide::Left);
    debug_assert_eq!(right_tree.side, PathSide::Right);
    debug_assert_ne!(left_tree.root, right_tree.root);

    let mut keys = BTreeSet::new();
    keys.extend(left_tree.entries.keys().cloned());
    keys.extend(right_tree.entries.keys().cloned());

    let mut entries: Vec<CompareEntry> = Vec::with_capacity(keys.len());
    for key in keys {
        let left = left_tree.entries.get(&key);
        let right = right_tree.entries.get(&key);

        let compare_entry = match (left, right) {
            (Some(left_entry), None) => {
                CompareEntry::new(key, left_entry.kind, EntryStatus::LeftOnly).with_detail(
                    EntryDetail::Message(format!(
                        "only on left: {}",
                        left_entry.absolute_path.display()
                    )),
                )
            }
            (None, Some(right_entry)) => {
                CompareEntry::new(key, right_entry.kind, EntryStatus::RightOnly).with_detail(
                    EntryDetail::Message(format!(
                        "only on right: {}",
                        right_entry.absolute_path.display()
                    )),
                )
            }
            (Some(left_entry), Some(right_entry)) => {
                debug_assert_eq!(left_entry.relative_path, key);
                debug_assert_eq!(right_entry.relative_path, key);

                if left_entry.kind != right_entry.kind {
                    CompareEntry::new(key, left_entry.kind, EntryStatus::Different).with_detail(
                        EntryDetail::TypeMismatch {
                            left: left_entry.kind,
                            right: right_entry.kind,
                        },
                    )
                } else {
                    match left_entry.kind {
                        EntryKind::Directory => {
                            CompareEntry::new(key, EntryKind::Directory, EntryStatus::Equal)
                        }
                        EntryKind::File => {
                            CompareEntry::new(key, EntryKind::File, EntryStatus::Pending)
                                .with_detail(EntryDetail::Message(format!(
                                    "content compare deferred (left_size={:?}, right_size={:?})",
                                    left_entry.size_bytes, right_entry.size_bytes
                                )))
                        }
                        EntryKind::Symlink => {
                            CompareEntry::new(key, EntryKind::Symlink, EntryStatus::Pending)
                                .with_detail(EntryDetail::ContentComparisonDeferred)
                        }
                        EntryKind::Other => {
                            CompareEntry::new(key, EntryKind::Other, EntryStatus::Skipped)
                                .with_detail(EntryDetail::Message(
                                    "special entry comparison is deferred".to_string(),
                                ))
                        }
                    }
                }
            }
            (None, None) => unreachable!("alignment key must exist on at least one side"),
        };

        entries.push(compare_entry);
    }

    entries
}
