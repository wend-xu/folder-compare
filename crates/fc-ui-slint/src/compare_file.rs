//! Dedicated compare-file projection for compare-originated file tabs.

use crate::view_models::{
    CompareEntryRowViewModel, CompareFilePanelViewModel, CompareFileRowViewModel,
    CompareFileTextSegmentViewModel,
};
use fc_core::{DiffHunk, DiffLine, DiffLineKind, TextDiffResult};
use std::fs;
use std::path::PathBuf;

pub fn map_text_diff_result_to_compare_file(
    relative_path: &str,
    result: TextDiffResult,
) -> CompareFilePanelViewModel {
    let summary = &result.summary;
    CompareFilePanelViewModel {
        relative_path: relative_path.to_string(),
        summary_text: format!(
            "hunks={} +{} -{} ctx={}",
            summary.hunk_count, summary.added_lines, summary.removed_lines, summary.context_lines
        ),
        rows: project_diff_rows(&result),
        warning: result.warning,
        truncated: result.truncated,
    }
}

pub fn build_preview_compare_file(
    left_root: &str,
    right_root: &str,
    row: &CompareEntryRowViewModel,
) -> Result<CompareFilePanelViewModel, String> {
    let relative_path = row.relative_path.trim();
    if relative_path.is_empty() {
        return Err("relative path is required for compare preview".to_string());
    }

    match row.status.as_str() {
        "left-only" => {
            let path = PathBuf::from(left_root.trim()).join(relative_path);
            let lines = load_text_lines(&path, "left-only")?;
            Ok(build_single_side_panel(relative_path, "left-only", &lines))
        }
        "right-only" => {
            let path = PathBuf::from(right_root.trim()).join(relative_path);
            let lines = load_text_lines(&path, "right-only")?;
            Ok(build_single_side_panel(relative_path, "right-only", &lines))
        }
        "equal" => {
            let left_path = PathBuf::from(left_root.trim()).join(relative_path);
            let right_path = PathBuf::from(right_root.trim()).join(relative_path);
            let path = if left_path.exists() {
                left_path
            } else {
                right_path
            };
            let lines = load_text_lines(&path, "equal")?;
            Ok(build_equal_panel(relative_path, &lines))
        }
        other => Err(format!(
            "compare preview is only available for left-only/right-only/equal rows, got {other}"
        )),
    }
}

fn project_diff_rows(result: &TextDiffResult) -> Vec<CompareFileRowViewModel> {
    let mut rows = Vec::new();
    for hunk in &result.hunks {
        rows.extend(project_hunk_rows(hunk));
    }
    rows
}

fn project_hunk_rows(hunk: &DiffHunk) -> Vec<CompareFileRowViewModel> {
    let mut rows = Vec::new();
    let mut cursor = 0usize;
    while cursor < hunk.lines.len() {
        let line = &hunk.lines[cursor];
        if line.kind == DiffLineKind::Context {
            rows.push(build_context_row(line));
            cursor += 1;
            continue;
        }

        let start = cursor;
        while cursor < hunk.lines.len() && hunk.lines[cursor].kind != DiffLineKind::Context {
            cursor += 1;
        }
        let block = &hunk.lines[start..cursor];
        let removed = block
            .iter()
            .filter(|line| line.kind == DiffLineKind::Removed)
            .collect::<Vec<_>>();
        let added = block
            .iter()
            .filter(|line| line.kind == DiffLineKind::Added)
            .collect::<Vec<_>>();

        let pair_count = removed.len().max(added.len());
        for pair_index in 0..pair_count {
            match (removed.get(pair_index), added.get(pair_index)) {
                (Some(left), Some(right)) => rows.push(build_modified_row(left, right)),
                (Some(left), None) => rows.push(build_left_only_row(left)),
                (None, Some(right)) => rows.push(build_right_only_row(right)),
                (None, None) => {}
            }
        }
    }
    rows
}

fn build_context_row(line: &DiffLine) -> CompareFileRowViewModel {
    CompareFileRowViewModel {
        row_kind: "context".to_string(),
        relation_label: String::new(),
        relation_tone: "context".to_string(),
        left_line_no: line.old_line_no,
        right_line_no: line.new_line_no,
        left_text: line.content.clone(),
        right_text: line.content.clone(),
        left_segments: plain_segments(&line.content),
        right_segments: plain_segments(&line.content),
        left_padding: false,
        right_padding: false,
        focusable: true,
    }
}

