//! Core comparison engine interfaces and domain model.

pub mod api;
pub mod domain;
pub mod ffi;
pub mod infra;
pub mod services;

pub use api::compare::compare_dirs;
pub use api::diff::diff_text_file;
pub use domain::diff::{DiffHunk, DiffLine, DiffLineKind, TextDiffResult, TextDiffSummary};
pub use domain::entry::{CompareEntry, EntryDetail, EntryKind, EntryStatus};
pub use domain::error::{
    CompareError, DeferredOperation, InvalidInputKind, IoOperation, PathSide,
    TextPathUnavailableReason, UnsupportedOperation,
};
pub use domain::options::{
    CompareOptions, CompareRequest, HashAlgorithm, IgnoreWhitespaceMode, LargeDirPolicy,
    TextDetectionStrategy, TextDiffOptions, TextDiffRequest,
};
pub use domain::report::{CompareReport, CompareSummary};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn compare_options_default_is_stable() {
        let options = CompareOptions::default();
        assert!(!options.follow_symlinks);
        assert!(!options.ignore_line_endings);
        assert_eq!(options.large_dir_policy, LargeDirPolicy::SummaryFirst);
        assert_eq!(options.max_entries, 10_000);
    }

    #[test]
    fn text_diff_options_default_is_stable() {
        let options = TextDiffOptions::default();
        assert_eq!(options.context_lines, 3);
        assert_eq!(options.max_hunks, 128);
        assert_eq!(options.max_lines, 20_000);
        assert!(!options.ignore_line_endings);
        assert_eq!(options.ignore_whitespace, IgnoreWhitespaceMode::Preserve);
    }

    #[test]
    fn compare_summary_helpers_work() {
        let mut summary = CompareSummary::default();
        assert!(summary.is_clean());
        assert!(!summary.has_differences());
        assert!(!summary.has_pending());

        summary.record_status(EntryStatus::Equal);
        summary.record_status(EntryStatus::Different);
        summary.record_status(EntryStatus::Pending);
        assert_eq!(summary.total_entries, 3);
        assert_eq!(summary.equal, 1);
        assert_eq!(summary.different, 1);
        assert_eq!(summary.pending, 1);
        assert!(summary.has_differences());
        assert!(summary.has_pending());
        assert!(!summary.is_clean());
    }

    #[test]
    fn compare_report_summary_matches_entries() {
        let entry = CompareEntry::new("a.txt", EntryKind::File, EntryStatus::Different);
        let report =
            CompareReport::from_entries(vec![entry], vec!["placeholder warning".to_string()], true);

        assert_eq!(report.summary.total_entries, 1);
        assert_eq!(report.summary.different, 1);
        assert_eq!(report.warnings.len(), 1);
        assert!(report.truncated);
    }

    #[test]
    fn compare_report_empty_is_consistent() {
        let report = CompareReport::empty();
        assert_eq!(report.summary.total_entries, 0);
        assert!(report.entries.is_empty());
        assert!(!report.truncated);
    }

    #[test]
    fn compare_dirs_rejects_empty_root() {
        let req = CompareRequest::new(
            PathBuf::new(),
            PathBuf::from("right"),
            CompareOptions::default(),
        );
        let err = compare_dirs(req).expect_err("empty path should be rejected");
        assert!(matches!(
            err,
            CompareError::InvalidInput {
                kind: InvalidInputKind::EmptyRootPath {
                    side: PathSide::Left
                }
            }
        ));
    }

    #[test]
    fn compare_dirs_rejects_same_root_after_normalization() {
        let req = CompareRequest::new(
            PathBuf::from("."),
            PathBuf::from("./"),
            CompareOptions::default(),
        );
        let err = compare_dirs(req).expect_err("same normalized roots should be rejected");
        assert!(matches!(err, CompareError::SameRootPathNotAllowed { .. }));
    }

    #[test]
    fn compare_dirs_rejects_missing_root() {
        let req = CompareRequest::new(
            PathBuf::from("left"),
            PathBuf::from("right"),
            CompareOptions::default(),
        );
        let err = compare_dirs(req).expect_err("missing roots should be rejected");
        assert!(matches!(
            err,
            CompareError::RootPathNotFound {
                side: PathSide::Left,
                ..
            } | CompareError::RootPathNotFound {
                side: PathSide::Right,
                ..
            }
        ));
    }

    #[test]
    fn diff_text_file_rejects_empty_path() {
        let req = TextDiffRequest::new(
            PathBuf::new(),
            PathBuf::from("b.txt"),
            TextDiffOptions::default(),
        );
        let err = diff_text_file(req).expect_err("empty file path should be rejected");
        assert!(matches!(
            err,
            CompareError::InvalidInput {
                kind: InvalidInputKind::EmptyFilePath {
                    side: PathSide::Left
                }
            }
        ));
    }

    #[test]
    fn diff_text_file_rejects_out_of_range_options() {
        let req = TextDiffRequest::new(
            PathBuf::from("a.txt"),
            PathBuf::from("b.txt"),
            TextDiffOptions {
                ignore_whitespace: IgnoreWhitespaceMode::Preserve,
                ignore_line_endings: false,
                text_detection: TextDetectionStrategy::ExtensionHeuristic,
                context_lines: 20_000,
                max_hunks: 128,
                max_lines: 20_000,
            },
        );
        let err = diff_text_file(req).expect_err("out-of-range option should be rejected");
        assert!(matches!(
            err,
            CompareError::InvalidInput {
                kind: InvalidInputKind::OptionOutOfRange {
                    name: "context_lines"
                }
            }
        ));
    }

    #[test]
    fn diff_text_file_returns_empty_when_same_path() {
        let req = TextDiffRequest::new(
            PathBuf::from("a/./same.txt"),
            PathBuf::from("a/same.txt"),
            TextDiffOptions::default(),
        );
        let result = diff_text_file(req)
            .expect("same normalized path should return empty placeholder result");
        assert!(result.summary.is_equal());
        assert!(result.hunks.is_empty());
    }

    #[test]
    fn text_diff_summary_empty_is_equal() {
        let summary = TextDiffSummary::empty();
        assert!(summary.is_equal());
    }

    #[test]
    fn diff_text_file_distinct_missing_paths_return_io_error() {
        let req = TextDiffRequest::new(
            PathBuf::from("a.txt"),
            PathBuf::from("b.txt"),
            TextDiffOptions::default(),
        );
        let err = diff_text_file(req).expect_err("missing files should return io error");
        assert!(matches!(err, CompareError::IoBoundary { .. }));
    }
}
