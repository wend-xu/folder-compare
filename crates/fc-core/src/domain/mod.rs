//! Domain types for comparison and diff.

pub mod diff;
pub mod entry;
pub mod error;
pub mod options;
pub mod report;

pub use diff::{DiffHunk, DiffLine, DiffLineKind, TextDiffResult, TextDiffSummary};
pub use entry::{CompareEntry, EntryDetail, EntryKind, EntryStatus};
pub use error::{
    CompareError, DeferredOperation, InvalidInputKind, IoOperation, PathSide, UnsupportedOperation,
};
pub use options::{
    CompareOptions, CompareRequest, HashAlgorithm, IgnoreWhitespaceMode, LargeDirPolicy,
    TextDetectionStrategy, TextDiffOptions, TextDiffRequest,
};
pub use report::{CompareReport, CompareSummary};
