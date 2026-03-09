//! AI analysis layer for diff interpretation.

pub mod api;
pub mod domain;
pub mod providers;
pub mod services;

pub use api::analyze::analyze_diff;
pub use domain::error::AiError;
pub use domain::provider::AiProvider;
pub use domain::types::{
    AiConfig, AiProviderKind, AnalysisTask, AnalyzeDiffRequest, AnalyzeDiffResponse, RiskLevel,
};
pub use services::mock_provider::MockAiProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_and_response_can_be_constructed() {
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::RiskReview,
            diff_excerpt: "-old\n+new".to_string(),
            summary: None,
            config: AiConfig::default(),
        };
        let resp = AnalyzeDiffResponse {
            risk_level: RiskLevel::Low,
            title: "placeholder".to_string(),
            rationale: "no-op".to_string(),
        };

        assert_eq!(req.task, AnalysisTask::RiskReview);
        assert_eq!(resp.risk_level, RiskLevel::Low);
    }

    #[test]
    fn mock_provider_smoke() {
        let provider = MockAiProvider::new();
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::RiskReview,
            diff_excerpt: "fn old() {} -> fn new() {}".to_string(),
            summary: None,
            config: AiConfig::default(),
        };

        let resp = provider
            .analyze_diff(req)
            .expect("mock provider should return deterministic response");

        assert_eq!(provider.name(), "mock");
        assert_eq!(resp.risk_level, RiskLevel::Low);
    }
}
