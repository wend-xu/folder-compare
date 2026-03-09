//! Analysis API entry points.

use crate::domain::error::AiError;
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse};
use crate::services::analyzer::Analyzer;

/// Runs diff analysis using a provided AI provider.
pub fn analyze_diff(
    provider: &dyn AiProvider,
    req: AnalyzeDiffRequest,
) -> Result<AnalyzeDiffResponse, AiError> {
    Analyzer::new(provider).run(req)
}
