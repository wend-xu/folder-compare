use super::*;
use crate::compare_foundation::{CompareBaseStatus, CompareFocusPath};
use fc_core::{DiffHunk, DiffLine, TextDiffSummary};
use std::path::PathBuf;
use tempfile::{NamedTempFile, tempdir};

#[test]
fn build_compare_request_validates_required_paths() {
    let left = tempdir().expect("left temp dir should be created");
    let right = tempdir().expect("right temp dir should be created");

    assert!(build_compare_request("", right.path().to_string_lossy().as_ref()).is_err());
    assert!(build_compare_request(left.path().to_string_lossy().as_ref(), "").is_err());
    assert!(
        build_compare_request(
            left.path().to_string_lossy().as_ref(),
            right.path().to_string_lossy().as_ref()
        )
        .is_ok()
    );
}

#[test]
fn build_compare_request_rejects_missing_or_non_directory_paths() {
    let left_missing = tempdir()
        .expect("left temp dir should be created")
        .path()
        .join("missing");
    let right_dir = tempdir().expect("right temp dir should be created");
    let file = NamedTempFile::new().expect("temp file should be created");

    let missing_err = build_compare_request(
        left_missing.to_string_lossy().as_ref(),
        right_dir.path().to_string_lossy().as_ref(),
    )
    .expect_err("missing folder should return error");
    assert!(missing_err.contains("does not exist"));

    let file_err = build_compare_request(
        file.path().to_string_lossy().as_ref(),
        right_dir.path().to_string_lossy().as_ref(),
    )
    .expect_err("file path should return error");
    assert!(file_err.contains("must be a folder"));
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

    assert_eq!(vm.compare_foundation.source_entry_count(), 2);
    assert_eq!(
        vm.compare_foundation
            .node("a.txt")
            .expect("a.txt node should exist")
            .base_status,
        CompareBaseStatus::Different
    );
    assert_eq!(
        vm.compare_foundation
            .node("b.txt")
            .expect("b.txt node should exist")
            .detail
            .kind_token(),
        "text-detail-deferred"
    );
    assert_eq!(
        vm.compare_foundation
            .immediate_children(&CompareFocusPath::root())
            .iter()
            .map(|child| child.relative_path.as_str())
            .collect::<Vec<_>>(),
        vec!["a.txt", "b.txt"]
    );
}

#[test]
fn build_text_diff_request_uses_selected_row_relative_path() {
    let row = CompareEntryRowViewModel {
        relative_path: "dir/a.txt".to_string(),
        status: "different".to_string(),
        detail: "text summary".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "text-diff".to_string(),
        can_load_diff: true,
        diff_blocked_reason: None,
        can_load_analysis: true,
        analysis_blocked_reason: None,
    };
    let req = build_text_diff_request("/tmp/left", "/tmp/right", &row)
        .expect("diff request should be built");

    assert_eq!(req.left_path, PathBuf::from("/tmp/left").join("dir/a.txt"));
    assert_eq!(
        req.right_path,
        PathBuf::from("/tmp/right").join("dir/a.txt")
    );
}

#[test]
fn build_text_diff_request_rejects_non_diffable_row() {
    let row = CompareEntryRowViewModel {
        relative_path: "dir/data.bin".to_string(),
        status: "different".to_string(),
        detail: "file compare".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "file-comparison".to_string(),
        can_load_diff: false,
        diff_blocked_reason: Some("binary candidate".to_string()),
        can_load_analysis: false,
        analysis_blocked_reason: Some("binary candidate".to_string()),
    };
    let err = build_text_diff_request("/tmp/left", "/tmp/right", &row)
        .expect_err("non-diffable row should fail");

    assert!(err.contains("binary candidate"));
}

#[test]
fn map_text_diff_result_projects_warning_truncation_and_lines() {
    let result = TextDiffResult {
        summary: TextDiffSummary {
            hunk_count: 1,
            added_lines: 1,
            removed_lines: 1,
            context_lines: 0,
        },
        hunks: vec![DiffHunk {
            old_start: 2,
            old_len: 1,
            new_start: 2,
            new_len: 1,
            lines: vec![
                DiffLine {
                    kind: DiffLineKind::Removed,
                    old_line_no: Some(2),
                    new_line_no: None,
                    content: "old".to_string(),
                },
                DiffLine {
                    kind: DiffLineKind::Added,
                    old_line_no: None,
                    new_line_no: Some(2),
                    content: "new".to_string(),
                },
            ],
        }],
        truncated: true,
        warning: Some("line limit reached".to_string()),
    };

    let vm = map_text_diff_result("a.txt", result);
    assert_eq!(vm.relative_path, "a.txt");
    assert!(vm.summary_text.contains("hunks=1"));
    assert_eq!(vm.warning.as_deref(), Some("line limit reached"));
    assert!(vm.truncated);
    assert_eq!(vm.hunks.len(), 1);
    assert_eq!(vm.hunks[0].lines.len(), 2);
    assert_eq!(vm.hunks[0].lines[0].kind, "Removed");
    assert_eq!(vm.hunks[0].lines[1].kind, "Added");
}

