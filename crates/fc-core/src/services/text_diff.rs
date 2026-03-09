//! Text diff calculation services.

use crate::domain::diff::{DiffHunk, DiffLine, DiffLineKind, TextDiffResult, TextDiffSummary};
use crate::domain::error::{CompareError, PathSide};
use crate::domain::options::{
    CompareOptions, IgnoreWhitespaceMode, TextDiffOptions, TextDiffRequest,
};
use crate::infra::path_norm;
use crate::services::text_loader;

/// Runs text diff pipeline for detailed diff API.
pub(crate) fn run_text_diff(req: TextDiffRequest) -> Result<TextDiffResult, CompareError> {
    req.validate()?;

    let left_path = path_norm::normalize_file_path(&req.left_path, PathSide::Left)?;
    let right_path = path_norm::normalize_file_path(&req.right_path, PathSide::Right)?;

    if left_path == right_path {
        return Ok(TextDiffResult::empty());
    }

    let left = text_loader::load_text_for_diff(
        &left_path,
        req.options.text_detection,
        req.options.max_file_size_bytes,
    )?;
    let right = text_loader::load_text_for_diff(
        &right_path,
        req.options.text_detection,
        req.options.max_file_size_bytes,
    )?;

    Ok(build_detailed_diff(
        &left.content,
        &right.content,
        &req.options,
    ))
}

/// Builds summary-level text diff information for `compare_dirs`.
pub(crate) fn summarize_text_pair(
    left: &str,
    right: &str,
    options: &CompareOptions,
) -> TextDiffSummary {
    let left_lines =
        normalize_and_split_lines(left, options.ignore_line_endings, options.ignore_whitespace);
    let right_lines = normalize_and_split_lines(
        right,
        options.ignore_line_endings,
        options.ignore_whitespace,
    );

    if left_lines == right_lines {
        return TextDiffSummary::empty();
    }

    let operations = build_line_operations(&left_lines, &right_lines);
    summarize_from_operations(&operations)
}

fn build_detailed_diff(left: &str, right: &str, options: &TextDiffOptions) -> TextDiffResult {
    let left_lines = prepare_lines(left, options.ignore_line_endings, options.ignore_whitespace);
    let right_lines = prepare_lines(
        right,
        options.ignore_line_endings,
        options.ignore_whitespace,
    );

    let left_norm: Vec<String> = left_lines.iter().map(|l| l.norm.clone()).collect();
    let right_norm: Vec<String> = right_lines.iter().map(|l| l.norm.clone()).collect();
    let operations = build_line_operations(&left_norm, &right_norm);

    let annotated = annotate_operations(&operations, &left_lines, &right_lines);
    let mut hunk_ranges = build_hunk_ranges(&annotated, options.context_lines);

    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut emitted_lines = 0usize;
    let mut truncated = false;

    while let Some((start, end)) = hunk_ranges.first().copied() {
        hunk_ranges.remove(0);

        if hunks.len() >= options.max_hunks {
            truncated = true;
            break;
        }

        let mut lines: Vec<DiffLine> = Vec::new();
        for item in annotated.iter().take(end + 1).skip(start) {
            if emitted_lines >= options.max_lines {
                truncated = true;
                break;
            }

            lines.push(DiffLine {
                kind: item.kind,
                old_line_no: item.old_line_no,
                new_line_no: item.new_line_no,
                content: item.content.clone(),
            });
            emitted_lines += 1;
        }

        if lines.is_empty() {
            break;
        }

        let old_start = annotated[start].old_anchor;
        let new_start = annotated[start].new_anchor;
        let old_len = lines
            .iter()
            .filter(|line| line.old_line_no.is_some())
            .count();
        let new_len = lines
            .iter()
            .filter(|line| line.new_line_no.is_some())
            .count();

        hunks.push(DiffHunk {
            old_start,
            old_len,
            new_start,
            new_len,
            lines,
        });

        if truncated {
            break;
        }
    }

    let mut summary = TextDiffSummary::empty();
    summary.hunk_count = hunks.len();
    for line in hunks.iter().flat_map(|hunk| hunk.lines.iter()) {
        match line.kind {
            DiffLineKind::Added => summary.added_lines += 1,
            DiffLineKind::Removed => summary.removed_lines += 1,
            DiffLineKind::Context => summary.context_lines += 1,
        }
    }

    let warning = if truncated {
        Some(format!(
            "diff output truncated to max_hunks={} and max_lines={}",
            options.max_hunks, options.max_lines
        ))
    } else {
        None
    };

    TextDiffResult {
        summary,
        hunks,
        truncated,
        warning,
    }
}

