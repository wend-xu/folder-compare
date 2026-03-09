use fc_core::{
    compare_dirs, diff_text_file, CompareError, CompareOptions, CompareRequest, EntryDetail,
    TextDiffOptions, TextDiffRequest, TextPathUnavailableReason,
};
use std::fs;
use std::path::Path;

fn write_bytes(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent should be created");
    }
    fs::write(path, bytes).expect("file should be written");
}

fn write_text(path: &Path, text: &str) {
    write_bytes(path, text.as_bytes());
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

#[test]
fn identical_text_files_produce_empty_hunks() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.txt");
    let right = dir.path().join("right.txt");
    write_text(&left, "a\nb\n");
    write_text(&right, "a\nb\n");

    let result = diff_text_file(TextDiffRequest::new(
        left,
        right,
        TextDiffOptions::default(),
    ))
    .expect("diff should succeed");

    assert!(result.hunks.is_empty());
    assert!(result.summary.is_equal());
    assert!(!result.truncated);
    assert!(result.warning.is_none());
}

#[test]
fn single_line_add_delete_modify_are_represented() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.txt");
    let right = dir.path().join("right.txt");
    write_text(&left, "a\nb\nc\n");
    write_text(&right, "a\nx\nc\nd\n");

    let result = diff_text_file(TextDiffRequest::new(
        left,
        right,
        TextDiffOptions::default(),
    ))
    .expect("diff should succeed");

    assert!(!result.hunks.is_empty());
    assert!(result.summary.added_lines > 0);
    assert!(result.summary.removed_lines > 0);

    let has_added = result
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .any(|line| matches!(line.kind, fc_core::DiffLineKind::Added));
    let has_removed = result
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .any(|line| matches!(line.kind, fc_core::DiffLineKind::Removed));
    assert!(has_added && has_removed);
}

#[test]
fn multiple_hunks_can_be_generated() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.txt");
    let right = dir.path().join("right.txt");
    write_text(&left, "a\nkeep1\nkeep2\nchange1\nkeep3\nkeep4\nchange2\n");
    write_text(&right, "a\nkeep1\nkeep2\nX\nkeep3\nkeep4\nY\n");

    let options = TextDiffOptions {
        context_lines: 0,
        ..TextDiffOptions::default()
    };
    let result =
        diff_text_file(TextDiffRequest::new(left, right, options)).expect("diff should succeed");

    assert!(result.hunks.len() >= 2);
}

#[test]
fn empty_vs_non_empty_file_reports_changes() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.txt");
    let right = dir.path().join("right.txt");
    write_text(&left, "");
    write_text(&right, "line\n");

    let result = diff_text_file(TextDiffRequest::new(
        left,
        right,
        TextDiffOptions::default(),
    ))
    .expect("diff should succeed");

    assert!(result.summary.added_lines > 0 || result.summary.removed_lines > 0);
}

#[test]
fn utf8_bom_utf16_le_and_be_are_supported() {
    let dir = tempfile::tempdir().expect("tempdir should be created");

    let left_utf8 = dir.path().join("left_utf8.txt");
    let right_utf8 = dir.path().join("right_utf8.txt");
    write_bytes(&left_utf8, &[0xEF, 0xBB, 0xBF, b'a', b'\n']);
    write_bytes(&right_utf8, &[0xEF, 0xBB, 0xBF, b'a', b'\n']);
    let utf8_result = diff_text_file(TextDiffRequest::new(
        left_utf8,
        right_utf8,
        TextDiffOptions::default(),
    ))
    .expect("utf8 bom should decode");
    assert!(utf8_result.summary.is_equal());

    let left_le = dir.path().join("left_le.txt");
    let right_le = dir.path().join("right_le.txt");
    let le = encode_utf16_le_bom("abc\n");
    write_bytes(&left_le, &le);
    write_bytes(&right_le, &le);
    let le_result = diff_text_file(TextDiffRequest::new(
        left_le,
        right_le,
        TextDiffOptions::default(),
    ))
    .expect("utf16 le should decode");
    assert!(le_result.summary.is_equal());

    let left_be = dir.path().join("left_be.txt");
    let right_be = dir.path().join("right_be.txt");
    let be = encode_utf16_be_bom("abc\n");
    write_bytes(&left_be, &be);
    write_bytes(&right_be, &be);
    let be_result = diff_text_file(TextDiffRequest::new(
        left_be,
        right_be,
        TextDiffOptions::default(),
    ))
    .expect("utf16 be should decode");
    assert!(be_result.summary.is_equal());
}