#[test]
fn build_analyze_diff_request_uses_selected_diff_payload() {
    let row = CompareEntryRowViewModel {
        relative_path: "src/lib.rs".to_string(),
        status: "different".to_string(),
        detail: "text summary".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "text-diff".to_string(),
        can_load_diff: true,
        diff_blocked_reason: None,
        can_load_analysis: true,
        analysis_blocked_reason: None,
    };
    let diff = DiffPanelViewModel {
        relative_path: "src/lib.rs".to_string(),
        summary_text: "hunks=1 +1 -1 ctx=0".to_string(),
        hunks: vec![DiffHunkViewModel {
            old_start: 2,
            old_len: 1,
            new_start: 2,
            new_len: 1,
            lines: vec![
                DiffLineViewModel {
                    old_line_no: Some(2),
                    new_line_no: None,
                    kind: "Removed".to_string(),
                    content: "old".to_string(),
                },
                DiffLineViewModel {
                    old_line_no: None,
                    new_line_no: Some(2),
                    kind: "Added".to_string(),
                    content: "new".to_string(),
                },
            ],
        }],
        warning: Some("line limit reached".to_string()),
        truncated: true,
    };

    let req = build_analyze_diff_request(
        &row,
        &diff,
        diff.warning.as_deref(),
        diff.truncated,
        AiConfig::default(),
    )
    .expect("analysis request should be built");
    assert_eq!(req.task, AnalysisTask::RiskReview);
    assert_eq!(req.relative_path.as_deref(), Some("src/lib.rs"));
    assert_eq!(req.language_hint.as_deref(), Some("rust"));
    assert!(req.diff_excerpt.contains("@@"));
    assert!(req.diff_excerpt.contains("-old"));
    assert!(req.diff_excerpt.contains("+new"));
    assert_eq!(
        req.summary.as_ref().map(|summary| summary.hunk_count),
        Some(1)
    );
    assert!(
        req.truncation_note
            .expect("note should exist")
            .contains("line limit reached")
    );
}

#[test]
fn build_analyze_diff_request_rejects_non_analyzable_row() {
    let row = CompareEntryRowViewModel {
        relative_path: "left-only.txt".to_string(),
        status: "left-only".to_string(),
        detail: "left only".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "none".to_string(),
        can_load_diff: true,
        diff_blocked_reason: None,
        can_load_analysis: false,
        analysis_blocked_reason: Some(
            "AI analysis is only available for changed file entries".to_string(),
        ),
    };
    let diff = DiffPanelViewModel::default();
    let err = build_analyze_diff_request(&row, &diff, None, false, AiConfig::default())
        .expect_err("request should reject blocked rows");
    assert!(err.contains("AI analysis") || err.contains("detailed diff"));
}

#[test]
fn build_analyze_diff_request_rejects_incomplete_remote_config() {
    let row = CompareEntryRowViewModel {
        relative_path: "src/lib.rs".to_string(),
        status: "different".to_string(),
        detail: "text summary".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "text-diff".to_string(),
        can_load_diff: true,
        diff_blocked_reason: None,
        can_load_analysis: true,
        analysis_blocked_reason: None,
    };
    let diff = DiffPanelViewModel {
        relative_path: "src/lib.rs".to_string(),
        summary_text: "hunks=1 +1 -1 ctx=0".to_string(),
        hunks: vec![DiffHunkViewModel {
            old_start: 1,
            old_len: 1,
            new_start: 1,
            new_len: 1,
            lines: vec![DiffLineViewModel {
                old_line_no: None,
                new_line_no: Some(1),
                kind: "Added".to_string(),
                content: "new".to_string(),
            }],
        }],
        warning: None,
        truncated: false,
    };
    let mut config = AiConfig::default();
    config.provider_kind = fc_ai::AiProviderKind::OpenAiCompatible;
    config.openai_endpoint = Some("http://localhost:11434/v1".to_string());
    config.openai_api_key = None;
    config.openai_model = Some("gpt-4o-mini".to_string());

    let err = build_analyze_diff_request(&row, &diff, None, false, config)
        .expect_err("request should reject incomplete remote config");
    assert!(err.contains("api key"));
}

#[test]
fn map_analyze_diff_response_projects_fields() {
    let response = AnalyzeDiffResponse {
        risk_level: AiRiskLevel::Medium,
        title: "Risk review for src/lib.rs".to_string(),
        rationale: "Some risky operations changed".to_string(),
        key_points: vec!["point a".to_string(), "point b".to_string()],
        review_suggestions: vec!["suggestion a".to_string()],
    };
    let vm = map_analyze_diff_response(response);
    assert_eq!(vm.risk_level, "medium");
    assert!(vm.title.contains("src/lib.rs"));
    assert!(vm.key_points_text().contains("• point a"));
    assert!(vm.review_suggestions_text().contains("• suggestion a"));
}

