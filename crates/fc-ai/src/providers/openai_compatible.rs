//! OpenAI-compatible provider placeholder.

use crate::domain::error::AiError;
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse};

/// Placeholder provider for an OpenAI-compatible backend.
#[derive(Debug, Default)]
pub struct OpenAiCompatibleProvider;

impl OpenAiCompatibleProvider {
    /// Creates a new placeholder provider.
    pub fn new() -> Self {
        Self
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    fn analyze_diff(&self, _req: AnalyzeDiffRequest) -> Result<AnalyzeDiffResponse, AiError> {
        Err(AiError::NotImplemented(
            "OpenAI-compatible provider is not implemented in Phase 1".to_string(),
        ))
    }

    fn name(&self) -> &'static str {
        "openai-compatible-placeholder"
    }
}