#[derive(Debug, Clone)]
struct PreparedLine {
    raw: String,
    norm: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineOp {
    Equal { left_idx: usize, right_idx: usize },
    Added { right_idx: usize },
    Removed { left_idx: usize },
}

#[derive(Debug, Clone)]
struct AnnotatedOp {
    kind: DiffLineKind,
    old_line_no: Option<usize>,
    new_line_no: Option<usize>,
    old_anchor: usize,
    new_anchor: usize,
    content: String,
}

fn prepare_lines(
    input: &str,
    ignore_line_endings: bool,
    whitespace_mode: IgnoreWhitespaceMode,
) -> Vec<PreparedLine> {
    split_lines(input, ignore_line_endings)
        .into_iter()
        .map(|line| PreparedLine {
            raw: line.clone(),
            norm: normalize_line(&line, whitespace_mode),
        })
        .collect()
}

fn normalize_and_split_lines(
    input: &str,
    ignore_line_endings: bool,
    whitespace_mode: IgnoreWhitespaceMode,
) -> Vec<String> {
    split_lines(input, ignore_line_endings)
        .into_iter()
        .map(|line| normalize_line(&line, whitespace_mode))
        .collect()
}

fn split_lines(input: &str, ignore_line_endings: bool) -> Vec<String> {
    let normalized_endings = if ignore_line_endings {
        input.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        input.to_string()
    };

    if normalized_endings.is_empty() {
        return Vec::new();
    }

    normalized_endings
        .split('\n')
        .map(|line| line.to_string())
        .collect()
}

fn normalize_line(line: &str, mode: IgnoreWhitespaceMode) -> String {
    match mode {
        IgnoreWhitespaceMode::Preserve => line.to_string(),
        IgnoreWhitespaceMode::TrimEdges => line.trim().to_string(),
        IgnoreWhitespaceMode::NormalizeRuns => {
            let mut output = String::with_capacity(line.len());
            let mut in_whitespace = false;
            for ch in line.chars() {
                if ch == ' ' || ch == '\t' {
                    if !in_whitespace {
                        output.push(' ');
                        in_whitespace = true;
                    }
                } else {
                    output.push(ch);
                    in_whitespace = false;
                }
            }
            output.trim().to_string()
        }
    }
}

fn build_line_operations(left: &[String], right: &[String]) -> Vec<LineOp> {
    let n = left.len();
    let m = right.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            if left[i - 1] == right[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut ops = Vec::new();
    let (mut i, mut j) = (n, m);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && left[i - 1] == right[j - 1] {
            ops.push(LineOp::Equal {
                left_idx: i - 1,
                right_idx: j - 1,
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(LineOp::Added { right_idx: j - 1 });
            j -= 1;
        } else {
            ops.push(LineOp::Removed { left_idx: i - 1 });
            i -= 1;
        }
    }
    ops.reverse();
    ops
}

fn annotate_operations(
    ops: &[LineOp],
    left_lines: &[PreparedLine],
    right_lines: &[PreparedLine],
) -> Vec<AnnotatedOp> {
    let mut old_no = 1usize;
    let mut new_no = 1usize;
    let mut out = Vec::with_capacity(ops.len());

    for op in ops {
        match *op {
            LineOp::Equal { right_idx, .. } => {
                out.push(AnnotatedOp {
                    kind: DiffLineKind::Context,
                    old_line_no: Some(old_no),
                    new_line_no: Some(new_no),
                    old_anchor: old_no,
                    new_anchor: new_no,
                    content: right_lines[right_idx].raw.clone(),
                });
                old_no += 1;
                new_no += 1;
            }
            LineOp::Removed { left_idx } => {
                out.push(AnnotatedOp {
                    kind: DiffLineKind::Removed,
                    old_line_no: Some(old_no),
                    new_line_no: None,
                    old_anchor: old_no,
                    new_anchor: new_no,
                    content: left_lines[left_idx].raw.clone(),
                });
                old_no += 1;
            }
            LineOp::Added { right_idx } => {
                out.push(AnnotatedOp {
                    kind: DiffLineKind::Added,
                    old_line_no: None,
                    new_line_no: Some(new_no),
                    old_anchor: old_no,
                    new_anchor: new_no,
                    content: right_lines[right_idx].raw.clone(),
                });
                new_no += 1;
            }
        }
    }

    out
}

fn build_hunk_ranges(ops: &[AnnotatedOp], context_lines: usize) -> Vec<(usize, usize)> {
    if ops.is_empty() {
        return Vec::new();
    }

    let change_indexes: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, op)| op.kind != DiffLineKind::Context)
        .map(|(idx, _)| idx)
        .collect();
    if change_indexes.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut start = change_indexes[0].saturating_sub(context_lines);
    let mut end = (change_indexes[0] + context_lines).min(ops.len() - 1);

    for idx in change_indexes.into_iter().skip(1) {
        if idx <= end + context_lines + 1 {
            end = end.max((idx + context_lines).min(ops.len() - 1));
        } else {
            ranges.push((start, end));
            start = idx.saturating_sub(context_lines);
            end = (idx + context_lines).min(ops.len() - 1);
        }
    }
    ranges.push((start, end));
    ranges
}

fn summarize_from_operations(operations: &[LineOp]) -> TextDiffSummary {
    let mut summary = TextDiffSummary::empty();
    let mut in_hunk = false;

    for op in operations {
        match op {
            LineOp::Equal { .. } => {
                summary.context_lines += 1;
                in_hunk = false;
            }
            LineOp::Added { .. } => {
                summary.added_lines += 1;
                if !in_hunk {
                    summary.hunk_count += 1;
                    in_hunk = true;
                }
            }
            LineOp::Removed { .. } => {
                summary.removed_lines += 1;
                if !in_hunk {
                    summary.hunk_count += 1;
                    in_hunk = true;
                }
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::options::{CompareOptions, TextDiffOptions};

    #[test]
    fn summarize_equal_text() {
        let options = CompareOptions::default();
        let summary = summarize_text_pair("a\nb\n", "a\nb\n", &options);
        assert!(summary.is_equal());
        assert_eq!(summary.hunk_count, 0);
    }

    #[test]
    fn summarize_different_text() {
        let options = CompareOptions::default();
        let summary = summarize_text_pair("a\nb\n", "a\nc\n", &options);
        assert!(!summary.is_equal());
        assert!(summary.added_lines > 0);
        assert!(summary.removed_lines > 0);
        assert!(summary.hunk_count > 0);
    }

    #[test]
    fn detailed_diff_marks_truncation() {
        let options = TextDiffOptions {
            ignore_whitespace: IgnoreWhitespaceMode::Preserve,
            ignore_line_endings: false,
            text_detection: crate::TextDetectionStrategy::ExtensionHeuristic,
            context_lines: 0,
            max_hunks: 1,
            max_lines: 1,
            max_file_size_bytes: 1_024,
        };

        let result = build_detailed_diff("a\nb\n", "x\ny\n", &options);
        assert!(result.truncated);
        assert!(result.warning.is_some());
    }
}
