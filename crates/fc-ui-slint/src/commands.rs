//! UI command definitions.

use crate::state::NavigatorViewMode;
use fc_ai::providers::openai_compatible::OpenAiCompatibleProvider;
use fc_ai::services::analyzer::Analyzer;
use fc_ai::{AiProviderKind, AnalyzeDiffRequest, AnalyzeDiffResponse, MockAiProvider};
use fc_core::{
    CompareReport, CompareRequest, TextDiffRequest, TextDiffResult, compare_dirs, diff_text_file,
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
    /// Updates compare row filter text.
    UpdateEntryFilter(String),
    /// Updates compare row status scope filter.
    UpdateEntryStatusFilter(String),
    /// Switches non-search Results / Navigator mode to tree.
    SetNavigatorViewModeTree,
    /// Switches non-search Results / Navigator mode to flat.
    SetNavigatorViewModeFlat,
    /// Toggles the top-level sidebar shell visibility.
    ToggleSidebarVisibility,
    /// Toggles one directory node in tree mode.
    ToggleNavigatorTreeNode(String),
    /// Updates selected result row.
    SelectRow(i32),
    /// Loads detailed diff for selected row.
    LoadSelectedDiff,
    /// Reveals one flat result in tree mode, clearing search when needed, and opens its file view.
    LocateAndOpen(String),
    /// Opens one directory compare target in workspace Compare View.
    /// Empty string means the compare root itself.
    OpenCompareView(String),
    /// Activates one outer workspace session tab.
    SelectWorkspaceSession(String),
    /// Closes one outer workspace session tab.
    CloseWorkspaceSession(String),
    /// Confirms one pending workspace session transition.
    ConfirmWorkspaceSessionAction,
    /// Cancels one pending workspace session transition.
    CancelWorkspaceSessionAction,
    /// Moves current Compare View target to its parent directory.
    CompareViewUpOneLevel,
    /// Reanchors current Compare View to one ancestor/root breadcrumb target without resetting the session.
    /// Empty string means the compare root itself.
    NavigateCompareView(String),
    /// Updates Compare Tree quick-locate query and jumps to the current/first match in the current anchor.
    UpdateCompareViewQuickLocate(String),
    /// Jumps to the next quick-locate match inside the current Compare Tree anchor.
    AdvanceCompareViewQuickLocate,
    /// Toggles one expandable directory row inside Compare View.
    ToggleCompareTreeNode(String),
    /// Focuses one visible row inside Compare View.
    FocusCompareRow(String),
    /// Toggles Compare Tree horizontal scroll lock between locked/unlocked modes.
    ToggleCompareViewScrollLock,
    /// Opens one file/special-entry row in File View while preserving Compare View return context.
    OpenFileViewFromCompare(String),
    /// Switches current file shell back to Diff.
    SetFileViewModeDiff,
    /// Switches current file shell to Analysis.
    SetFileViewModeAnalysis,
    /// Loads AI analysis for selected detailed diff.
    LoadAiAnalysis,
    /// Switches AI provider mode to mock.
    SetAiProviderModeMock,
    /// Switches AI provider mode to OpenAI-compatible.
    SetAiProviderModeOpenAiCompatible,
    /// Updates OpenAI-compatible endpoint input.
    UpdateAiEndpoint(String),
    /// Updates OpenAI-compatible API key input.
    UpdateAiApiKey(String),
    /// Updates OpenAI-compatible model input.
    UpdateAiModel(String),
    /// Saves application settings using dialog draft values.
    SaveAppSettings {
        provider_kind: AiProviderKind,
        endpoint: String,
        api_key: String,
        model: String,
        timeout_secs_text: String,
        show_hidden_files: bool,
        default_results_view: NavigatorViewMode,
    },
    /// Clears settings validation/persistence error.
    ClearSettingsError,
}

/// Executes one compare request against `fc-core`.
pub fn run_compare(req: CompareRequest) -> Result<CompareReport, String> {
    compare_dirs(req).map_err(|err| err.to_string())
}

/// Executes one text diff request against `fc-core`.
pub fn run_text_diff(req: TextDiffRequest) -> Result<TextDiffResult, String> {
    diff_text_file(req).map_err(|err| err.to_string())
}

/// Executes one AI analysis request against `fc-ai` provider via analyzer.
pub fn run_ai_analysis(req: AnalyzeDiffRequest) -> Result<AnalyzeDiffResponse, String> {
    match req.config.provider_kind {
        AiProviderKind::Mock => {
            let provider = MockAiProvider::new();
            Analyzer::new(&provider)
                .run(req)
                .map_err(|err| err.to_string())
        }
        AiProviderKind::OpenAiCompatible => {
            let provider = OpenAiCompatibleProvider::new();
            Analyzer::new(&provider)
                .run(req)
                .map_err(|err| err.to_string())
        }
    }
}
