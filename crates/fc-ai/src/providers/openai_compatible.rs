//! OpenAI-compatible provider placeholder.

use crate::domain::error::AiError;
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse, PromptPayload};

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
    fn analyze_diff(
        &self,
        _req: AnalyzeDiffRequest,
        _prompt: PromptPayload,
    ) -> Result<AnalyzeDiffResponse, AiError> {
        Err(AiError::NotImplemented {
            feature: "openai-compatible provider execution",
        })
    }

    fn name(&self) -> &'static str {
        "openai-compatible"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{AiConfig, AnalysisTask};

    #[test]
    fn placeholder_provider_returns_not_implemented() {
        let provider = OpenAiCompatibleProvider::new();
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::Summary,
            relative_path: Some("src/lib.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "-a\n+b".to_string(),
            summary: None,
            truncation_note: None,
            config: AiConfig::default(),
        };

        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("placeholder provider should be not implemented");
        assert!(matches!(err, AiError::NotImplemented { .. }));
    }
}
