//! AI analysis layer for diff interpretation.

pub mod api;
pub mod domain;
pub mod providers;
pub mod services;

pub use api::analyze::analyze_diff;
pub use domain::error::{
    AiError, InputPreparationFailureKind, InvalidRequestKind, PromptBuildFailureKind,
    ProviderExecutionFailureKind, ResponseParseFailureKind,
};
pub use domain::provider::AiProvider;
pub use domain::types::{
    AiConfig, AiProviderKind, AnalysisTask, AnalyzeDiffRequest, AnalyzeDiffResponse, PromptPayload,
    RiskLevel,
};
pub use services::mock_provider::MockAiProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_and_response_can_be_constructed() {
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::RiskReview,
            relative_path: Some("src/lib.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "-old\n+new".to_string(),
            summary: None,
            truncation_note: None,
            config: AiConfig::default(),
        };
        let resp = AnalyzeDiffResponse {
            risk_level: RiskLevel::Low,
            title: "placeholder".to_string(),
            rationale: "no-op".to_string(),
            key_points: vec!["point".to_string()],
            review_suggestions: vec!["suggestion".to_string()],
        };

        assert_eq!(req.task, AnalysisTask::RiskReview);
        assert_eq!(req.relative_path.as_deref(), Some("src/lib.rs"));
        assert_eq!(resp.risk_level, RiskLevel::Low);
    }

    #[test]
    fn ai_config_default_is_stable() {
        let config = AiConfig::default();
        assert_eq!(config.provider_kind, AiProviderKind::Mock);
        assert_eq!(config.max_input_chars, 12_000);
        assert_eq!(config.max_output_tokens, 512);
        assert_eq!(config.temperature, 0.0);
        assert!(config.openai_endpoint.is_none());
        assert!(config.openai_api_key.is_none());
        assert_eq!(config.openai_model.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn mock_provider_smoke() {
        let provider = MockAiProvider::new();
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::RiskReview,
            relative_path: Some("src/main.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "fn old() {} -> fn new() {}".to_string(),
            summary: None,
            truncation_note: None,
            config: AiConfig::default(),
        };

        let resp = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect("mock provider should return deterministic response");

        assert_eq!(provider.name(), "mock");
        assert_eq!(resp.risk_level, RiskLevel::Low);
        assert!(!resp.key_points.is_empty());
    }
}
