//! Request and response models for AI analysis.

use fc_core::TextDiffSummary;
use serde::{Deserialize, Serialize};

/// Request payload for AI diff analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeDiffRequest {
    /// Analysis task category.
    pub task: AnalysisTask,
    /// Relative file path associated with this diff.
    pub relative_path: Option<String>,
    /// Optional language hint for provider prompt context.
    pub language_hint: Option<String>,
    /// Raw diff excerpt for model context.
    pub diff_excerpt: String,
    /// Optional structured summary from `fc-core`.
    pub summary: Option<TextDiffSummary>,
    /// Optional truncation marker from upstream preparation stages.
    pub truncation_note: Option<String>,
    /// Runtime AI config.
    pub config: AiConfig,
}

/// AI-produced analysis response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalyzeDiffResponse {
    /// Estimated risk level.
    pub risk_level: RiskLevel,
    /// Short headline for UI display.
    pub title: String,
    /// Human-readable reasoning text.
    pub rationale: String,
    /// Key bullet points for compact UI card display.
    pub key_points: Vec<String>,
    /// Follow-up review suggestions for users.
    pub review_suggestions: Vec<String>,
}

/// Supported analysis task types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisTask {
    /// Summarize what changed.
    Summary,
    /// Assess potential risk.
    RiskReview,
    /// Produce code-review style suggestions.
    ReviewComments,
}

/// Risk rating returned by the AI layer.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    /// Low risk.
    Low,
    /// Medium risk.
    Medium,
    /// High risk.
    High,
}

/// AI runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiConfig {
    /// Selected provider kind.
    pub provider_kind: AiProviderKind,
    /// Maximum characters allowed for prepared input excerpt.
    pub max_input_chars: usize,
    /// Token budget for responses.
    pub max_output_tokens: usize,
    /// Sampling temperature.
    pub temperature: f32,
    /// Optional OpenAI-compatible endpoint root.
    pub openai_endpoint: Option<String>,
    /// Optional OpenAI-compatible API key.
    pub openai_api_key: Option<String>,
    /// Optional OpenAI-compatible model id.
    pub openai_model: Option<String>,
    /// HTTP request timeout in seconds for remote provider calls.
    pub request_timeout_secs: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider_kind: AiProviderKind::Mock,
            max_input_chars: 12_000,
            max_output_tokens: 512,
            temperature: 0.0,
            openai_endpoint: None,
            openai_api_key: None,
            openai_model: Some("gpt-4o-mini".to_string()),
            request_timeout_secs: 30,
        }
    }
}

/// Provider implementation kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiProviderKind {
    /// Deterministic local mock provider.
    Mock,
    /// OpenAI-compatible provider placeholder.
    OpenAiCompatible,
}

/// Provider-neutral prompt payload built by analyzer services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptPayload {
    /// Stable system instruction for the provider.
    pub system_instruction: String,
    /// User-facing analysis prompt content.
    pub user_prompt: String,
}
