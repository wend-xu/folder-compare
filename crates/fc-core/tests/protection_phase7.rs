use fc_core::{
    CompareError, CompareOptions, CompareRequest, EntryDetail, EntryStatus, LargeDirPolicy,
    TextDetailDeferredReason, TextDiffOptions, TextDiffRequest, compare_dirs, diff_text_file,
};
use std::fs;
use std::path::Path;

fn write_bytes(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, bytes).expect("file should be written");
}

fn write_text(path: &Path, text: &str) {
    write_bytes(path, text.as_bytes());
}

#[test]
fn compare_dirs_soft_entry_limit_enables_large_mode_and_keeps_report_usable() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    write_text(&left.path().join("a.txt"), "left-a\n");
    write_text(&right.path().join("a.txt"), "right-a\n");
    write_text(&left.path().join("b.txt"), "left-b\n");
    write_text(&right.path().join("b.txt"), "right-b\n");
    write_text(&left.path().join("c.txt"), "left-c\n");
    write_text(&right.path().join("c.txt"), "right-c\n");

    let mut options = CompareOptions::default();
    options.max_entries_soft_limit = 2;
    options.max_entries_hard_limit = 100;
    options.max_total_bytes_soft_limit = u64::MAX / 2;
    options.max_total_bytes_hard_limit = u64::MAX;

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        options,
    ))
    .expect("compare should succeed under soft limit");

    assert!(report.summary.large_mode);
    assert!(report.summary.summary_first_mode);
    assert!(!report.entries.is_empty());
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("soft limits reached"))
    );
    assert!(report.entries.iter().any(|entry| matches!(
        entry.detail,
        EntryDetail::TextDetailDeferred {
            reason: TextDetailDeferredReason::LargeDirectoryMode,
            ..
        }
    )));
}

#[test]
fn compare_dirs_soft_total_bytes_limit_enables_large_mode() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    write_text(&left.path().join("huge.txt"), &"A".repeat(256));
    write_text(&right.path().join("huge.txt"), &"B".repeat(256));

    let mut options = CompareOptions::default();
    options.max_entries_soft_limit = 100;
    options.max_entries_hard_limit = 1_000;
    options.max_total_bytes_soft_limit = 64;
    options.max_total_bytes_hard_limit = 10_000;

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        options,
    ))
    .expect("compare should succeed under soft bytes limit");

    assert!(report.summary.large_mode);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("soft limits reached"))
    );
}

#[test]
fn compare_dirs_hard_limit_refuse_policy_returns_structured_error() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    write_text(&left.path().join("a.txt"), "same\n");
    write_text(&right.path().join("a.txt"), "same\n");
    write_text(&left.path().join("b.txt"), "same\n");
    write_text(&right.path().join("b.txt"), "same\n");

    let mut options = CompareOptions::default();
    options.large_dir_policy = LargeDirPolicy::RefuseAboveHardLimit;
    options.max_entries_soft_limit = 1;
    options.max_entries_hard_limit = 1;
    options.max_total_bytes_soft_limit = u64::MAX / 2;
    options.max_total_bytes_hard_limit = u64::MAX;

    let err = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        options,
    ))
    .expect_err("refuse policy should fail above hard limit");

    assert!(matches!(
        err,
        CompareError::DirectoryTooLarge {
            max_entries_hard_limit: 1,
            ..
        }
    ));
}

#[test]
fn compare_dirs_hard_limit_summary_first_truncates_output() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    write_text(&left.path().join("a.txt"), "left-a\n");
    write_text(&right.path().join("a.txt"), "right-a\n");
    write_text(&left.path().join("b.txt"), "left-b\n");
    write_text(&right.path().join("b.txt"), "right-b\n");
    write_text(&left.path().join("c.txt"), "left-c\n");
    write_text(&right.path().join("c.txt"), "right-c\n");

    let mut options = CompareOptions::default();
    options.large_dir_policy = LargeDirPolicy::SummaryFirst;
    options.max_entries_soft_limit = 1;
    options.max_entries_hard_limit = 2;
    options.max_total_bytes_soft_limit = u64::MAX / 2;
    options.max_total_bytes_hard_limit = u64::MAX;

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        options,
    ))
    .expect("summary-first should return truncated report");

    assert!(report.truncated);
    assert_eq!(report.entries.len(), 2);
    assert!(report.summary.large_mode);
    assert!(report.summary.summary_first_mode);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("report truncated by large directory policy"))
    );
}

#[test]
fn compare_dirs_oversized_text_file_defers_text_detail_but_keeps_file_status() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    let content = "line\n".repeat(64);
    write_text(&left.path().join("big.txt"), &content);
    write_text(&right.path().join("big.txt"), &content);

    let mut options = CompareOptions::default();
    options.max_entries_soft_limit = 100;
    options.max_entries_hard_limit = 1_000;
    options.max_total_bytes_soft_limit = u64::MAX / 2;
    options.max_total_bytes_hard_limit = u64::MAX;
    options.max_text_file_size_bytes = 32;

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        options,
    ))
    .expect("oversized text should not fail compare");

    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "big.txt")
        .expect("entry should exist");
    assert_eq!(entry.status, EntryStatus::Equal);
    assert!(matches!(
        entry.detail,
        EntryDetail::TextDetailDeferred {
            reason: TextDetailDeferredReason::FileTooLarge,
            ..
        }
    ));
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("oversized text entries"))
    );
}

#[test]
fn diff_text_file_rejects_oversized_input_with_structured_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.txt");
    let right = dir.path().join("right.txt");
    write_text(&left, &"a".repeat(256));
    write_text(&right, &"b".repeat(256));

    let options = TextDiffOptions {
        max_file_size_bytes: 32,
        ..TextDiffOptions::default()
    };
    let err = diff_text_file(TextDiffRequest::new(left, right, options))
        .expect_err("oversized detailed diff input should be rejected");

    assert!(matches!(
        err,
        CompareError::DetailedDiffInputTooLarge { max_bytes: 32, .. }
    ));
}

#[test]
fn regression_small_directory_and_binary_path_remain_stable() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    write_text(&left.path().join("doc.txt"), "a\nleft\n");
    write_text(&right.path().join("doc.txt"), "a\nright\n");
    write_bytes(&left.path().join("blob.bin"), b"AAAA");
    write_bytes(&right.path().join("blob.bin"), b"AAAB");

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        CompareOptions::default(),
    ))
    .expect("small compare should remain stable");

    let doc = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "doc.txt")
        .expect("text entry should exist");
    let blob = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "blob.bin")
        .expect("binary entry should exist");

    assert!(matches!(doc.detail, EntryDetail::TextDiff(_)));
    assert!(matches!(blob.detail, EntryDetail::FileComparison { .. }));
}
