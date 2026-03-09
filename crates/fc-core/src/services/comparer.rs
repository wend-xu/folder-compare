//! Entry comparison pipeline skeleton.

use crate::domain::entry::{
    CompareEntry, EntryDetail, EntryKind, EntryStatus, TextDetailDeferredReason,
};
use crate::domain::error::{CompareError, PathSide};
use crate::domain::options::{CompareOptions, CompareRequest};
use crate::domain::report::CompareReport;
use crate::infra::path_norm;
use crate::services::hasher;
use crate::services::large_dir::{self, LargeDirPlan, LargeDirStats};
use crate::services::scanner;
use crate::services::text_diff;
use crate::services::text_loader::{self, TextLoadOutcome};
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

#[derive(Debug, Default)]
struct AlignmentOutcome {
    entries: Vec<CompareEntry>,
    warnings: Vec<String>,
    truncated: bool,
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

    let keys = collect_alignment_keys(&left_tree, &right_tree);
    let stats = LargeDirStats {
        aligned_entries: keys.len(),
        total_bytes: estimate_total_file_bytes(&left_tree, &right_tree),
    };
    let plan = large_dir::build_plan(stats, &context.options)?;
    let mut outcome = align_scanned_trees(&left_tree, &right_tree, &context.options, &plan, keys)?;

    let mut warnings = plan.warnings.clone();
    warnings.append(&mut outcome.warnings);
    if plan.force_truncated && !outcome.truncated {
        warnings
            .push("report marked truncated due to summary-first hard-limit protection".to_string());
    }

    let mut report = CompareReport::from_entries(
        outcome.entries,
        warnings,
        outcome.truncated || plan.force_truncated,
    );
    report
        .summary
        .set_protection_mode(plan.large_mode, plan.summary_first_mode);

    Ok(report)
}

fn collect_alignment_keys(
    left_tree: &scanner::ScannedTree,
    right_tree: &scanner::ScannedTree,
) -> Vec<String> {
    let mut keys = BTreeSet::new();
    keys.extend(left_tree.entries.keys().cloned());
    keys.extend(right_tree.entries.keys().cloned());
    keys.into_iter().collect()
}

fn estimate_total_file_bytes(
    left_tree: &scanner::ScannedTree,
    right_tree: &scanner::ScannedTree,
) -> u64 {
    left_tree
        .entries
        .values()
        .chain(right_tree.entries.values())
        .filter_map(|entry| entry.size_bytes)
        .fold(0_u64, u64::saturating_add)
}

