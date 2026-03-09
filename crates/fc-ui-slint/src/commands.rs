//! UI command definitions.

use fc_core::{
    compare_dirs, diff_text_file, CompareReport, CompareRequest, TextDiffRequest, TextDiffResult,
};

/// Commands emitted by UI interactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCommand {
    /// Initializes presenter state.
    Initialize,
    /// Updates left root path from UI input.
    UpdateLeftRoot(String),
    /// Updates right root path from UI input.
    UpdateRightRoot(String),
    /// Triggers directory compare.
    RunCompare,
    /// Updates selected result row.
    SelectRow(i32),
    /// Loads detailed diff for selected row.
    LoadSelectedDiff,
}

/// Executes one compare request against `fc-core`.
pub fn run_compare(req: CompareRequest) -> Result<CompareReport, String> {
    compare_dirs(req).map_err(|err| err.to_string())
}

/// Executes one text diff request against `fc-core`.
pub fn run_text_diff(req: TextDiffRequest) -> Result<TextDiffResult, String> {
    diff_text_file(req).map_err(|err| err.to_string())
}
