//! Text diff calculation pipeline skeleton.

use crate::domain::diff::{TextDiffResult, TextDiffSummary};
use crate::domain::error::{CompareError, DeferredOperation, PathSide};
use crate::domain::options::{CompareOptions, IgnoreWhitespaceMode, TextDiffRequest};
use crate::infra::path_norm;

/// Runs text diff pipeline skeleton.
pub(crate) fn run_text_diff(req: TextDiffRequest) -> Result<TextDiffResult, CompareError> {
    req.validate()?;

    let left_path = path_norm::normalize_file_path(&req.left_path, PathSide::Left)?;
    let right_path = path_norm::normalize_file_path(&req.right_path, PathSide::Right)?;

    if left_path == right_path {
        return Ok(TextDiffResult::empty());
    }

    Err(CompareError::Deferred {
        operation: DeferredOperation::TextDiffAlgorithm,
    })
}

/// Builds summary-level text diff information for `compare_dirs`.
pub(crate) fn summarize_text_pair(
    left: &str,
    right: &str,
    options: &CompareOptions,
) -> TextDiffSummary {
    let left_lines = normalize_and_split_lines(left, options);
    let right_lines = normalize_and_split_lines(right, options);

    if left_lines == right_lines {
        return TextDiffSummary::empty();
    }

    let operations = build_line_operations(&left_lines, &right_lines);
    let mut summary = TextDiffSummary::empty();
    let mut in_hunk = false;
    for op in operations {
        match op {
            LineOp::Equal => {
                summary.context_lines += 1;
                in_hunk = false;
            }
            LineOp::Added => {
                summary.added_lines += 1;
                if !in_hunk {
                    summary.hunk_count += 1;
                    in_hunk = true;
                }
            }
            LineOp::Removed => {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineOp {
    Equal,
    Added,
    Removed,
}

fn normalize_and_split_lines(input: &str, options: &CompareOptions) -> Vec<String> {
    let normalized_endings = if options.ignore_line_endings {
        input.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        input.to_string()
    };

    normalized_endings
        .split('\n')
        .map(|line| normalize_line(line, options.ignore_whitespace))
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

    let mut ops: Vec<LineOp> = Vec::new();
    let (mut i, mut j) = (n, m);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && left[i - 1] == right[j - 1] {
            ops.push(LineOp::Equal);
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(LineOp::Added);
            j -= 1;
        } else {
            ops.push(LineOp::Removed);
            i -= 1;
        }
    }
    ops.reverse();
    ops
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::options::CompareOptions;

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
}
