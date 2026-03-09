//! Provider abstraction for pluggable AI backends.

use crate::domain::error::AiError;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse, PromptPayload};

/// AI provider interface for diff analysis.
pub trait AiProvider: Send + Sync {
    /// Analyzes diff content and returns structured response.
    fn analyze_diff(
        &self,
        req: AnalyzeDiffRequest,
        prompt: PromptPayload,
    ) -> Result<AnalyzeDiffResponse, AiError>;

    /// Returns provider identifier for telemetry and debugging.
    fn name(&self) -> &'static str;
}
