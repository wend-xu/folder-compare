//! Domain models for AI analysis.

pub mod error;
pub mod provider;
pub mod types;

pub use error::{AiError, InputPreparationFailureKind, InvalidRequestKind, PromptBuildFailureKind};
pub use provider::AiProvider;
pub use types::{
    AiConfig, AiProviderKind, AnalysisTask, AnalyzeDiffRequest, AnalyzeDiffResponse, PromptPayload,
    RiskLevel,
};
