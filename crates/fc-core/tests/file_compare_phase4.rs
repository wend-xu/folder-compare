use fc_core::{CompareOptions, CompareRequest, EntryDetail, EntryStatus, compare_dirs};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn create_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, content).expect("file should be written");
}

fn compare(left: &Path, right: &Path) -> fc_core::CompareReport {
    compare_dirs(CompareRequest::new(
        left.to_path_buf(),
        right.to_path_buf(),
        CompareOptions::default(),
    ))
    .expect("compare should succeed")
}

#[test]
fn same_name_same_content_is_equal() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_file(&left.path().join("same.bin"), b"abcdef");
    create_file(&right.path().join("same.bin"), b"abcdef");

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "same.bin")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Equal);
    assert!(matches!(
        &entry.detail,
        EntryDetail::FileComparison {
            left_size: 6,
            right_size: 6,
            content_checked: true
        }
    ));
}

#[test]
fn same_name_different_content_same_size_is_different() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_file(&left.path().join("same.bin"), b"abc123");
    create_file(&right.path().join("same.bin"), b"abc124");

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "same.bin")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Different);
    assert!(matches!(
        &entry.detail,
        EntryDetail::FileComparison {
            left_size: 6,
            right_size: 6,
            content_checked: true
        }
    ));
}

#[test]
fn same_name_different_size_is_different_without_content_check() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_file(&left.path().join("same.bin"), b"short");
    create_file(&right.path().join("same.bin"), b"much-longer");

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "same.bin")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Different);
    assert!(matches!(
        &entry.detail,
        EntryDetail::FileComparison {
            left_size: 5,
            right_size: 11,
            content_checked: false
        }
    ));
}

#[test]
fn empty_files_are_equal() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_file(&left.path().join("empty.txt"), b"");
    create_file(&right.path().join("empty.txt"), b"");

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "empty.txt")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Equal);
}

#[test]
fn nested_file_comparison_coexists_with_alignment_states() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_file(&left.path().join("only_left.bin"), b"L");
    create_file(&right.path().join("only_right.bin"), b"R");

    create_file(&left.path().join("type_mismatch"), b"left-file");
    fs::create_dir_all(right.path().join("type_mismatch")).expect("mismatch dir should be created");

    fs::create_dir_all(left.path().join("shared_dir")).expect("left shared dir should be created");
    fs::create_dir_all(right.path().join("shared_dir"))
        .expect("right shared dir should be created");

    create_file(&left.path().join("a/b/same.bin"), b"same");
    create_file(&right.path().join("a/b/same.bin"), b"same");
    create_file(&left.path().join("a/b/diff.bin"), b"left");
    create_file(&right.path().join("a/b/diff.bin"), b"right");

    let report = compare(left.path(), right.path());
    let by_path: BTreeMap<_, _> = report
        .entries
        .iter()
        .map(|entry| (entry.relative_path.clone(), entry.status))
        .collect();

    assert_eq!(by_path["only_left.bin"], EntryStatus::LeftOnly);
    assert_eq!(by_path["only_right.bin"], EntryStatus::RightOnly);
    assert_eq!(by_path["type_mismatch"], EntryStatus::Different);
    assert_eq!(by_path["shared_dir"], EntryStatus::Equal);
    assert_eq!(by_path["a/b/same.bin"], EntryStatus::Equal);
    assert_eq!(by_path["a/b/diff.bin"], EntryStatus::Different);

    assert_eq!(report.summary.total_entries, report.entries.len());
    assert_eq!(report.summary.pending, 0);
}
