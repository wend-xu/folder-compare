//! App state placeholders for UI composition.

use fc_ai::AnalyzeDiffResponse;
use fc_core::CompareSummary;

/// In-memory UI state for Phase 1.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    /// Last compare summary if available.
    pub last_compare_summary: Option<CompareSummary>,
    /// Last AI analysis result if available.
    pub last_analysis: Option<AnalyzeDiffResponse>,
    /// Plain status text for rendering.
    pub status_text: String,
}
