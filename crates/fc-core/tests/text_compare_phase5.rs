use fc_core::{CompareOptions, CompareRequest, EntryDetail, EntryStatus, compare_dirs};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn create_bytes(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, bytes).expect("file should be written");
}

fn create_text(path: &Path, content: &str) {
    create_bytes(path, content.as_bytes());
}

fn encode_utf16_le_bom(text: &str) -> Vec<u8> {
    let mut out = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

fn encode_utf16_be_bom(text: &str) -> Vec<u8> {
    let mut out = vec![0xFE, 0xFF];
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_be_bytes());
    }
    out
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
fn utf8_text_uses_text_summary_detail() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_text(&left.path().join("a.txt"), "hello\nworld\n");
    create_text(&right.path().join("a.txt"), "hello\nworld\n");

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "a.txt")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Equal);
    assert!(matches!(&entry.detail, EntryDetail::TextDiff(_)));
}

#[test]
fn utf8_bom_is_decoded() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_bytes(
        &left.path().join("bom.txt"),
        &[0xEF, 0xBB, 0xBF, b'a', b'\n'],
    );
    create_bytes(
        &right.path().join("bom.txt"),
        &[0xEF, 0xBB, 0xBF, b'a', b'\n'],
    );

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "bom.txt")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Equal);
    assert!(matches!(&entry.detail, EntryDetail::TextDiff(_)));
}

#[test]
fn utf16le_bom_is_decoded() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    let bytes = encode_utf16_le_bom("abc\n");
    create_bytes(&left.path().join("utf16.txt"), &bytes);
    create_bytes(&right.path().join("utf16.txt"), &bytes);

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "utf16.txt")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Equal);
    assert!(matches!(&entry.detail, EntryDetail::TextDiff(_)));
}

#[test]
fn utf16be_bom_is_decoded() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    let bytes = encode_utf16_be_bom("abc\n");
    create_bytes(&left.path().join("utf16be.txt"), &bytes);
    create_bytes(&right.path().join("utf16be.txt"), &bytes);

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "utf16be.txt")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Equal);
    assert!(matches!(&entry.detail, EntryDetail::TextDiff(_)));
}

#[test]
fn decode_failure_falls_back_to_file_compare() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    let invalid = [0xFF, 0xFE, 0x41];
    create_bytes(&left.path().join("bad.txt"), &invalid);
    create_bytes(&right.path().join("bad.txt"), &invalid);

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "bad.txt")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Equal);
    assert!(matches!(&entry.detail, EntryDetail::FileComparison { .. }));
}

#[test]
fn binary_extension_stays_in_file_compare_path() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_bytes(&left.path().join("blob.bin"), b"abcdef");
    create_bytes(&right.path().join("blob.bin"), b"abcdeg");

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "blob.bin")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Different);
    assert!(matches!(&entry.detail, EntryDetail::FileComparison { .. }));
}

#[test]
fn text_difference_populates_summary() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_text(&left.path().join("doc.md"), "line1\nline2\nline3\n");
    create_text(&right.path().join("doc.md"), "line1\nlineX\nline3\nline4\n");

    let report = compare(left.path(), right.path());
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "doc.md")
        .expect("entry should exist");

    assert_eq!(entry.status, EntryStatus::Different);
    let EntryDetail::TextDiff(summary) = &entry.detail else {
        panic!("expected text summary detail");
    };
    assert!(summary.added_lines > 0);
    assert!(summary.removed_lines > 0);
    assert!(summary.hunk_count > 0);
}

#[test]
fn mixed_directory_compare_remains_consistent() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");

    create_text(&left.path().join("same.txt"), "a\n");
    create_text(&right.path().join("same.txt"), "a\n");
    create_text(&left.path().join("diff.txt"), "left\n");
    create_text(&right.path().join("diff.txt"), "right\n");

    create_bytes(&left.path().join("bin.bin"), b"AAAA");
    create_bytes(&right.path().join("bin.bin"), b"AAAB");

    create_text(&left.path().join("only_left.txt"), "left");
    create_text(&right.path().join("only_right.txt"), "right");

    create_text(&left.path().join("mismatch"), "left-file");
    fs::create_dir_all(right.path().join("mismatch")).expect("mismatch dir should be created");

    let report = compare(left.path(), right.path());
    let by_path: BTreeMap<_, _> = report
        .entries
        .iter()
        .map(|entry| (entry.relative_path.clone(), entry.status))
        .collect();

    assert_eq!(by_path["same.txt"], EntryStatus::Equal);
    assert_eq!(by_path["diff.txt"], EntryStatus::Different);
    assert_eq!(by_path["bin.bin"], EntryStatus::Different);
    assert_eq!(by_path["only_left.txt"], EntryStatus::LeftOnly);
    assert_eq!(by_path["only_right.txt"], EntryStatus::RightOnly);
    assert_eq!(by_path["mismatch"], EntryStatus::Different);

    assert_eq!(report.summary.total_entries, report.entries.len());
}