fn align_scanned_trees(
    left_tree: &scanner::ScannedTree,
    right_tree: &scanner::ScannedTree,
    options: &CompareOptions,
    large_dir_plan: &LargeDirPlan,
    keys: Vec<String>,
) -> Result<AlignmentOutcome, CompareError> {
    debug_assert_eq!(left_tree.side, PathSide::Left);
    debug_assert_eq!(right_tree.side, PathSide::Right);
    debug_assert_ne!(left_tree.root, right_tree.root);

    let process_limit = large_dir_plan.truncate_entries_to.unwrap_or(keys.len());
    let mut outcome = AlignmentOutcome {
        entries: Vec::with_capacity(keys.len().min(process_limit)),
        warnings: Vec::new(),
        truncated: false,
    };
    let mut deferred_large_mode = 0usize;
    let mut deferred_oversized_text = 0usize;

    for key in keys.iter().take(process_limit) {
        let left = left_tree.entries.get(key.as_str());
        let right = right_tree.entries.get(key.as_str());

        let compare_entry = match (left, right) {
            (Some(left_entry), None) => {
                CompareEntry::new(key.clone(), left_entry.kind, EntryStatus::LeftOnly).with_detail(
                    EntryDetail::Message(format!(
                        "only on left: {}",
                        left_entry.absolute_path.display()
                    )),
                )
            }
            (None, Some(right_entry)) => {
                CompareEntry::new(key.clone(), right_entry.kind, EntryStatus::RightOnly)
                    .with_detail(EntryDetail::Message(format!(
                        "only on right: {}",
                        right_entry.absolute_path.display()
                    )))
            }
            (Some(left_entry), Some(right_entry)) => {
                debug_assert_eq!(left_entry.relative_path, key.as_str());
                debug_assert_eq!(right_entry.relative_path, key.as_str());

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
                            CompareEntry::new(key.clone(), EntryKind::Directory, EntryStatus::Equal)
                        }
                        EntryKind::File => {
                            if let Some(reason) = deferred_text_reason(
                                left_entry,
                                right_entry,
                                options,
                                large_dir_plan,
                            ) {
                                let file_result = hasher::compare_files(
                                    &left_entry.absolute_path,
                                    &right_entry.absolute_path,
                                )?;
                                let detail = EntryDetail::TextDetailDeferred {
                                    reason,
                                    left_size: file_result.left_size,
                                    right_size: file_result.right_size,
                                    max_text_file_size_bytes: options.max_text_file_size_bytes,
                                    content_checked: file_result.content_checked,
                                };
                                if matches!(reason, TextDetailDeferredReason::FileTooLarge) {
                                    deferred_oversized_text += 1;
                                } else {
                                    deferred_large_mode += 1;
                                }
                                if file_result.is_equal {
                                    CompareEntry::new(
                                        key.clone(),
                                        EntryKind::File,
                                        EntryStatus::Equal,
                                    )
                                    .with_detail(detail)
                                } else {
                                    CompareEntry::new(
                                        key.clone(),
                                        EntryKind::File,
                                        EntryStatus::Different,
                                    )
                                    .with_detail(detail)
                                }
                            } else if large_dir_plan.large_mode {
                                let file_result = hasher::compare_files(
                                    &left_entry.absolute_path,
                                    &right_entry.absolute_path,
                                )?;
                                let detail = EntryDetail::FileComparison {
                                    left_size: file_result.left_size,
                                    right_size: file_result.right_size,
                                    content_checked: file_result.content_checked,
                                };
                                if file_result.is_equal {
                                    CompareEntry::new(
                                        key.clone(),
                                        EntryKind::File,
                                        EntryStatus::Equal,
                                    )
                                    .with_detail(detail)
                                } else {
                                    CompareEntry::new(
                                        key.clone(),
                                        EntryKind::File,
                                        EntryStatus::Different,
                                    )
                                    .with_detail(detail)
                                }
                            } else {
                                let left_text = text_loader::load_text_if_candidate(
                                    &left_entry.absolute_path,
                                    options.text_detection,
                                )?;
                                let right_text = text_loader::load_text_if_candidate(
                                    &right_entry.absolute_path,
                                    options.text_detection,
                                )?;

                                match (left_text, right_text) {
                                    (
                                        TextLoadOutcome::Loaded(left_doc),
                                        TextLoadOutcome::Loaded(right_doc),
                                    ) => {
                                        let summary = text_diff::summarize_text_pair(
                                            &left_doc.content,
                                            &right_doc.content,
                                            options,
                                        );
                                        if summary.is_equal() {
                                            CompareEntry::new(
                                                key.clone(),
                                                EntryKind::File,
                                                EntryStatus::Equal,
                                            )
                                            .with_detail(EntryDetail::TextDiff(summary))
                                        } else {
                                            CompareEntry::new(
                                                key.clone(),
                                                EntryKind::File,
                                                EntryStatus::Different,
                                            )
                                            .with_detail(EntryDetail::TextDiff(summary))
                                        }
                                    }
                                    _ => {
                                        let file_result = hasher::compare_files(
                                            &left_entry.absolute_path,
                                            &right_entry.absolute_path,
                                        )?;
                                        debug_assert_eq!(
                                            left_entry.size_bytes,
                                            Some(file_result.left_size)
                                        );
                                        debug_assert_eq!(
                                            right_entry.size_bytes,
                                            Some(file_result.right_size)
                                        );
                                        let detail = EntryDetail::FileComparison {
                                            left_size: file_result.left_size,
                                            right_size: file_result.right_size,
                                            content_checked: file_result.content_checked,
                                        };
                                        if file_result.is_equal {
                                            CompareEntry::new(
                                                key.clone(),
                                                EntryKind::File,
                                                EntryStatus::Equal,
                                            )
                                            .with_detail(detail)
                                        } else {
                                            CompareEntry::new(
                                                key.clone(),
                                                EntryKind::File,
                                                EntryStatus::Different,
                                            )
                                            .with_detail(detail)
                                        }
                                    }
                                }
                            }
                        }
                        EntryKind::Symlink => {
                            CompareEntry::new(key.clone(), EntryKind::Symlink, EntryStatus::Pending)
                                .with_detail(EntryDetail::ContentComparisonDeferred)
                        }
                        EntryKind::Other => {
                            CompareEntry::new(key.clone(), EntryKind::Other, EntryStatus::Skipped)
                                .with_detail(EntryDetail::Message(
                                    "special entry comparison is deferred".to_string(),
                                ))
                        }
                    }
                }
            }
            (None, None) => unreachable!("alignment key must exist on at least one side"),
        };

        outcome.entries.push(compare_entry);
    }

    if keys.len() > process_limit {
        outcome.truncated = true;
        outcome.warnings.push(format!(
            "report truncated by large directory policy: returned {} of {} entries",
            process_limit,
            keys.len()
        ));
    }
    if deferred_large_mode > 0 {
        outcome.warnings.push(format!(
            "text detail deferred for {} entries due to large directory mode",
            deferred_large_mode
        ));
    }
    if deferred_oversized_text > 0 {
        outcome.warnings.push(format!(
            "text detail deferred for {} oversized text entries",
            deferred_oversized_text
        ));
    }

    Ok(outcome)
}

fn deferred_text_reason(
    left_entry: &scanner::ScannedEntry,
    right_entry: &scanner::ScannedEntry,
    options: &CompareOptions,
    large_dir_plan: &LargeDirPlan,
) -> Option<TextDetailDeferredReason> {
    let text_hint = text_loader::has_text_extension_hint(&left_entry.absolute_path)
        || text_loader::has_text_extension_hint(&right_entry.absolute_path);
    if !text_hint {
        return None;
    }

    let max_size = left_entry
        .size_bytes
        .unwrap_or(0)
        .max(right_entry.size_bytes.unwrap_or(0));
    if max_size > options.max_text_file_size_bytes {
        return Some(TextDetailDeferredReason::FileTooLarge);
    }
    if large_dir_plan.large_mode {
        return Some(TextDetailDeferredReason::LargeDirectoryMode);
    }

    None
}