fn build_modified_row(left: &DiffLine, right: &DiffLine) -> CompareFileRowViewModel {
    let (left_segments, right_segments) =
        emphasize_inline_difference(&left.content, &right.content);
    CompareFileRowViewModel {
        row_kind: "modified".to_string(),
        relation_label: "Diff".to_string(),
        relation_tone: "modified".to_string(),
        left_line_no: left.old_line_no,
        right_line_no: right.new_line_no,
        left_text: left.content.clone(),
        right_text: right.content.clone(),
        left_segments,
        right_segments,
        left_padding: false,
        right_padding: false,
        focusable: true,
    }
}

fn build_left_only_row(left: &DiffLine) -> CompareFileRowViewModel {
    CompareFileRowViewModel {
        row_kind: "left-only".to_string(),
        relation_label: "Left".to_string(),
        relation_tone: "left".to_string(),
        left_line_no: left.old_line_no,
        right_line_no: None,
        left_text: left.content.clone(),
        right_text: String::new(),
        left_segments: plain_segments(&left.content),
        right_segments: Vec::new(),
        left_padding: false,
        right_padding: true,
        focusable: true,
    }
}

fn build_right_only_row(right: &DiffLine) -> CompareFileRowViewModel {
    CompareFileRowViewModel {
        row_kind: "right-only".to_string(),
        relation_label: "Right".to_string(),
        relation_tone: "right".to_string(),
        left_line_no: None,
        right_line_no: right.new_line_no,
        left_text: String::new(),
        right_text: right.content.clone(),
        left_segments: Vec::new(),
        right_segments: plain_segments(&right.content),
        left_padding: true,
        right_padding: false,
        focusable: true,
    }
}

fn build_single_side_panel(
    relative_path: &str,
    row_kind: &str,
    lines: &[String],
) -> CompareFilePanelViewModel {
    let rows = lines
        .iter()
        .enumerate()
        .map(|(index, line)| match row_kind {
            "left-only" => CompareFileRowViewModel {
                row_kind: "left-only".to_string(),
                relation_label: "Left".to_string(),
                relation_tone: "left".to_string(),
                left_line_no: Some(index + 1),
                right_line_no: None,
                left_text: line.clone(),
                right_text: String::new(),
                left_segments: plain_segments(line),
                right_segments: Vec::new(),
                left_padding: false,
                right_padding: true,
                focusable: true,
            },
            "right-only" => CompareFileRowViewModel {
                row_kind: "right-only".to_string(),
                relation_label: "Right".to_string(),
                relation_tone: "right".to_string(),
                left_line_no: None,
                right_line_no: Some(index + 1),
                left_text: String::new(),
                right_text: line.clone(),
                left_segments: Vec::new(),
                right_segments: plain_segments(line),
                left_padding: true,
                right_padding: false,
                focusable: true,
            },
            _ => unreachable!("single-side panel only supports one-sided rows"),
        })
        .collect();

    CompareFilePanelViewModel {
        relative_path: relative_path.to_string(),
        summary_text: format!("{row_kind} lines={}", lines.len()),
        rows,
        warning: None,
        truncated: false,
    }
}

fn build_equal_panel(relative_path: &str, lines: &[String]) -> CompareFilePanelViewModel {
    let rows = lines
        .iter()
        .enumerate()
        .map(|(index, line)| CompareFileRowViewModel {
            row_kind: "context".to_string(),
            relation_label: String::new(),
            relation_tone: "context".to_string(),
            left_line_no: Some(index + 1),
            right_line_no: Some(index + 1),
            left_text: line.clone(),
            right_text: line.clone(),
            left_segments: plain_segments(line),
            right_segments: plain_segments(line),
            left_padding: false,
            right_padding: false,
            focusable: true,
        })
        .collect();

    CompareFilePanelViewModel {
        relative_path: relative_path.to_string(),
        summary_text: format!("identical lines={}", lines.len()),
        rows,
        warning: None,
        truncated: false,
    }
}

fn load_text_lines(path: &PathBuf, side_label: &str) -> Result<Vec<String>, String> {
    let bytes = fs::read(path).map_err(|err| {
        format!("cannot read {side_label} file content for Compare File View: {err}")
    })?;
    if bytes.contains(&0) {
        return Err(format!(
            "{side_label} file preview is unavailable in Compare File View for binary content"
        ));
    }
    let text = String::from_utf8(bytes).map_err(|err| {
        format!(
            "{side_label} file preview is unavailable in Compare File View for non-UTF-8 text: {err}"
        )
    })?;
    Ok(split_text_lines(&text))
}

