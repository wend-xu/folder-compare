//! UI command definitions.

use fc_core::{compare_dirs, CompareReport, CompareRequest};

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
}

/// Executes one compare request against `fc-core`.
pub fn run_compare(req: CompareRequest) -> Result<CompareReport, String> {
    compare_dirs(req).map_err(|err| err.to_string())
}
