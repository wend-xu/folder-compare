//! Mock provider implementation used in tests and local development.

use crate::domain::error::AiError;
use crate::domain::provider::AiProvider;
use crate::domain::types::{
    AnalysisTask, AnalyzeDiffRequest, AnalyzeDiffResponse, PromptPayload, RiskLevel,
};

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
    fn analyze_diff(
        &self,
        req: AnalyzeDiffRequest,
        _prompt: PromptPayload,
    ) -> Result<AnalyzeDiffResponse, AiError> {
        if req.diff_excerpt.trim().is_empty() {
            return Err(AiError::InvalidRequest {
                kind: crate::domain::error::InvalidRequestKind::EmptyDiffExcerpt,
            });
        }

        let target = req.relative_path.as_deref().unwrap_or("(unknown path)");
        let risk_level = estimate_risk_level(&req);
        let (title, rationale, key_points, review_suggestions) = match req.task {
            AnalysisTask::Summary => (
                format!("Summary for {target}"),
                "Mock provider generated a deterministic change summary.".to_string(),
                vec![
                    format!("Task=Summary for {target}"),
                    format!("Diff chars={}", req.diff_excerpt.chars().count()),
                ],
                vec![
                    "Review the summary against full diff if available.".to_string(),
                    "Use detailed diff panel for line-level context.".to_string(),
                ],
            ),
            AnalysisTask::RiskReview => (
                format!("Risk review for {target}"),
                "Mock provider estimated risk using deterministic keyword and size heuristics."
                    .to_string(),
                vec![
                    format!("Estimated risk={risk_level:?}"),
                    format!("Diff chars={}", req.diff_excerpt.chars().count()),
                ],
                vec![
                    "Check changed logic paths and error handling.".to_string(),
                    "Confirm tests cover new branches.".to_string(),
                ],
            ),
            AnalysisTask::ReviewComments => (
                format!("Review comments for {target}"),
                "Mock provider produced deterministic review comments.".to_string(),
                vec![
                    "Prefer smaller focused commits for easier review.".to_string(),
                    "Keep naming and formatting consistent with nearby code.".to_string(),
                ],
                vec![
                    "Add or update tests for modified behavior.".to_string(),
                    "Document intent for non-obvious logic changes.".to_string(),
                ],
            ),
        };

        Ok(AnalyzeDiffResponse {
            risk_level,
            title,
            rationale,
            key_points,
            review_suggestions,
        })
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

fn estimate_risk_level(req: &AnalyzeDiffRequest) -> RiskLevel {
    let excerpt = req.diff_excerpt.to_ascii_lowercase();
    let has_high_risk_marker = excerpt.contains("unsafe")
        || excerpt.contains("panic!")
        || excerpt.contains("unwrap(")
        || excerpt.contains("std::mem::transmute");
    if has_high_risk_marker {
        return RiskLevel::High;
    }

    let diff_size = req.diff_excerpt.chars().count();
    let summary_total = req
        .summary
        .as_ref()
        .map(|s| s.added_lines + s.removed_lines)
        .unwrap_or(0);
    if diff_size > 2_000 || summary_total > 120 {
        return RiskLevel::Medium;
    }

    RiskLevel::Low
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{AiConfig, AnalysisTask};

    #[test]
    fn mock_provider_supports_multiple_tasks_deterministically() {
        let provider = MockAiProvider::new();
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::Summary,
            relative_path: Some("src/main.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "-old\n+new".to_string(),
            summary: None,
            truncation_note: None,
            config: AiConfig::default(),
        };
        let prompt = PromptPayload {
            system_instruction: "sys".to_string(),
            user_prompt: "user".to_string(),
        };

        let a = provider
            .analyze_diff(req.clone(), prompt.clone())
            .expect("first call should succeed");
        let b = provider
            .analyze_diff(req, prompt)
            .expect("second call should succeed");
        assert_eq!(a, b);
        assert!(!a.key_points.is_empty());
        assert!(!a.review_suggestions.is_empty());
    }

    #[test]
    fn risk_review_detects_high_risk_markers() {
        let provider = MockAiProvider::new();
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::RiskReview,
            relative_path: Some("src/unsafe.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "+unsafe { panic!(\"boom\"); }".to_string(),
            summary: None,
            truncation_note: None,
            config: AiConfig::default(),
        };

        let response = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect("analysis should succeed");
        assert_eq!(response.risk_level, RiskLevel::High);
    }
}
