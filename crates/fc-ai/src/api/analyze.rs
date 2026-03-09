//! Analysis API entry points.

use crate::domain::error::AiError;
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse};

/// Runs diff analysis using a provided AI provider.
pub fn analyze_diff(
    provider: &dyn AiProvider,
    req: AnalyzeDiffRequest,
) -> Result<AnalyzeDiffResponse, AiError> {
    provider.analyze_diff(req)
}