fn split_text_lines(input: &str) -> Vec<String> {
    if input.is_empty() {
        return Vec::new();
    }

    input
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .map(|line| line.to_string())
        .collect()
}

fn plain_segments(text: &str) -> Vec<CompareFileTextSegmentViewModel> {
    vec![CompareFileTextSegmentViewModel {
        text: text.to_string(),
        tone: "plain".to_string(),
    }]
}

fn emphasize_inline_difference(
    left: &str,
    right: &str,
) -> (
    Vec<CompareFileTextSegmentViewModel>,
    Vec<CompareFileTextSegmentViewModel>,
) {
    if left == right {
        return (plain_segments(left), plain_segments(right));
    }

    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();

    let mut prefix = 0usize;
    while prefix < left_chars.len()
        && prefix < right_chars.len()
        && left_chars[prefix] == right_chars[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0usize;
    while suffix < left_chars.len().saturating_sub(prefix)
        && suffix < right_chars.len().saturating_sub(prefix)
        && left_chars[left_chars.len() - 1 - suffix] == right_chars[right_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let left_prefix = left_chars[..prefix].iter().collect::<String>();
    let right_prefix = right_chars[..prefix].iter().collect::<String>();
    let left_focus = left_chars[prefix..left_chars.len().saturating_sub(suffix)]
        .iter()
        .collect::<String>();
    let right_focus = right_chars[prefix..right_chars.len().saturating_sub(suffix)]
        .iter()
        .collect::<String>();
    let left_suffix = left_chars[left_chars.len().saturating_sub(suffix)..]
        .iter()
        .collect::<String>();
    let right_suffix = right_chars[right_chars.len().saturating_sub(suffix)..]
        .iter()
        .collect::<String>();

    (
        build_segment_runs(&left_prefix, &left_focus, &left_suffix),
        build_segment_runs(&right_prefix, &right_focus, &right_suffix),
    )
}

fn build_segment_runs(
    prefix: &str,
    focus: &str,
    suffix: &str,
) -> Vec<CompareFileTextSegmentViewModel> {
    let mut segments = Vec::new();
    if !prefix.is_empty() {
        segments.push(CompareFileTextSegmentViewModel {
            text: prefix.to_string(),
            tone: "plain".to_string(),
        });
    }
    if !focus.is_empty() {
        segments.push(CompareFileTextSegmentViewModel {
            text: focus.to_string(),
            tone: "emphasis".to_string(),
        });
    }
    if !suffix.is_empty() {
        segments.push(CompareFileTextSegmentViewModel {
            text: suffix.to_string(),
            tone: "plain".to_string(),
        });
    }
    if segments.is_empty() {
        segments.push(CompareFileTextSegmentViewModel {
            text: String::new(),
            tone: "plain".to_string(),
        });
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use fc_core::{DiffHunk, TextDiffSummary};

    #[test]
    fn pairs_removed_and_added_lines_into_modified_rows() {
        let result = TextDiffResult {
            summary: TextDiffSummary {
                hunk_count: 1,
                added_lines: 1,
                removed_lines: 1,
                context_lines: 0,
            },
            hunks: vec![DiffHunk {
                old_start: 1,
                old_len: 1,
                new_start: 1,
                new_len: 1,
                lines: vec![
                    DiffLine {
                        kind: DiffLineKind::Removed,
                        old_line_no: Some(1),
                        new_line_no: None,
                        content: "hello old".to_string(),
                    },
                    DiffLine {
                        kind: DiffLineKind::Added,
                        old_line_no: None,
                        new_line_no: Some(1),
                        content: "hello new".to_string(),
                    },
                ],
            }],
            truncated: false,
            warning: None,
        };

        let panel = map_text_diff_result_to_compare_file("src/main.rs", result);
        assert_eq!(panel.rows.len(), 1);
        assert_eq!(panel.rows[0].row_kind, "modified");
        assert_eq!(panel.rows[0].left_line_no, Some(1));
        assert_eq!(panel.rows[0].right_line_no, Some(1));
        assert_eq!(panel.rows[0].left_segments.len(), 2);
        assert_eq!(panel.rows[0].right_segments.len(), 2);
    }

    #[test]
    fn inline_emphasis_handles_cjk_prefix_and_suffix() {
        let (left, right) = emphasize_inline_difference("你好，世界", "你好，Rust");
        assert_eq!(left.len(), 2);
        assert_eq!(left[0].text, "你好，");
        assert_eq!(left[1].tone, "emphasis");
        assert_eq!(right[0].text, "你好，");
        assert_eq!(right[1].tone, "emphasis");
    }
}
