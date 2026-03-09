//! Request and response models for AI analysis.

use fc_core::TextDiffSummary;
use serde::{Deserialize, Serialize};

/// Request payload for AI diff analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeDiffRequest {
    /// Analysis task category.
    pub task: AnalysisTask,
    /// Raw diff excerpt for model context.
    pub diff_excerpt: String,
    /// Optional structured summary from `fc-core`.
    pub summary: Option<TextDiffSummary>,
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
}

/// Supported analysis task types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisTask {
    /// Summarize what changed.
    Summary,
    /// Assess potential risk.
    RiskReview,
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
    /// Token budget for responses.
    pub max_output_tokens: usize,
    /// Sampling temperature.
    pub temperature: f32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider_kind: AiProviderKind::Mock,
            max_output_tokens: 512,
            temperature: 0.0,
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
