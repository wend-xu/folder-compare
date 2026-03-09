//! UI command definitions.

/// Commands emitted by UI interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCommand {
    /// Initializes presenter state.
    Initialize,
    /// Placeholder compare command.
    RunComparePlaceholder,
}