#[test]
fn non_text_file_returns_structured_boundary_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.bin");
    let right = dir.path().join("right.bin");
    write_bytes(&left, &[0x00, 0x01, 0x02]);
    write_bytes(&right, &[0x00, 0x01, 0x03]);

    let err = diff_text_file(TextDiffRequest::new(
        left,
        right,
        TextDiffOptions::default(),
    ))
    .expect_err("non-text diff should return boundary error");
    assert!(matches!(
        err,
        CompareError::TextPathUnavailable {
            reason: TextPathUnavailableReason::NotTextCandidate,
            ..
        }
    ));
}

#[test]
fn decode_failure_returns_structured_boundary_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.txt");
    let right = dir.path().join("right.txt");
    write_bytes(&left, &[0xFF, 0xFE, 0x41]);
    write_bytes(&right, &[0xFF, 0xFE, 0x41]);

    let err = diff_text_file(TextDiffRequest::new(
        left,
        right,
        TextDiffOptions::default(),
    ))
    .expect_err("decode failure should return boundary error");
    assert!(matches!(
        err,
        CompareError::TextPathUnavailable {
            reason: TextPathUnavailableReason::DecodeFailed,
            ..
        }
    ));
}

#[test]
fn invalid_input_path_is_rejected() {
    let req = TextDiffRequest::new(
        std::path::PathBuf::new(),
        std::path::PathBuf::from("b.txt"),
        TextDiffOptions::default(),
    );
    let err = diff_text_file(req).expect_err("invalid input must fail");
    assert!(matches!(err, CompareError::InvalidInput { .. }));
}

#[test]
fn truncation_sets_flag_and_warning() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let left = dir.path().join("left.txt");
    let right = dir.path().join("right.txt");

    let left_text = (0..100)
        .map(|i| format!("left-{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let right_text = (0..100)
        .map(|i| format!("right-{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    write_text(&left, &left_text);
    write_text(&right, &right_text);

    let options = TextDiffOptions {
        context_lines: 0,
        max_hunks: 1,
        max_lines: 10,
        ..TextDiffOptions::default()
    };
    let result =
        diff_text_file(TextDiffRequest::new(left, right, options)).expect("diff should succeed");

    assert!(result.truncated);
    assert!(result.warning.is_some());
}

#[test]
fn compare_dirs_and_diff_text_file_keep_boundary_clear() {
    let left = tempfile::tempdir().expect("left tempdir should be created");
    let right = tempfile::tempdir().expect("right tempdir should be created");
    write_text(&left.path().join("doc.txt"), "a\nleft\n");
    write_text(&right.path().join("doc.txt"), "a\nright\n");

    let report = compare_dirs(CompareRequest::new(
        left.path().to_path_buf(),
        right.path().to_path_buf(),
        CompareOptions::default(),
    ))
    .expect("compare_dirs should succeed");
    let compare_entry = report
        .entries
        .iter()
        .find(|entry| entry.relative_path == "doc.txt")
        .expect("compare entry should exist");
    assert!(matches!(&compare_entry.detail, EntryDetail::TextDiff(_)));

    let detailed = diff_text_file(TextDiffRequest::new(
        left.path().join("doc.txt"),
        right.path().join("doc.txt"),
        TextDiffOptions::default(),
    ))
    .expect("detailed diff should succeed");
    assert!(!detailed.hunks.is_empty());
}
