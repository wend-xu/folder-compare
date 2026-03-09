//! Text diff API skeleton.

use crate::domain::diff::TextDiffResult;
use crate::domain::error::CompareError;
use crate::domain::options::TextDiffRequest;
use crate::services::text_diff;

/// Computes text diff for a pair of files.
///
/// Phase 2 performs validation and normalization, then enters the text diff
/// pipeline with deferred algorithm execution.
pub fn diff_text_file(req: TextDiffRequest) -> Result<TextDiffResult, CompareError> {
    text_diff::run_text_diff(req)
}
