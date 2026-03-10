//! Analyzer orchestration service.

use crate::domain::error::{
    AiError, InputPreparationFailureKind, InvalidRequestKind, PromptBuildFailureKind,
    ResponseParseFailureKind,
};
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse};
use crate::services::prompt;
use crate::services::truncation;

/// Analyzer service that validates, prepares input, and delegates to provider.
pub struct Analyzer<'a> {
    provider: &'a dyn AiProvider,
}

impl<'a> Analyzer<'a> {
    /// Creates an analyzer from a provider reference.
    pub fn new(provider: &'a dyn AiProvider) -> Self {
        Self { provider }
    }

    /// Runs analysis with request validation and prompt/input orchestration.
    pub fn run(&self, req: AnalyzeDiffRequest) -> Result<AnalyzeDiffResponse, AiError> {
        self.validate_request(&req)?;

        if req.config.max_input_chars == 0 {
            return Err(AiError::InputPreparationFailed {
                reason: InputPreparationFailureKind::InvalidBudget,
            });
        }
        let prepared =
            truncation::prepare_diff_excerpt(&req.diff_excerpt, req.config.max_input_chars);

        let mut prepared_req = req;
        prepared_req.diff_excerpt = prepared.prepared_excerpt;
        prepared_req.truncation_note = merge_truncation_notes(
            prepared_req.truncation_note.take(),
            prepared.truncation_note,
        );

        let prompt_payload = prompt::build_prompt_payload(&prepared_req);
        if prompt_payload.system_instruction.trim().is_empty()
            || prompt_payload.user_prompt.trim().is_empty()
        {
            return Err(AiError::PromptBuildFailed {
                reason: PromptBuildFailureKind::EmptyPromptPayload,
            });
        }

        let provider_name = self.provider.name().to_string();
        let response = self.provider.analyze_diff(prepared_req, prompt_payload)?;
        normalize_response(response, &provider_name)
    }

    fn validate_request(&self, req: &AnalyzeDiffRequest) -> Result<(), AiError> {
        if req.diff_excerpt.trim().is_empty() {
            return Err(AiError::InvalidRequest {
                kind: InvalidRequestKind::EmptyDiffExcerpt,
            });
        }
        if req.config.max_input_chars == 0 {
            return Err(AiError::InvalidRequest {
                kind: InvalidRequestKind::InvalidInputBudget,
            });
        }
        if req.config.max_output_tokens == 0 {
            return Err(AiError::InvalidRequest {
                kind: InvalidRequestKind::InvalidOutputBudget,
            });
        }
        if !(0.0..=2.0).contains(&req.config.temperature) {
            return Err(AiError::InvalidRequest {
                kind: InvalidRequestKind::InvalidTemperature,
            });
        }

        Ok(())
    }
}

fn merge_truncation_notes(existing: Option<String>, generated: Option<String>) -> Option<String> {
    match (existing, generated) {
        (Some(left), Some(right)) => Some(format!("{left}; {right}")),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn normalize_response(
    response: AnalyzeDiffResponse,
    provider_name: &str,
) -> Result<AnalyzeDiffResponse, AiError> {
    let AnalyzeDiffResponse {
        risk_level,
        title,
        rationale,
        key_points,
        review_suggestions,
    } = response;
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(AiError::ResponseParseFailed {
            provider: provider_name.to_string(),
            kind: ResponseParseFailureKind::InvalidContract,
            message: "title is empty".to_string(),
        });
    }

    let rationale = rationale.trim().to_string();
    if rationale.is_empty() {
        return Err(AiError::ResponseParseFailed {
            provider: provider_name.to_string(),
            kind: ResponseParseFailureKind::InvalidContract,
            message: "rationale is empty".to_string(),
        });
    }

    let key_points = key_points
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    let review_suggestions = review_suggestions
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();

    Ok(AnalyzeDiffResponse {
        risk_level,
        title,
        rationale,
        key_points,
        review_suggestions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{AiConfig, AnalysisTask, PromptPayload, RiskLevel};
    use std::sync::Mutex;

    #[derive(Default)]
    struct CaptureProvider {
        captured_excerpt: Mutex<Option<String>>,
        captured_prompt: Mutex<Option<String>>,
    }

    impl AiProvider for CaptureProvider {
        fn analyze_diff(
            &self,
            req: AnalyzeDiffRequest,
            prompt: PromptPayload,
        ) -> Result<AnalyzeDiffResponse, AiError> {
            *self.captured_excerpt.lock().expect("lock should succeed") = Some(req.diff_excerpt);
            *self.captured_prompt.lock().expect("lock should succeed") = Some(prompt.user_prompt);
            Ok(AnalyzeDiffResponse {
                risk_level: RiskLevel::Low,
                title: "ok".to_string(),
                rationale: "ok".to_string(),
                key_points: vec!["k1".to_string()],
                review_suggestions: vec!["s1".to_string()],
            })
        }

        fn name(&self) -> &'static str {
            "capture"
        }
    }

    #[test]
    fn analyzer_validates_empty_diff_excerpt() {
        let provider = CaptureProvider::default();
        let analyzer = Analyzer::new(&provider);
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::Summary,
            relative_path: None,
            language_hint: None,
            diff_excerpt: "   ".to_string(),
            summary: None,
            truncation_note: None,
            config: AiConfig::default(),
        };

        let err = analyzer.run(req).expect_err("empty excerpt should fail");
        assert!(matches!(
            err,
            AiError::InvalidRequest {
                kind: InvalidRequestKind::EmptyDiffExcerpt
            }
        ));
    }

    #[test]
    fn analyzer_truncates_input_and_calls_provider_with_prompt() {
        let provider = CaptureProvider::default();
        let analyzer = Analyzer::new(&provider);
        let mut config = AiConfig::default();
        config.max_input_chars = 4;
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::Summary,
            relative_path: Some("src/lib.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "abcdef".to_string(),
            summary: None,
            truncation_note: None,
            config,
        };

        let response = analyzer.run(req).expect("analyzer should succeed");
        assert_eq!(response.title, "ok");
        assert_eq!(
            provider
                .captured_excerpt
                .lock()
                .expect("lock should succeed")
                .as_deref(),
            Some("abcd")
        );
        let prompt = provider
            .captured_prompt
            .lock()
            .expect("lock should succeed")
            .clone()
            .expect("prompt should be captured");
        assert!(prompt.contains("Target Path: src/lib.rs"));
        assert!(prompt.contains("Truncation Note: diff excerpt truncated from 6 chars to 4 chars"));
    }
}
