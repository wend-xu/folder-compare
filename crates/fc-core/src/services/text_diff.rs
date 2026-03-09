//! Text diff calculation pipeline skeleton.

use crate::domain::diff::TextDiffResult;
use crate::domain::error::{CompareError, DeferredOperation, PathSide};
use crate::domain::options::TextDiffRequest;
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
