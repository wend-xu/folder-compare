use fc_ai::{
    AiConfig, AiError, AiProvider, AnalysisTask, AnalyzeDiffRequest, AnalyzeDiffResponse,
    InvalidRequestKind, PromptPayload, ProviderExecutionFailureKind, RiskLevel, analyze_diff,
};
use std::sync::Mutex;

fn base_request(task: AnalysisTask, excerpt: &str) -> AnalyzeDiffRequest {
    AnalyzeDiffRequest {
        task,
        relative_path: Some("src/lib.rs".to_string()),
        language_hint: Some("rust".to_string()),
        diff_excerpt: excerpt.to_string(),
        summary: Some(fc_core::TextDiffSummary {
            hunk_count: 1,
            added_lines: 3,
            removed_lines: 1,
            context_lines: 2,
        }),
        truncation_note: None,
        config: AiConfig::default(),
    }
}

#[test]
fn dto_and_default_config_are_serde_friendly() {
    let req = base_request(AnalysisTask::Summary, "-old\n+new");
    let resp = AnalyzeDiffResponse {
        risk_level: RiskLevel::Low,
        title: "Summary".to_string(),
        rationale: "Looks safe.".to_string(),
        key_points: vec!["kp1".to_string()],
        review_suggestions: vec!["rs1".to_string()],
    };

    let req_json = serde_json::to_string(&req).expect("request should serialize");
    let req_back: AnalyzeDiffRequest =
        serde_json::from_str(&req_json).expect("request should deserialize");
    assert_eq!(req, req_back);

    let resp_json = serde_json::to_string(&resp).expect("response should serialize");
    let resp_back: AnalyzeDiffResponse =
        serde_json::from_str(&resp_json).expect("response should deserialize");
    assert_eq!(resp, resp_back);

    let config = AiConfig::default();
    assert_eq!(config.max_input_chars, 12_000);
    assert_eq!(config.max_output_tokens, 512);
    assert_eq!(config.temperature, 0.0);
}

#[test]
fn invalid_request_is_reported_structurally() {
    let provider = fc_ai::MockAiProvider::new();
    let req = base_request(AnalysisTask::Summary, " ");

    let err = analyze_diff(&provider, req).expect_err("empty diff should fail");
    assert!(matches!(
        err,
        AiError::InvalidRequest {
            kind: InvalidRequestKind::EmptyDiffExcerpt
        }
    ));
}

struct FailingProvider;

impl AiProvider for FailingProvider {
    fn analyze_diff(
        &self,
        _req: AnalyzeDiffRequest,
        _prompt: PromptPayload,
    ) -> Result<AnalyzeDiffResponse, AiError> {
        Err(AiError::ProviderExecutionFailed {
            provider: "failing".to_string(),
            kind: ProviderExecutionFailureKind::NetworkFailure,
            message: "simulated failure".to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "failing"
    }
}

#[test]
fn provider_failure_is_passed_through() {
    let req = base_request(AnalysisTask::Summary, "-a\n+b");
    let err = analyze_diff(&FailingProvider, req).expect_err("provider should fail");
    assert!(matches!(err, AiError::ProviderExecutionFailed { .. }));
}

struct ParseBadProvider;

impl AiProvider for ParseBadProvider {
    fn analyze_diff(
        &self,
        _req: AnalyzeDiffRequest,
        _prompt: PromptPayload,
    ) -> Result<AnalyzeDiffResponse, AiError> {
        Ok(AnalyzeDiffResponse {
            risk_level: RiskLevel::Low,
            title: "   ".to_string(),
            rationale: "has rationale".to_string(),
            key_points: vec![],
            review_suggestions: vec![],
        })
    }

    fn name(&self) -> &'static str {
        "parse-bad"
    }
}

#[test]
fn parse_failure_is_reported_structurally() {
    let req = base_request(AnalysisTask::Summary, "-a\n+b");
    let err = analyze_diff(&ParseBadProvider, req).expect_err("parse should fail");
    assert!(matches!(err, AiError::ResponseParseFailed { .. }));
}

#[test]
fn mock_provider_is_deterministic_for_multiple_tasks() {
    let provider = fc_ai::MockAiProvider::new();
    for task in [
        AnalysisTask::Summary,
        AnalysisTask::RiskReview,
        AnalysisTask::ReviewComments,
    ] {
        let req = base_request(task, "+unsafe { panic!(\"x\"); }");
        let a = analyze_diff(&provider, req.clone()).expect("first call should succeed");
        let b = analyze_diff(&provider, req).expect("second call should succeed");
        assert_eq!(a, b);
        assert!(!a.key_points.is_empty());
        assert!(!a.review_suggestions.is_empty());
    }
}

#[test]
fn openai_provider_requires_remote_configuration() {
    let provider = fc_ai::providers::openai_compatible::OpenAiCompatibleProvider::new();
    let req = base_request(AnalysisTask::Summary, "-a\n+b");
    let err = analyze_diff(&provider, req).expect_err("provider should require endpoint");
    assert!(matches!(
        err,
        AiError::ProviderExecutionFailed {
            kind: ProviderExecutionFailureKind::MissingEndpoint,
            ..
        }
    ));
}

#[derive(Default)]
struct CaptureProvider {
    req_excerpt: Mutex<Option<String>>,
    req_note: Mutex<Option<String>>,
    prompt_body: Mutex<Option<String>>,
}

impl AiProvider for CaptureProvider {
    fn analyze_diff(
        &self,
        req: AnalyzeDiffRequest,
        prompt: PromptPayload,
    ) -> Result<AnalyzeDiffResponse, AiError> {
        *self.req_excerpt.lock().expect("lock should succeed") = Some(req.diff_excerpt);
        *self.req_note.lock().expect("lock should succeed") = req.truncation_note;
        *self.prompt_body.lock().expect("lock should succeed") = Some(prompt.user_prompt);
        Ok(AnalyzeDiffResponse {
            risk_level: RiskLevel::Low,
            title: "ok".to_string(),
            rationale: "ok".to_string(),
            key_points: vec!["k".to_string()],
            review_suggestions: vec!["s".to_string()],
        })
    }

    fn name(&self) -> &'static str {
        "capture"
    }
}

#[test]
fn analyzer_orchestrates_truncation_and_prompt_build() {
    let provider = CaptureProvider::default();
    let mut req = base_request(AnalysisTask::Summary, "abcdef");
    req.config.max_input_chars = 4;

    let _ = analyze_diff(&provider, req).expect("analysis should succeed");
    assert_eq!(
        provider
            .req_excerpt
            .lock()
            .expect("lock should succeed")
            .as_deref(),
        Some("abcd")
    );
    assert!(
        provider
            .req_note
            .lock()
            .expect("lock should succeed")
            .as_deref()
            .unwrap_or_default()
            .contains("truncated from 6 chars to 4 chars")
    );
    assert!(
        provider
            .prompt_body
            .lock()
            .expect("lock should succeed")
            .as_deref()
            .unwrap_or_default()
            .contains("Target Path: src/lib.rs")
    );
}
