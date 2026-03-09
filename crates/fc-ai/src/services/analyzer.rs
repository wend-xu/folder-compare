//! Analyzer orchestration placeholder.

use crate::domain::error::AiError;
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse};

/// Thin analyzer service that delegates to a provider.
pub struct Analyzer<'a> {
    provider: &'a dyn AiProvider,
}

impl<'a> Analyzer<'a> {
    /// Creates an analyzer from a provider reference.
    pub fn new(provider: &'a dyn AiProvider) -> Self {
        Self { provider }
    }

    /// Runs analysis with no extra orchestration yet.
    pub fn run(&self, req: AnalyzeDiffRequest) -> Result<AnalyzeDiffResponse, AiError> {
        self.provider.analyze_diff(req)
    }
}
