//! Error model for AI analysis layer.

use thiserror::Error;

/// Error type for `fc-ai` APIs.
#[derive(Debug, Error)]
pub enum AiError {
    /// Request is invalid.
    #[error("invalid request: {kind}")]
    InvalidRequest {
        /// Structured invalid request kind.
        kind: InvalidRequestKind,
    },
    /// Prompt payload construction failed.
    #[error("prompt build failed: {reason}")]
    PromptBuildFailed {
        /// Structured prompt build failure reason.
        reason: PromptBuildFailureKind,
    },
    /// Input preparation stage failed before provider call.
    #[error("input preparation failed: {reason}")]
    InputPreparationFailed {
        /// Structured input preparation failure reason.
        reason: InputPreparationFailureKind,
    },
    /// Provider returned an execution failure.
    #[error("provider execution failed ({provider}): {message}")]
    ProviderExecutionFailed {
        /// Provider identifier.
        provider: String,
        /// Failure message.
        message: String,
    },
    /// Provider response could not be normalized/parsed.
    #[error("response parse failed ({provider}): {message}")]
    ResponseParseFailed {
        /// Provider identifier.
        provider: String,
        /// Parse failure message.
        message: String,
    },
    /// Feature is intentionally not implemented yet.
    #[error("not implemented: {feature}")]
    NotImplemented {
        /// Feature name or capability marker.
        feature: &'static str,
    },
}

/// Invalid request category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidRequestKind {
    /// Diff excerpt is empty.
    EmptyDiffExcerpt,
    /// Input character budget is zero.
    InvalidInputBudget,
    /// Output token budget is zero.
    InvalidOutputBudget,
    /// Temperature is outside supported range.
    InvalidTemperature,
}

impl std::fmt::Display for InvalidRequestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::EmptyDiffExcerpt => "diff excerpt is empty",
            Self::InvalidInputBudget => "max_input_chars must be greater than zero",
            Self::InvalidOutputBudget => "max_output_tokens must be greater than zero",
            Self::InvalidTemperature => "temperature must be between 0.0 and 2.0",
        };
        write!(f, "{text}")
    }
}

/// Prompt build failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptBuildFailureKind {
    /// Prompt builder produced empty output.
    EmptyPromptPayload,
}

impl std::fmt::Display for PromptBuildFailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::EmptyPromptPayload => "prompt payload is empty",
        };
        write!(f, "{text}")
    }
}

/// Input preparation failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputPreparationFailureKind {
    /// Input budget is invalid.
    InvalidBudget,
}

impl std::fmt::Display for InputPreparationFailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::InvalidBudget => "input budget is invalid",
        };
        write!(f, "{text}")
    }
}
