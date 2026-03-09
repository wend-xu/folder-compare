//! Mock provider implementation used in tests and local development.

use crate::domain::error::AiError;
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse, RiskLevel};

/// Deterministic mock provider.
#[derive(Debug, Default)]
pub struct MockAiProvider;

impl MockAiProvider {
    /// Creates a new mock provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl AiProvider for MockAiProvider {
    fn analyze_diff(&self, req: AnalyzeDiffRequest) -> Result<AnalyzeDiffResponse, AiError> {
        if req.diff_excerpt.trim().is_empty() {
            return Err(AiError::InvalidRequest(
                "diff_excerpt must not be empty".to_string(),
            ));
        }

        Ok(AnalyzeDiffResponse {
            risk_level: RiskLevel::Low,
            title: "Mock analysis".to_string(),
            rationale: "Deterministic placeholder output from MockAiProvider.".to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}
