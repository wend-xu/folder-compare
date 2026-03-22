use fc_core::{
    CompareError, CompareOptions, CompareRequest, EntryDetail, EntryStatus, PathSide, compare_dirs,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn create_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, content).expect("file should be written");
}

#[test]
fn compare_dirs_empty_directories_are_clean() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        CompareOptions::default(),
    ))
    .expect("empty directories should compare successfully");

    assert!(report.entries.is_empty());
    assert_eq!(report.summary.total_entries, 0);
    assert!(report.summary.is_clean());
    assert!(report.warnings.is_empty());
    assert!(!report.truncated);
}

#[test]
fn compare_dirs_aligns_entries_and_compares_files() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_file(&left.path().join("only_left.txt"), "L");
    create_file(&right.path().join("only_right.txt"), "R");

    fs::create_dir_all(left.path().join("shared_dir")).expect("left shared dir should be created");
    fs::create_dir_all(right.path().join("shared_dir"))
        .expect("right shared dir should be created");

    create_file(&left.path().join("type_mismatch"), "left file");
    fs::create_dir_all(right.path().join("type_mismatch"))
        .expect("right mismatch dir should be created");

    create_file(
        &left.path().join("nested/common/aligned.txt"),
        "left content",
    );
    create_file(
        &right.path().join("nested/common/aligned.txt"),
        "right content",
    );

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        CompareOptions::default(),
    ))
    .expect("directories should compare successfully");

    let by_path: BTreeMap<_, _> = report
        .entries
        .iter()
        .map(|entry| (entry.relative_path.clone(), entry))
        .collect();

    assert_eq!(report.summary.total_entries, report.entries.len());

    assert_eq!(by_path["only_left.txt"].status, EntryStatus::LeftOnly);
    assert_eq!(by_path["only_right.txt"].status, EntryStatus::RightOnly);

    assert_eq!(by_path["shared_dir"].status, EntryStatus::Equal);

    assert_eq!(
        by_path["nested/common/aligned.txt"].status,
        EntryStatus::Different
    );
    assert!(matches!(
        &by_path["nested/common/aligned.txt"].detail,
        EntryDetail::TextDiff(_)
    ));

    assert_eq!(by_path["type_mismatch"].status, EntryStatus::Different);
    assert!(matches!(
        &by_path["type_mismatch"].detail,
        EntryDetail::TypeMismatch { .. }
    ));

    assert!(report.summary.left_only >= 1);
    assert!(report.summary.right_only >= 1);
    assert!(report.summary.different >= 1);
}

#[test]
fn compare_dirs_rejects_root_that_is_file() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let left_file = temp.path().join("left.txt");
    create_file(&left_file, "content");

    let right = tempfile::tempdir().expect("right tempdir should be created");

    let err = compare_dirs(CompareRequest::new(
        left_file,
        right.path().to_path_buf(),
        CompareOptions::default(),
    ))
    .expect_err("file root should be rejected");

    assert!(matches!(
        err,
        CompareError::RootPathNotDirectory {
            side: PathSide::Left,
            ..
        }
    ));
}

#[test]
fn compare_dirs_scans_nested_directories() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_file(&left.path().join("a/b/c/only_left.txt"), "left");
    create_file(&right.path().join("a/b/c/only_right.txt"), "right");

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        CompareOptions::default(),
    ))
    .expect("nested compare should succeed");

    let by_path: BTreeMap<_, _> = report
        .entries
        .iter()
        .map(|entry| (entry.relative_path.clone(), entry.status))
        .collect();

    assert_eq!(by_path["a"], EntryStatus::Equal);
    assert_eq!(by_path["a/b"], EntryStatus::Equal);
    assert_eq!(by_path["a/b/c"], EntryStatus::Equal);
    assert_eq!(by_path["a/b/c/only_left.txt"], EntryStatus::LeftOnly);
    assert_eq!(by_path["a/b/c/only_right.txt"], EntryStatus::RightOnly);
}