#[test]
fn map_compare_report_marks_left_only_as_previewable_but_not_analyzable() {
    let report = CompareReport::from_entries(
        vec![CompareEntry::new(
            "only-left.txt",
            EntryKind::File,
            EntryStatus::LeftOnly,
        )],
        Vec::new(),
        false,
    );

    let vm = map_compare_report(report);
    assert_eq!(vm.entry_rows.len(), 1);
    assert!(vm.entry_rows[0].can_load_diff);
    assert!(vm.entry_rows[0].diff_blocked_reason.is_none());
    assert!(!vm.entry_rows[0].can_load_analysis);
    assert!(vm.entry_rows[0].analysis_blocked_reason.is_some());

    let node = vm
        .compare_foundation
        .node("only-left.txt")
        .expect("left-only node should exist");
    assert_eq!(node.base_status, CompareBaseStatus::LeftOnly);
    assert!(node.capabilities.can_load_diff());
    assert!(!node.capabilities.can_load_analysis());
}

#[test]
fn map_single_side_file_preview_projects_left_only_text_lines() {
    let left = tempdir().expect("left temp dir should be created");
    let right = tempdir().expect("right temp dir should be created");
    std::fs::write(left.path().join("only-left.txt"), "alpha\nbeta\n")
        .expect("left file should be written");
    let row = CompareEntryRowViewModel {
        relative_path: "only-left.txt".to_string(),
        status: "left-only".to_string(),
        detail: "left only".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "none".to_string(),
        can_load_diff: true,
        diff_blocked_reason: None,
        can_load_analysis: false,
        analysis_blocked_reason: Some("blocked".to_string()),
    };

    let vm = map_single_side_file_preview(
        left.path().to_string_lossy().as_ref(),
        right.path().to_string_lossy().as_ref(),
        &row,
    );
    assert!(vm.summary_text.contains("left-only preview"));
    assert_eq!(vm.hunks.len(), 1);
    assert_eq!(vm.hunks[0].lines.len(), 2);
    assert_eq!(vm.hunks[0].lines[0].kind, "Context");
    assert_eq!(vm.hunks[0].lines[0].old_line_no, Some(1));
    assert_eq!(vm.hunks[0].lines[0].new_line_no, None);
}

#[test]
fn map_single_side_file_preview_returns_explainer_for_binary_files() {
    let left = tempdir().expect("left temp dir should be created");
    let right = tempdir().expect("right temp dir should be created");
    std::fs::write(
        left.path().join("only-left.bin"),
        [0u8, 159u8, 146u8, 150u8],
    )
    .expect("left binary should be written");
    let row = CompareEntryRowViewModel {
        relative_path: "only-left.bin".to_string(),
        status: "left-only".to_string(),
        detail: "left only".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "none".to_string(),
        can_load_diff: true,
        diff_blocked_reason: None,
        can_load_analysis: false,
        analysis_blocked_reason: Some("blocked".to_string()),
    };

    let vm = map_single_side_file_preview(
        left.path().to_string_lossy().as_ref(),
        right.path().to_string_lossy().as_ref(),
        &row,
    );
    assert!(vm.summary_text.contains("unavailable"));
    assert!(vm.warning.is_some());
    assert_eq!(vm.hunks.len(), 1);
}

#[test]
fn map_single_side_file_preview_projects_equal_text_lines() {
    let left = tempdir().expect("left temp dir should be created");
    let right = tempdir().expect("right temp dir should be created");
    std::fs::write(left.path().join("equal.txt"), "same\nline\n")
        .expect("left file should be written");
    std::fs::write(right.path().join("equal.txt"), "same\nline\n")
        .expect("right file should be written");
    let row = CompareEntryRowViewModel {
        relative_path: "equal.txt".to_string(),
        status: "equal".to_string(),
        detail: "equal".to_string(),
        entry_kind: "file".to_string(),
        detail_kind: "file-comparison".to_string(),
        can_load_diff: true,
        diff_blocked_reason: None,
        can_load_analysis: false,
        analysis_blocked_reason: Some("blocked".to_string()),
    };

    let vm = map_single_side_file_preview(
        left.path().to_string_lossy().as_ref(),
        right.path().to_string_lossy().as_ref(),
        &row,
    );
    assert!(vm.summary_text.contains("equal preview"));
    assert_eq!(vm.hunks.len(), 1);
    assert_eq!(vm.hunks[0].lines.len(), 2);
    assert_eq!(vm.hunks[0].lines[0].kind, "Context");
    assert_eq!(vm.hunks[0].lines[0].old_line_no, Some(1));
    assert_eq!(vm.hunks[0].lines[0].new_line_no, Some(1));
}
