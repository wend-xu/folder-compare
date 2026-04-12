//! App state for compare + detailed diff UI workflow.

use crate::compare_foundation::{
    CompareBaseStatus, CompareFocusPath, CompareFoundation, CompareFoundationDetail,
    CompareFoundationNode, CompareNodeKind,
};
use crate::compare_tree::{
    compare_tree_expansion_state, compare_tree_reveal_targets, compare_tree_search_paths,
    compare_tree_toggle_target, project_compare_tree_rows,
};
use crate::navigator_tree::{
    NavigatorTreeProjection, NavigatorTreeRowProjection, navigator_tree_reveal_targets,
    navigator_tree_toggle_target, project_navigator_tree_rows,
};
use crate::view_models::{
    AnalysisResultViewModel, CompareEntryRowViewModel, CompareFilePanelViewModel,
    CompareFileRowViewModel, DiffPanelViewModel,
};
use fc_ai::{AiConfig, AiProviderKind};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

const WARNING_WRAP_COLUMNS: usize = 96;
const PATH_DISPLAY_MAX_CHARS: usize = 140;
const PATH_DISPLAY_HEAD_CHARS: usize = 90;
const PATH_DISPLAY_TAIL_CHARS: usize = 45;
const NAVIGATOR_PARENT_PATH_MAX_CHARS: usize = 52;
const NAVIGATOR_PARENT_PATH_HEAD_CHARS: usize = 18;
const NAVIGATOR_PARENT_PATH_TAIL_CHARS: usize = 28;
const NAVIGATOR_SECONDARY_MAX_CHARS: usize = 96;
const ROOT_PAIR_MAX_CHARS: usize = 54;
const ROOT_PAIR_HEAD_CHARS: usize = 30;
const ROOT_PAIR_TAIL_CHARS: usize = 16;
const COMPARE_ROOT_BREADCRUMB_LABEL: &str = "Compare Root";

/// Diff tab shell state for unified status rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffShellState {
    /// No row selected in Results / Navigator.
    NoSelection,
    /// A previous file path exists, but the row is no longer active in current results.
    StaleSelection,
    /// Diff or preview loading is in progress.
    Loading,
    /// Detailed diff payload is ready.
    DetailedReady,
    /// Preview payload is ready.
    PreviewReady,
    /// Selection is valid but this viewer cannot render content.
    Unavailable,
    /// Loading failed due to runtime error.
    Error,
}

/// Analysis tab shell state for unified status rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisPanelState {
    /// No row selected in Results / Navigator.
    NoSelection,
    /// A previous file path exists, but the row is no longer active in current results.
    StaleSelection,
    /// One row is selected and analysis is waiting for diff context.
    WaitingForDiff,
    /// One row is selected and analysis can start immediately.
    Ready,
    /// One row is selected, but analysis cannot start for this selection.
    Unavailable,
    /// AI analysis is currently running.
    Loading,
    /// AI analysis failed in the current session.
    Error,
    /// Structured analysis result is ready.
    Success,
}

/// Compare File View shell state for compare-originated file tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareFileShellState {
    /// No row selected in File View.
    NoSelection,
    /// Previously opened compare file is no longer active in current results.
    StaleSelection,
    /// Compare File View content is loading.
    Loading,
    /// Side-by-side compare rows are ready.
    Ready,
    /// Selection is valid but Compare File View cannot render content.
    Unavailable,
    /// Loading failed due to runtime error.
    Error,
}

/// One filtered Results / Navigator row with presentation-friendly fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigatorRowProjection {
    /// Source index in the unfiltered compare row vector.
    pub source_index: usize,
    /// Original compare row view model.
    pub row: CompareEntryRowViewModel,
    /// Primary label shown in the row (usually file name / leaf path segment).
    pub display_name: String,
    /// Weak path context used to disambiguate rows with the same display name.
    pub parent_path: String,
    /// Secondary summary explaining why this row is diff/equal/left/right.
    pub secondary_text: String,
    /// Row-level tooltip completion text (full filename + full parent path when present).
    pub tooltip_text: String,
    /// Whether current filter matched the display name.
    pub display_name_matches_filter: bool,
    /// Whether current filter matched the weak parent-path context.
    pub parent_path_matches_filter: bool,
}

/// One visible Compare View tree row projected from `compare_foundation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareViewRowProjection {
    /// Canonical node relative path.
    pub relative_path: String,
    /// Visible depth inside the current compare anchor subtree.
    pub depth: u16,
    /// Whether the left/base side exists for this child key.
    pub left_present: bool,
    /// Lightweight left-side icon token.
    pub left_icon: String,
    /// Left-side display name or placeholder.
    pub left_name: String,
    /// Compare status label rendered in the center column.
    pub status_label: String,
    /// Compare status tone token for restrained color styling.
    pub status_tone: String,
    /// Whether the right/modified side exists for this child key.
    pub right_present: bool,
    /// Lightweight right-side icon token.
    pub right_icon: String,
    /// Right-side display name or placeholder.
    pub right_name: String,
    /// Whether the row represents one directory target.
    pub is_directory: bool,
    /// Whether this row should show the compare-view disclosure affordance.
    pub is_expandable: bool,
    /// Whether the row's descendant subtree is currently expanded.
    pub is_expanded: bool,
}

/// Execution target for one Compare View row activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareViewRowAction {
    /// Toggle the child directory subtree in Compare View.
    ToggleDirectory,
    /// Open the child file/special entry in File View.
    OpenFileView,
    /// Row is a type mismatch and cannot open yet.
    TypeMismatch,
}

/// Runtime Results / Navigator view mode for non-search browsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigatorViewMode {
    Tree,
    Flat,
}

impl NavigatorViewMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tree => "tree",
            Self::Flat => "flat",
        }
    }
}

/// Outer workspace mode for the future Compare View / File View split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMode {
    FileView,
    #[allow(dead_code)]
    CompareView,
}

impl WorkspaceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileView => "file-view",
            Self::CompareView => "compare-view",
        }
    }
}

const COMPARE_TREE_SESSION_ID: &str = "compare-tree";

/// Outer workspace session kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSessionKind {
    CompareTree,
    File,
}

impl WorkspaceSessionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CompareTree => "compare-tree",
            Self::File => "file",
        }
    }
}

/// One visible outer workspace tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSession {
    /// Stable session identifier.
    pub session_id: String,
    /// Session kind used by presenter/UI branching.
    pub kind: WorkspaceSessionKind,
    /// Short tab label.
    pub label: String,
    /// Full tooltip text for truncated tab labels.
    pub tooltip_text: String,
    /// Whether the tab exposes close affordance.
    pub closable: bool,
}

/// File-session inner content mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSessionMode {
    Diff,
    Analysis,
}

impl FileSessionMode {
    pub fn tab_index(self) -> i32 {
        match self {
            Self::Diff => 0,
            Self::Analysis => 1,
        }
    }
}

/// The single compare-tree session carried by 19D workspace tabs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareTreeSession {
    /// Stable compare-session tab id.
    pub session_id: String,
    /// Left compare root snapshot for this session.
    pub left_root: String,
    /// Right compare root snapshot for this session.
    pub right_root: String,
    /// Compare-view focus anchor independent from file selection.
    pub compare_focus_path: CompareFocusPath,
    /// Focused visible compare-tree row inside Compare View.
    pub compare_row_focus_path: Option<String>,
    /// Expansion overrides for compare-tree directory nodes.
    pub expansion_overrides: BTreeMap<String, bool>,
    /// Whether Compare Tree horizontal scrolling is locked between left/right panes.
    pub horizontal_scroll_locked: bool,
}

impl CompareTreeSession {
    pub fn new(left_root: &str, right_root: &str) -> Self {
        Self {
            session_id: COMPARE_TREE_SESSION_ID.to_string(),
            left_root: left_root.to_string(),
            right_root: right_root.to_string(),
            compare_focus_path: CompareFocusPath::root(),
            compare_row_focus_path: None,
            expansion_overrides: BTreeMap::new(),
            horizontal_scroll_locked: true,
        }
    }
}

/// One compare-originated file session tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSession {
    /// Stable file-session tab id.
    pub session_id: String,
    /// Owning compare-session id.
    pub source_compare_session_id: String,
    /// Canonical compare relative path for this file tab.
    pub relative_path: String,
    /// Short display title used in the tab strip.
    pub display_title: String,
    /// Inner `Diff / Analysis` mode for this file session.
    pub mode: FileSessionMode,
    /// Active results membership source index when still valid.
    pub source_index: Option<usize>,
    /// Whether detailed diff loading is running.
    pub diff_loading: bool,
    /// Top-level detailed diff error.
    pub diff_error_message: Option<String>,
    /// Structured detailed diff panel payload.
    pub selected_diff: Option<DiffPanelViewModel>,
    /// Dedicated Compare File View payload for compare-originated file tabs.
    pub selected_compare_file: Option<CompareFilePanelViewModel>,
    /// Optional warning from detailed diff result.
    pub diff_warning: Option<String>,
    /// Whether selected detailed diff is truncated.
    pub diff_truncated: bool,
    /// Whether AI analysis can be triggered for current selection.
    pub analysis_available: bool,
    /// Whether AI analysis loading is running.
    pub analysis_loading: bool,
    /// Optional hint text for AI analysis availability.
    pub analysis_hint: Option<String>,
    /// Top-level AI analysis error.
    pub analysis_error_message: Option<String>,
    /// Structured AI analysis payload for panel rendering.
    pub analysis_result: Option<AnalysisResultViewModel>,
}

impl FileSession {
    pub fn new(relative_path: &str, source_compare_session_id: &str) -> Self {
        let (_, leaf_name) = split_relative_path_leaf(relative_path.trim());
        Self {
            session_id: file_session_id(relative_path),
            source_compare_session_id: source_compare_session_id.to_string(),
            relative_path: relative_path.trim().to_string(),
            display_title: if leaf_name.is_empty() {
                relative_path.trim().to_string()
            } else {
                leaf_name
            },
            mode: FileSessionMode::Diff,
            source_index: None,
            diff_loading: false,
            diff_error_message: None,
            selected_diff: None,
            selected_compare_file: None,
            diff_warning: None,
            diff_truncated: false,
            analysis_available: false,
            analysis_loading: false,
            analysis_hint: Some("Select one changed text file to analyze.".to_string()),
            analysis_error_message: None,
            analysis_result: None,
        }
    }
}

/// Result of opening or activating a file session tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSessionOpenResult {
    /// Whether the tab already existed before this request.
    pub activated_existing: bool,
    /// Whether the restored session already had cached file-view content/state.
    pub has_cached_view_state: bool,
}

/// Pending workspace session transition that requires confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSessionConfirmationAction {
    /// End the current compare session and close all compare-originated file tabs.
    CloseCompareSession,
    /// Leave compare-session mode and open the requested file through the standard File View.
    OpenStandardFileView {
        relative_path: String,
        source_index: Option<usize>,
    },
    /// Reset the current compare session to a new compare anchor and close all child file tabs.
    ResetCompareSession {
        relative_path: String,
        preferred_row_focus: Option<String>,
    },
}

/// Pending confirmation before applying one workspace session transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionConfirmation {
    /// Pending action to apply after confirmation.
    pub action: WorkspaceSessionConfirmationAction,
    /// Number of compare-originated file tabs that will be closed together.
    pub related_file_tab_count: usize,
}

/// Immediate follow-up requested after confirming one workspace session transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSessionConfirmationEffect {
    None,
    LoadSelectedDiff,
}

/// In-memory UI state for compare workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    /// Left compare root path.
    pub left_root: String,
    /// Right compare root path.
    pub right_root: String,
    /// Whether compare command is currently running.
    pub running: bool,
    /// Plain status text for rendering.
    pub status_text: String,
    /// Summary text derived from compare result.
    pub summary_text: String,
    /// Structured compare foundation for workspace-level projections.
    pub compare_foundation: CompareFoundation,
    /// Revision for flat navigator projection refreshes.
    pub navigator_flat_projection_revision: u64,
    /// Revision for tree navigator projection refreshes.
    pub navigator_tree_projection_revision: u64,
    /// Revision for one-shot flat ensure-visible requests.
    pub navigator_flat_scroll_request_revision: u64,
    /// Source index requested for flat ensure-visible.
    pub navigator_flat_scroll_target_source_index: Option<usize>,
    /// Revision for one-shot tree ensure-visible requests.
    pub navigator_tree_scroll_request_revision: u64,
    /// Source index requested for tree ensure-visible.
    pub navigator_tree_scroll_target_source_index: Option<usize>,
    /// Result rows for list rendering.
    pub entry_rows: Vec<CompareEntryRowViewModel>,
    /// Filter text applied to compare rows.
    pub entry_filter: String,
    /// Status filter scope applied to compare rows (`all`, `different`, ...).
    pub entry_status_filter: String,
    /// Non-search runtime mode for Results / Navigator.
    pub navigator_runtime_view_mode: NavigatorViewMode,
    /// Persisted default mode for non-search Results / Navigator.
    pub default_navigator_view_mode: NavigatorViewMode,
    /// Whether the top-level sidebar shell is currently visible.
    pub sidebar_visible: bool,
    /// Outer workspace mode owned in Rust state.
    pub workspace_mode: WorkspaceMode,
    /// Outer workspace session tabs.
    pub workspace_sessions: Vec<WorkspaceSession>,
    /// Currently active outer workspace session id.
    pub active_session_id: Option<String>,
    /// Optional compare-tree session currently attached to the workspace.
    pub compare_tree_session: Option<CompareTreeSession>,
    /// Compare-originated file sessions attached to the current compare session.
    pub file_sessions: Vec<FileSession>,
    /// Compare-view focus anchor independent from file selection.
    pub compare_focus_path: CompareFocusPath,
    /// Focused visible compare-tree row inside Compare View, independent from file selection.
    pub compare_row_focus_path: Option<String>,
    /// Whether Compare Tree horizontal scrolling is locked between left/right panes.
    pub compare_view_horizontal_scroll_locked: bool,
    /// Current Compare Tree quick-locate query text.
    pub compare_view_quick_locate_query: String,
    /// Revision for Compare View visible-tree projection refreshes.
    pub compare_view_projection_revision: u64,
    /// Revision for Compare View one-shot ensure-visible requests.
    pub compare_view_scroll_request_revision: u64,
    /// Relative path requested for Compare View ensure-visible.
    pub compare_view_scroll_target_relative_path: Option<String>,
    /// Whether current File View session should show compare-context header treatment.
    pub can_return_to_compare_view: bool,
    /// Current active file-view inner mode (`Diff` / `Analysis`).
    pub file_view_mode: FileSessionMode,
    /// Expansion overrides for compare-tree directory nodes.
    pub compare_view_expansion_overrides: BTreeMap<String, bool>,
    /// Expansion overrides for directory nodes in tree mode.
    pub navigator_tree_expansion_overrides: BTreeMap<String, bool>,
    /// Warning lines from compare report.
    pub warning_lines: Vec<String>,
    /// Top-level compare error message.
    pub error_message: Option<String>,
    /// Whether current report is truncated.
    pub truncated: bool,
    /// Optional selected row index.
    pub selected_row: Option<usize>,
    /// Whether detailed diff loading is running.
    pub diff_loading: bool,
    /// Top-level detailed diff error.
    pub diff_error_message: Option<String>,
    /// Relative path from current selected row.
    pub selected_relative_path: Option<String>,
    /// Structured detailed diff panel payload.
    pub selected_diff: Option<DiffPanelViewModel>,
    /// Dedicated Compare File View payload for compare-originated file tabs.
    pub selected_compare_file: Option<CompareFilePanelViewModel>,
    /// Optional warning from detailed diff result.
    pub diff_warning: Option<String>,
    /// Whether selected detailed diff is truncated.
    pub diff_truncated: bool,
    /// Whether AI analysis can be triggered for current selection.
    pub analysis_available: bool,
    /// Whether AI analysis loading is running.
    pub analysis_loading: bool,
    /// Optional hint text for AI analysis availability.
    pub analysis_hint: Option<String>,
    /// Top-level AI analysis error.
    pub analysis_error_message: Option<String>,
    /// Structured AI analysis payload for panel rendering.
    pub analysis_result: Option<AnalysisResultViewModel>,
    /// Selected AI provider mode.
    pub analysis_provider_kind: AiProviderKind,
    /// OpenAI-compatible endpoint input.
    pub analysis_openai_endpoint: String,
    /// OpenAI-compatible API key input.
    pub analysis_openai_api_key: String,
    /// OpenAI-compatible model input.
    pub analysis_openai_model: String,
    /// OpenAI-compatible request timeout input in seconds.
    pub analysis_request_timeout_secs: u64,
    /// Whether hidden files stay visible in Results / Navigator.
    pub show_hidden_files: bool,
    /// Settings dialog error message.
    pub settings_error_message: Option<String>,
    /// Pending workspace session transition confirmation.
    pub pending_workspace_session_confirmation: Option<WorkspaceSessionConfirmation>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            left_root: String::new(),
            right_root: String::new(),
            running: false,
            status_text: "Ready".to_string(),
            summary_text: String::new(),
            compare_foundation: CompareFoundation::default(),
            navigator_flat_projection_revision: 0,
            navigator_tree_projection_revision: 0,
            navigator_flat_scroll_request_revision: 0,
            navigator_flat_scroll_target_source_index: None,
            navigator_tree_scroll_request_revision: 0,
            navigator_tree_scroll_target_source_index: None,
            entry_rows: Vec::new(),
            entry_filter: String::new(),
            entry_status_filter: "all".to_string(),
            navigator_runtime_view_mode: NavigatorViewMode::Tree,
            default_navigator_view_mode: NavigatorViewMode::Tree,
            sidebar_visible: true,
            workspace_mode: WorkspaceMode::FileView,
            workspace_sessions: Vec::new(),
            active_session_id: None,
            compare_tree_session: None,
            file_sessions: Vec::new(),
            compare_focus_path: CompareFocusPath::root(),
            compare_row_focus_path: None,
            compare_view_horizontal_scroll_locked: true,
            compare_view_quick_locate_query: String::new(),
            compare_view_projection_revision: 0,
            compare_view_scroll_request_revision: 0,
            compare_view_scroll_target_relative_path: None,
            can_return_to_compare_view: false,
            file_view_mode: FileSessionMode::Diff,
            compare_view_expansion_overrides: BTreeMap::new(),
            navigator_tree_expansion_overrides: BTreeMap::new(),
            warning_lines: Vec::new(),
            error_message: None,
            truncated: false,
            selected_row: None,
            diff_loading: false,
            diff_error_message: None,
            selected_relative_path: None,
            selected_diff: None,
            selected_compare_file: None,
            diff_warning: None,
            diff_truncated: false,
            analysis_available: false,
            analysis_loading: false,
            analysis_hint: Some("Select one changed text file to analyze.".to_string()),
            analysis_error_message: None,
            analysis_result: None,
            analysis_provider_kind: AiProviderKind::Mock,
            analysis_openai_endpoint: String::new(),
            analysis_openai_api_key: String::new(),
            analysis_openai_model: "gpt-4o-mini".to_string(),
            analysis_request_timeout_secs: 30,
            show_hidden_files: true,
            settings_error_message: None,
            pending_workspace_session_confirmation: None,
        }
    }
}

impl AppState {
    fn has_selection_path(&self) -> bool {
        self.selected_relative_path
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
    }

    fn active_workspace_session_kind(&self) -> Option<WorkspaceSessionKind> {
        let active_id = self.active_session_id.as_deref()?;
        self.workspace_sessions
            .iter()
            .find(|session| session.session_id == active_id)
            .map(|session| session.kind)
    }

    fn active_file_session_index(&self) -> Option<usize> {
        let active_id = self.active_session_id.as_deref()?;
        self.file_sessions
            .iter()
            .position(|session| session.session_id == active_id)
    }

    fn active_file_session_mut(&mut self) -> Option<&mut FileSession> {
        let index = self.active_file_session_index()?;
        self.file_sessions.get_mut(index)
    }

    fn file_session_index_for_relative_path(&self, relative_path: &str) -> Option<usize> {
        let normalized = relative_path.trim();
        if normalized.is_empty() {
            return None;
        }
        self.file_sessions
            .iter()
            .position(|session| session.relative_path == normalized)
    }

    fn refresh_workspace_sessions(&mut self) {
        let mut sessions = Vec::new();
        if self.compare_tree_session.is_some() {
            sessions.push(WorkspaceSession {
                session_id: COMPARE_TREE_SESSION_ID.to_string(),
                kind: WorkspaceSessionKind::CompareTree,
                label: "Compare Tree".to_string(),
                tooltip_text: "Compare Tree".to_string(),
                closable: true,
            });
        }
        sessions.extend(self.file_sessions.iter().map(|session| WorkspaceSession {
            session_id: session.session_id.clone(),
            kind: WorkspaceSessionKind::File,
            label: session.display_title.clone(),
            tooltip_text: session.relative_path.clone(),
            closable: true,
        }));
        self.workspace_sessions = sessions;

        let active_still_exists = self.active_session_id.as_deref().is_some_and(|session_id| {
            self.workspace_sessions
                .iter()
                .any(|session| session.session_id == session_id)
        });
        if !active_still_exists {
            self.active_session_id = self
                .workspace_sessions
                .first()
                .map(|session| session.session_id.clone());
        }
        self.sync_workspace_mode_from_active_session();
    }

    fn sync_workspace_mode_from_active_session(&mut self) {
        self.workspace_mode = match self.active_workspace_session_kind() {
            Some(WorkspaceSessionKind::CompareTree) => WorkspaceMode::CompareView,
            Some(WorkspaceSessionKind::File) | None => WorkspaceMode::FileView,
        };
        self.can_return_to_compare_view = matches!(
            self.active_workspace_session_kind(),
            Some(WorkspaceSessionKind::File)
        ) && self.compare_tree_session.is_some();
    }

    fn sync_compare_tree_session_from_top_level(&mut self) {
        if let Some(session) = self.compare_tree_session.as_mut() {
            session.left_root = self.left_root.clone();
            session.right_root = self.right_root.clone();
            session.compare_focus_path = self.compare_focus_path.clone();
            session.compare_row_focus_path = self.compare_row_focus_path.clone();
            session.expansion_overrides = self.compare_view_expansion_overrides.clone();
            session.horizontal_scroll_locked = self.compare_view_horizontal_scroll_locked;
        }
    }

    fn restore_compare_tree_session_to_top_level(&mut self) {
        let Some(session) = self.compare_tree_session.as_ref() else {
            return;
        };
        self.compare_focus_path = session.compare_focus_path.clone();
        self.compare_row_focus_path = session.compare_row_focus_path.clone();
        self.compare_view_expansion_overrides = session.expansion_overrides.clone();
        self.compare_view_horizontal_scroll_locked = session.horizontal_scroll_locked;
    }

    pub fn sync_active_file_session_from_top_level(&mut self) {
        let selected_relative_path = self.selected_relative_path.clone();
        let selected_row = self.selected_row;
        let file_view_mode = self.file_view_mode;
        let diff_loading = self.diff_loading;
        let diff_error_message = self.diff_error_message.clone();
        let selected_diff = self.selected_diff.clone();
        let selected_compare_file = self.selected_compare_file.clone();
        let diff_warning = self.diff_warning.clone();
        let diff_truncated = self.diff_truncated;
        let analysis_available = self.analysis_available;
        let analysis_loading = self.analysis_loading;
        let analysis_hint = self.analysis_hint.clone();
        let analysis_error_message = self.analysis_error_message.clone();
        let analysis_result = self.analysis_result.clone();
        let computed_title = selected_relative_path
            .as_deref()
            .and_then(normalize_optional_text)
            .map(|value| split_relative_path_leaf(&value).1)
            .filter(|value| !value.is_empty());

        if let Some(session) = self.active_file_session_mut() {
            if let Some(relative_path) = selected_relative_path
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                session.relative_path = relative_path.to_string();
            }
            session.display_title = computed_title.unwrap_or_else(|| session.display_title.clone());
            session.source_index = selected_row;
            session.mode = file_view_mode;
            session.diff_loading = diff_loading;
            session.diff_error_message = diff_error_message;
            session.selected_diff = selected_diff;
            session.selected_compare_file = selected_compare_file;
            session.diff_warning = diff_warning;
            session.diff_truncated = diff_truncated;
            session.analysis_available = analysis_available;
            session.analysis_loading = analysis_loading;
            session.analysis_hint = analysis_hint;
            session.analysis_error_message = analysis_error_message;
            session.analysis_result = analysis_result;
        }
        self.refresh_workspace_sessions();
    }

    fn restore_file_session_to_top_level(&mut self, session_index: usize) {
        let Some(session) = self.file_sessions.get(session_index).cloned() else {
            return;
        };
        self.selected_row = session.source_index;
        self.selected_relative_path = Some(session.relative_path.clone());
        self.file_view_mode = session.mode;
        self.diff_loading = session.diff_loading;
        self.diff_error_message = session.diff_error_message;
        self.selected_diff = session.selected_diff;
        self.selected_compare_file = session.selected_compare_file;
        self.diff_warning = session.diff_warning;
        self.diff_truncated = session.diff_truncated;
        self.analysis_available = session.analysis_available;
        self.analysis_loading = session.analysis_loading;
        self.analysis_hint = session.analysis_hint;
        self.analysis_error_message = session.analysis_error_message;
        self.analysis_result = session.analysis_result;
        self.workspace_mode = WorkspaceMode::FileView;
        self.can_return_to_compare_view = self.compare_tree_session.is_some();
    }

    pub fn has_compare_tree_session(&self) -> bool {
        self.compare_tree_session.is_some()
    }

    pub fn active_file_session_uses_compare_file_view(&self) -> bool {
        self.compare_tree_session.is_some() && self.active_file_session_index().is_some()
    }

    pub fn active_workspace_session_index(&self) -> i32 {
        self.active_session_id
            .as_deref()
            .and_then(|session_id| {
                self.workspace_sessions
                    .iter()
                    .position(|session| session.session_id == session_id)
            })
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    }

    pub fn workspace_session_ids(&self) -> Vec<String> {
        self.workspace_sessions
            .iter()
            .map(|session| session.session_id.clone())
            .collect()
    }

    pub fn workspace_session_labels(&self) -> Vec<String> {
        self.workspace_sessions
            .iter()
            .map(|session| session.label.clone())
            .collect()
    }

    pub fn workspace_session_tooltips(&self) -> Vec<String> {
        self.workspace_sessions
            .iter()
            .map(|session| session.tooltip_text.clone())
            .collect()
    }

    pub fn workspace_session_kinds(&self) -> Vec<String> {
        self.workspace_sessions
            .iter()
            .map(|session| session.kind.as_str().to_string())
            .collect()
    }

    pub fn workspace_session_closable(&self) -> Vec<bool> {
        self.workspace_sessions
            .iter()
            .map(|session| session.closable)
            .collect()
    }

    pub fn workspace_sessions_visible(&self) -> bool {
        !self.workspace_sessions.is_empty()
    }

    pub fn file_view_mode_tab_index(&self) -> i32 {
        self.file_view_mode.tab_index()
    }

    pub fn set_file_view_mode(&mut self, mode: FileSessionMode) -> bool {
        if self.file_view_mode == mode {
            return false;
        }
        self.file_view_mode = mode;
        self.sync_active_file_session_from_top_level();
        true
    }

    pub fn compare_tree_file_tab_count(&self) -> usize {
        let Some(compare_session) = self.compare_tree_session.as_ref() else {
            return 0;
        };
        self.file_sessions
            .iter()
            .filter(|session| session.source_compare_session_id == compare_session.session_id)
            .count()
    }

    pub fn ensure_compare_tree_session(&mut self) {
        if self.compare_tree_session.is_none() {
            self.compare_tree_session = Some(CompareTreeSession::new(
                self.left_root.as_str(),
                self.right_root.as_str(),
            ));
        }
        self.sync_compare_tree_session_from_top_level();
        self.refresh_workspace_sessions();
    }

    pub fn activate_workspace_session(&mut self, session_id: &str) -> bool {
        let normalized = session_id.trim();
        if normalized.is_empty()
            || self.active_session_id.as_deref() == Some(normalized)
            || !self
                .workspace_sessions
                .iter()
                .any(|session| session.session_id == normalized)
        {
            return false;
        }

        self.sync_active_file_session_from_top_level();
        self.active_session_id = Some(normalized.to_string());
        match self.active_workspace_session_kind() {
            Some(WorkspaceSessionKind::CompareTree) => {
                self.restore_compare_tree_session_to_top_level();
                self.workspace_mode = WorkspaceMode::CompareView;
                self.can_return_to_compare_view = false;
            }
            Some(WorkspaceSessionKind::File) => {
                if let Some(index) = self.active_file_session_index() {
                    self.restore_file_session_to_top_level(index);
                }
            }
            None => {
                self.workspace_mode = WorkspaceMode::FileView;
                self.can_return_to_compare_view = false;
            }
        }
        self.refresh_workspace_sessions();
        true
    }

    pub fn open_or_activate_file_session(
        &mut self,
        relative_path: &str,
    ) -> Option<FileSessionOpenResult> {
        let normalized = relative_path.trim();
        let Some(compare_session_id) = self
            .compare_tree_session
            .as_ref()
            .map(|session| session.session_id.clone())
        else {
            return None;
        };
        if normalized.is_empty() {
            return None;
        }

        self.sync_active_file_session_from_top_level();
        let session_id = file_session_id(normalized);
        if let Some(index) = self.file_session_index_for_relative_path(normalized) {
            let has_cached_view_state = self.file_sessions.get(index).is_some_and(|session| {
                session.selected_diff.is_some()
                    || session.selected_compare_file.is_some()
                    || session.diff_error_message.is_some()
                    || session.analysis_result.is_some()
                    || session.analysis_error_message.is_some()
            });
            self.active_session_id = Some(session_id);
            self.restore_file_session_to_top_level(index);
            self.refresh_workspace_sessions();
            return Some(FileSessionOpenResult {
                activated_existing: true,
                has_cached_view_state,
            });
        }

        let mut session = FileSession::new(normalized, compare_session_id.as_str());
        session.source_index = self
            .row_index_for_relative_path(normalized)
            .filter(|index| self.is_row_member_in_active_results(*index));
        self.file_sessions.push(session);
        let new_index = self.file_sessions.len().saturating_sub(1);
        self.active_session_id = Some(session_id);
        self.restore_file_session_to_top_level(new_index);
        self.refresh_workspace_sessions();
        Some(FileSessionOpenResult {
            activated_existing: false,
            has_cached_view_state: false,
        })
    }

    pub fn close_workspace_session(&mut self, session_id: &str) -> bool {
        let normalized = session_id.trim();
        if normalized.is_empty() {
            return false;
        }
        if normalized == COMPARE_TREE_SESSION_ID {
            return self.request_compare_tree_session_close();
        }

        let Some(index) = self
            .file_sessions
            .iter()
            .position(|session| session.session_id == normalized)
        else {
            return false;
        };

        self.sync_active_file_session_from_top_level();
        let was_active = self.active_session_id.as_deref() == Some(normalized);
        self.file_sessions.remove(index);
        if was_active {
            self.active_session_id =
                if let Some(compare_session) = self.compare_tree_session.as_ref() {
                    Some(compare_session.session_id.clone())
                } else {
                    self.file_sessions
                        .last()
                        .map(|session| session.session_id.clone())
                };
            if let Some(next_index) = self.active_file_session_index() {
                self.restore_file_session_to_top_level(next_index);
            } else {
                self.workspace_mode = if self.compare_tree_session.is_some() {
                    WorkspaceMode::CompareView
                } else {
                    WorkspaceMode::FileView
                };
                self.can_return_to_compare_view = false;
                self.file_view_mode = FileSessionMode::Diff;
            }
        }
        self.refresh_workspace_sessions();
        true
    }

    fn clear_top_level_file_view_state(&mut self) {
        self.file_view_mode = FileSessionMode::Diff;
        self.selected_row = None;
        self.selected_relative_path = None;
        self.clear_diff_panel();
        self.clear_compare_file_panel();
        self.analysis_available = false;
        self.clear_analysis_panel();
        self.analysis_hint = Some("Select one changed text file to analyze.".to_string());
    }

    fn clear_compare_session_shell(&mut self) {
        self.compare_tree_session = None;
        self.file_sessions.clear();
        self.active_session_id = None;
        self.workspace_mode = WorkspaceMode::FileView;
        self.can_return_to_compare_view = false;
        self.compare_focus_path = CompareFocusPath::root();
        self.compare_row_focus_path = None;
        self.compare_view_expansion_overrides.clear();
        self.compare_view_horizontal_scroll_locked = true;
        self.compare_view_quick_locate_query.clear();
    }

    fn apply_close_compare_session(&mut self) {
        self.pending_workspace_session_confirmation = None;
        self.clear_compare_session_shell();
        self.clear_top_level_file_view_state();
        self.refresh_workspace_sessions();
    }

    fn apply_standard_file_view_selection(
        &mut self,
        relative_path: &str,
        source_index: Option<usize>,
    ) -> WorkspaceSessionConfirmationEffect {
        let normalized = relative_path.trim();
        if normalized.is_empty() {
            return WorkspaceSessionConfirmationEffect::None;
        }

        let selected_index = source_index
            .filter(|index| *index < self.entry_rows.len())
            .filter(|index| self.entry_rows[*index].relative_path == normalized)
            .filter(|index| self.entry_rows[*index].entry_kind == "file")
            .filter(|index| self.is_row_member_in_active_results(*index))
            .or_else(|| {
                self.row_index_for_relative_path(normalized)
                    .filter(|index| self.is_row_member_in_active_results(*index))
                    .filter(|index| self.entry_rows[*index].entry_kind == "file")
            });

        self.workspace_mode = WorkspaceMode::FileView;
        self.can_return_to_compare_view = false;
        self.file_view_mode = FileSessionMode::Diff;
        self.clear_diff_panel();
        self.analysis_available = false;
        self.clear_analysis_panel();

        match selected_index
            .and_then(|index| self.entry_rows.get(index).cloned().map(|row| (index, row)))
        {
            Some((index, row)) => {
                self.selected_row = Some(index);
                self.selected_relative_path = Some(row.relative_path.clone());
                self.analysis_hint = Some(if !row.can_load_analysis {
                    row.analysis_blocked_reason
                        .unwrap_or_else(|| "selected row does not support AI analysis".to_string())
                } else {
                    "Load detailed diff, then click Analyze.".to_string()
                });
                WorkspaceSessionConfirmationEffect::LoadSelectedDiff
            }
            None => {
                self.selected_row = None;
                self.selected_relative_path = Some(normalized.to_string());
                self.analysis_hint = Some(
                    "Previous selection is no longer active in the current Results / Navigator set."
                        .to_string(),
                );
                WorkspaceSessionConfirmationEffect::None
            }
        }
    }

    fn apply_compare_session_reset(
        &mut self,
        relative_path: &str,
        preferred_row_focus: Option<&str>,
    ) -> bool {
        let normalized = relative_path.trim();
        if self.compare_tree_session.is_none() {
            self.compare_tree_session = Some(CompareTreeSession::new(
                self.left_root.as_str(),
                self.right_root.as_str(),
            ));
        }
        self.pending_workspace_session_confirmation = None;
        self.file_sessions.clear();
        self.active_session_id = Some(COMPARE_TREE_SESSION_ID.to_string());
        self.workspace_mode = WorkspaceMode::CompareView;
        self.can_return_to_compare_view = false;
        self.compare_view_expansion_overrides.clear();

        self.set_compare_focus_path(CompareFocusPath::relative(normalized));
        if let Some(preferred) = preferred_row_focus {
            self.reveal_compare_view_path(preferred);
            self.set_compare_row_focus_path(Some(preferred));
        }
        if let Some(path) = self.compare_row_focus_path.clone() {
            self.reveal_compare_view_path(path.as_str());
            self.set_compare_row_focus_path(Some(path.as_str()));
            self.request_compare_view_scroll_to_path(path.as_str());
        }
        self.refresh_workspace_sessions();
        true
    }

    pub fn request_compare_tree_session_close(&mut self) -> bool {
        if self.compare_tree_session.is_none() {
            return false;
        }

        let related_file_tab_count = self.compare_tree_file_tab_count();
        if related_file_tab_count > 0 {
            self.pending_workspace_session_confirmation = Some(WorkspaceSessionConfirmation {
                action: WorkspaceSessionConfirmationAction::CloseCompareSession,
                related_file_tab_count,
            });
            return true;
        }
        self.apply_close_compare_session();
        true
    }

    pub fn request_standard_file_view_after_compare_session_close(
        &mut self,
        relative_path: &str,
        source_index: Option<usize>,
    ) -> bool {
        let normalized = relative_path.trim();
        if normalized.is_empty() || self.compare_tree_session.is_none() {
            return false;
        }
        self.pending_workspace_session_confirmation = Some(WorkspaceSessionConfirmation {
            action: WorkspaceSessionConfirmationAction::OpenStandardFileView {
                relative_path: normalized.to_string(),
                source_index,
            },
            related_file_tab_count: self.compare_tree_file_tab_count(),
        });
        true
    }

    pub fn request_compare_session_reset(
        &mut self,
        relative_path: &str,
        preferred_row_focus: Option<&str>,
    ) -> bool {
        let normalized = relative_path.trim();
        let related_file_tab_count = self.compare_tree_file_tab_count();
        if related_file_tab_count > 0 {
            self.pending_workspace_session_confirmation = Some(WorkspaceSessionConfirmation {
                action: WorkspaceSessionConfirmationAction::ResetCompareSession {
                    relative_path: normalized.to_string(),
                    preferred_row_focus: preferred_row_focus.map(ToString::to_string),
                },
                related_file_tab_count,
            });
            return true;
        }
        self.apply_compare_session_reset(normalized, preferred_row_focus)
    }

    pub fn confirm_workspace_session_action(&mut self) -> WorkspaceSessionConfirmationEffect {
        let Some(pending) = self.pending_workspace_session_confirmation.clone() else {
            return WorkspaceSessionConfirmationEffect::None;
        };

        match pending.action {
            WorkspaceSessionConfirmationAction::CloseCompareSession => {
                self.apply_close_compare_session();
                WorkspaceSessionConfirmationEffect::None
            }
            WorkspaceSessionConfirmationAction::OpenStandardFileView {
                relative_path,
                source_index,
            } => {
                self.apply_close_compare_session();
                self.apply_standard_file_view_selection(relative_path.as_str(), source_index)
            }
            WorkspaceSessionConfirmationAction::ResetCompareSession {
                relative_path,
                preferred_row_focus,
            } => {
                self.apply_compare_session_reset(
                    relative_path.as_str(),
                    preferred_row_focus.as_deref(),
                );
                WorkspaceSessionConfirmationEffect::None
            }
        }
    }

    pub fn cancel_workspace_session_action(&mut self) -> bool {
        if self.pending_workspace_session_confirmation.is_none() {
            return false;
        }
        self.pending_workspace_session_confirmation = None;
        true
    }

    pub fn workspace_session_confirmation_open(&self) -> bool {
        self.pending_workspace_session_confirmation.is_some()
    }

    pub fn workspace_session_confirmation_title_text(&self) -> String {
        match self
            .pending_workspace_session_confirmation
            .as_ref()
            .map(|pending| &pending.action)
        {
            Some(WorkspaceSessionConfirmationAction::CloseCompareSession) => {
                "Close Compare session?".to_string()
            }
            Some(WorkspaceSessionConfirmationAction::OpenStandardFileView { .. }) => {
                "Open standard File View and close current Compare session?".to_string()
            }
            Some(WorkspaceSessionConfirmationAction::ResetCompareSession { .. }) => {
                "Reset Compare session?".to_string()
            }
            None => String::new(),
        }
    }

    pub fn workspace_session_confirmation_body_text(&self) -> String {
        let Some(pending) = self.pending_workspace_session_confirmation.as_ref() else {
            return String::new();
        };
        match &pending.action {
            WorkspaceSessionConfirmationAction::CloseCompareSession => {
                if pending.related_file_tab_count == 1 {
                    "Closing Compare Tree will also close 1 related file tab.".to_string()
                } else {
                    format!(
                        "Closing Compare Tree will also close {} related file tabs.",
                        pending.related_file_tab_count
                    )
                }
            }
            WorkspaceSessionConfirmationAction::OpenStandardFileView { .. } => {
                if pending.related_file_tab_count == 0 {
                    "The current Compare Tree tab will be closed before opening the standard File View."
                        .to_string()
                } else if pending.related_file_tab_count == 1 {
                    "The current Compare Tree tab and 1 related file tab will be closed before opening the standard File View."
                        .to_string()
                } else {
                    format!(
                        "The current Compare Tree tab and {} related file tabs will be closed before opening the standard File View.",
                        pending.related_file_tab_count
                    )
                }
            }
            WorkspaceSessionConfirmationAction::ResetCompareSession { .. } => {
                if pending.related_file_tab_count == 1 {
                    "This will close 1 related file tab and retarget the current Compare Tree tab."
                        .to_string()
                } else {
                    format!(
                        "This will close {} related file tabs and retarget the current Compare Tree tab.",
                        pending.related_file_tab_count
                    )
                }
            }
        }
    }

    pub fn workspace_session_confirmation_action_label_text(&self) -> String {
        match self
            .pending_workspace_session_confirmation
            .as_ref()
            .map(|pending| &pending.action)
        {
            Some(WorkspaceSessionConfirmationAction::CloseCompareSession) => {
                "Close Session".to_string()
            }
            Some(WorkspaceSessionConfirmationAction::OpenStandardFileView { .. }) => {
                "Open File View".to_string()
            }
            Some(WorkspaceSessionConfirmationAction::ResetCompareSession { .. }) => {
                "Reset Session".to_string()
            }
            None => String::new(),
        }
    }

    pub fn reconcile_file_sessions_with_active_results(&mut self) {
        let session_paths = self
            .file_sessions
            .iter()
            .map(|session| session.relative_path.clone())
            .collect::<Vec<_>>();
        let resolved_indices = session_paths
            .iter()
            .map(|relative_path| {
                self.row_index_for_relative_path(relative_path)
                    .filter(|index| self.is_row_member_in_active_results(*index))
            })
            .collect::<Vec<_>>();

        for (session, source_index) in self.file_sessions.iter_mut().zip(resolved_indices) {
            if session.source_index == source_index {
                continue;
            }
            session.source_index = source_index;
            if session.source_index.is_none() {
                session.diff_loading = false;
                session.diff_error_message = None;
                session.selected_diff = None;
                session.selected_compare_file = None;
                session.diff_warning = None;
                session.diff_truncated = false;
                session.analysis_available = false;
                session.analysis_loading = false;
                session.analysis_hint = Some(
                    "Previous selection is no longer active in the current Results / Navigator set."
                        .to_string(),
                );
                session.analysis_error_message = None;
                session.analysis_result = None;
            }
        }

        if let Some(index) = self.active_file_session_index() {
            self.restore_file_session_to_top_level(index);
        }
        self.refresh_workspace_sessions();
    }

    pub fn mark_file_sessions_for_compare_restore(&mut self) {
        let session_paths = self
            .file_sessions
            .iter()
            .map(|session| session.relative_path.clone())
            .collect::<Vec<_>>();
        let resolved_indices = session_paths
            .iter()
            .map(|relative_path| {
                self.row_index_for_relative_path(relative_path)
                    .filter(|index| self.is_row_member_in_active_results(*index))
            })
            .collect::<Vec<_>>();

        for (session, source_index) in self.file_sessions.iter_mut().zip(resolved_indices) {
            session.source_index = source_index;
            session.diff_loading = false;
            session.diff_error_message = None;
            session.selected_diff = None;
            session.selected_compare_file = None;
            session.diff_warning = None;
            session.diff_truncated = false;
            session.analysis_available = false;
            session.analysis_loading = false;
            session.analysis_error_message = None;
            session.analysis_result = None;
            session.analysis_hint = Some(if session.source_index.is_some() {
                "Previous selection will be rechecked after compare finishes.".to_string()
            } else {
                "Previous selection is no longer active in the current Results / Navigator set."
                    .to_string()
            });
        }

        if let Some(index) = self.active_file_session_index() {
            self.restore_file_session_to_top_level(index);
        }
        self.refresh_workspace_sessions();
    }

    pub fn active_file_session_needs_diff_reload(&self) -> bool {
        matches!(
            self.active_workspace_session_kind(),
            Some(WorkspaceSessionKind::File)
        ) && self.selected_row.is_some()
            && if self.active_file_session_uses_compare_file_view() {
                self.selected_compare_file.is_none()
            } else {
                self.selected_diff.is_none()
            }
            && !self.diff_loading
            && self.diff_error_message.is_none()
    }

    fn has_stale_selection(&self) -> bool {
        self.selected_row.is_none() && self.has_selection_path()
    }

    fn selected_entry_row(&self) -> Option<&CompareEntryRowViewModel> {
        self.selected_row.and_then(|idx| self.entry_rows.get(idx))
    }

    fn selected_row_status_token(&self) -> &str {
        self.selected_entry_row()
            .map(|row| row.status.as_str())
            .unwrap_or("")
    }

    fn selected_file_type_hint(&self) -> Option<String> {
        let entry = self.selected_entry_row()?;
        if entry.entry_kind != "file" {
            return Some(format!("entry {}", entry.entry_kind));
        }

        let relative_path = self
            .selected_relative_path
            .as_deref()
            .unwrap_or_default()
            .trim();
        if relative_path.is_empty() {
            return Some("type file".to_string());
        }
        let ext = Path::new(relative_path)
            .extension()
            .and_then(|value| value.to_str());
        match ext.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => Some(format!("type .{}", value.to_ascii_lowercase())),
            None => Some("type file".to_string()),
        }
    }

    fn selected_row_can_load_analysis(&self) -> bool {
        self.selected_entry_row()
            .map(|row| row.can_load_analysis)
            .unwrap_or(false)
    }

    fn analysis_waiting_for_diff_context(&self) -> bool {
        if self.selected_row.is_none() || !self.selected_row_can_load_analysis() {
            return false;
        }
        if self
            .diff_error_message
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return false;
        }

        self.diff_loading || self.selected_diff.is_none()
    }

    fn analysis_selection_unavailable(&self) -> bool {
        self.selected_row.is_some()
            && !self.analysis_loading
            && self.analysis_result.is_none()
            && !self.analysis_waiting_for_diff_context()
            && !self.analysis_can_start_now()
    }

    fn diff_payload_unavailable_message(&self) -> Option<String> {
        let diff = self.selected_diff.as_ref()?;
        let line_message = diff
            .hunks
            .first()
            .and_then(|hunk| hunk.lines.first())
            .map(|line| line.content.trim().to_string())
            .filter(|content| content.starts_with("[preview unavailable]"));
        if line_message.is_some() {
            return line_message;
        }
        let summary = diff.summary_text.trim();
        if summary.to_ascii_lowercase().contains("unavailable") {
            return Some(summary.to_string());
        }
        None
    }

    fn diff_status_technical_reason(&self) -> Option<String> {
        let warning = self
            .diff_warning
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        if warning.is_some() {
            return warning;
        }

        let payload_message = self.diff_payload_unavailable_message();
        if payload_message.is_some() {
            return payload_message;
        }

        self.diff_error_message
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    }

    /// Returns true when selected row is expected to be rendered in preview mode.
    pub fn diff_is_preview_mode(&self) -> bool {
        matches!(
            self.selected_row_status_token(),
            "left-only" | "right-only" | "equal"
        )
    }

    /// Returns file context mode label for Diff header.
    pub fn diff_mode_label(&self) -> String {
        if self.diff_is_preview_mode() {
            "Preview".to_string()
        } else {
            "Detailed Diff".to_string()
        }
    }

    /// Returns style tone for diff mode pill.
    pub fn diff_mode_tone(&self) -> String {
        if self.diff_is_preview_mode() {
            "info".to_string()
        } else {
            "neutral".to_string()
        }
    }

    /// Returns result status label for selected row.
    pub fn diff_result_status_label(&self) -> String {
        match self.selected_row_status_token() {
            "different" => "Changed".to_string(),
            "left-only" => "Left Only".to_string(),
            "right-only" => "Right Only".to_string(),
            "equal" => "Equal".to_string(),
            "pending" => "Pending".to_string(),
            "skipped" => "Unavailable".to_string(),
            _ => "Unavailable".to_string(),
        }
    }

    /// Returns style tone for selected row status.
    pub fn diff_result_status_tone(&self) -> String {
        match self.selected_row_status_token() {
            "different" => "different".to_string(),
            "left-only" => "left".to_string(),
            "right-only" => "right".to_string(),
            "equal" => "equal".to_string(),
            "pending" => "info".to_string(),
            "skipped" => "warn".to_string(),
            _ => "neutral".to_string(),
        }
    }

    /// Returns normalized Diff shell state for state panel and top header.
    pub fn diff_shell_state(&self) -> DiffShellState {
        if self.selected_row.is_none() {
            return if self.has_stale_selection() {
                DiffShellState::StaleSelection
            } else {
                DiffShellState::NoSelection
            };
        }
        if self.diff_loading {
            return DiffShellState::Loading;
        }
        if self
            .diff_error_message
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return DiffShellState::Error;
        }
        if self.selected_diff.is_none() {
            return DiffShellState::Unavailable;
        }
        if self.diff_payload_unavailable_message().is_some() {
            return DiffShellState::Unavailable;
        }
        if self.diff_is_preview_mode() {
            return DiffShellState::PreviewReady;
        }
        DiffShellState::DetailedReady
    }

    /// Returns short state badge text for Diff shell.
    pub fn diff_shell_state_label(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "No Selection".to_string(),
            DiffShellState::StaleSelection => "Stale".to_string(),
            DiffShellState::Loading => "Loading".to_string(),
            DiffShellState::DetailedReady => "Detailed Ready".to_string(),
            DiffShellState::PreviewReady => "Preview Ready".to_string(),
            DiffShellState::Unavailable => "Unavailable".to_string(),
            DiffShellState::Error => "Load Failed".to_string(),
        }
    }

    /// Returns stable token for diff shell state branching in UI layer.
    pub fn diff_shell_state_token(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "no-selection".to_string(),
            DiffShellState::StaleSelection => "stale-selection".to_string(),
            DiffShellState::Loading => "loading".to_string(),
            DiffShellState::DetailedReady => "detailed-ready".to_string(),
            DiffShellState::PreviewReady => "preview-ready".to_string(),
            DiffShellState::Unavailable => "unavailable".to_string(),
            DiffShellState::Error => "error".to_string(),
        }
    }

    /// Returns state tone for Diff shell badge.
    pub fn diff_shell_state_tone(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "neutral".to_string(),
            DiffShellState::StaleSelection => "warn".to_string(),
            DiffShellState::Loading => "info".to_string(),
            DiffShellState::DetailedReady => "success".to_string(),
            DiffShellState::PreviewReady => "info".to_string(),
            DiffShellState::Unavailable => "warn".to_string(),
            DiffShellState::Error => "error".to_string(),
        }
    }

    /// Returns compact second-layer summary text for Diff header.
    pub fn diff_context_summary_text(&self) -> String {
        let diff_summary = self
            .selected_diff
            .as_ref()
            .map(|diff| diff.summary_text.trim())
            .filter(|text| !text.is_empty());
        if let Some(summary) = diff_summary {
            return abbreviate_middle(summary, 96, 64, 24);
        }

        match self.diff_shell_state() {
            DiffShellState::NoSelection => "Choose a row from Results / Navigator.".to_string(),
            DiffShellState::StaleSelection => "Previous selection is no longer active.".to_string(),
            DiffShellState::Loading => {
                if self.diff_is_preview_mode() {
                    "Preparing preview lines...".to_string()
                } else {
                    "Preparing detailed diff...".to_string()
                }
            }
            DiffShellState::DetailedReady => "Detailed diff ready.".to_string(),
            DiffShellState::PreviewReady => "Preview ready.".to_string(),
            DiffShellState::Unavailable => {
                if self.diff_is_preview_mode() {
                    "Preview unavailable.".to_string()
                } else {
                    "Detailed diff unavailable.".to_string()
                }
            }
            DiffShellState::Error => "Load failed.".to_string(),
        }
    }

    /// Returns third-layer weak context text for Diff header.
    pub fn diff_context_hint_text(&self) -> String {
        if self.selected_row.is_none() {
            return String::new();
        }

        let mut parts = Vec::new();
        if let Some(type_hint) = self.selected_file_type_hint() {
            parts.push(type_hint);
        }

        if self.diff_is_preview_mode() {
            let preview_reason = match self.selected_row_status_token() {
                "left-only" => "source left-only",
                "right-only" => "source right-only",
                "equal" => "source equal",
                _ => "source selected",
            };
            parts.push(preview_reason.to_string());
        }

        if self.diff_truncated {
            parts.push("truncated".to_string());
        }

        parts.join(" · ")
    }

    /// Returns title text for the unified Diff shell.
    pub fn diff_shell_title_text(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => "No file selected".to_string(),
            DiffShellState::StaleSelection => "Selection no longer active".to_string(),
            DiffShellState::Loading => {
                if self.diff_is_preview_mode() {
                    "Loading preview".to_string()
                } else {
                    "Loading detailed diff".to_string()
                }
            }
            DiffShellState::DetailedReady => {
                if self.diff_has_rows() {
                    "Detailed diff ready".to_string()
                } else {
                    "Detailed diff has no lines".to_string()
                }
            }
            DiffShellState::PreviewReady => {
                if self.diff_has_rows() {
                    "Preview ready".to_string()
                } else {
                    "Preview has no lines".to_string()
                }
            }
            DiffShellState::Unavailable => {
                if self.diff_is_preview_mode() {
                    "Preview unavailable".to_string()
                } else {
                    "Detailed diff unavailable".to_string()
                }
            }
            DiffShellState::Error => {
                if self.diff_is_preview_mode() {
                    "Failed to load preview".to_string()
                } else {
                    "Failed to load detailed diff".to_string()
                }
            }
        }
    }

    /// Returns primary body text for the unified Diff shell.
    pub fn diff_shell_body_text(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => {
                "Choose one row from Results / Navigator to open the file-level Diff view."
                    .to_string()
            }
            DiffShellState::StaleSelection => {
                "The previously opened file is not part of the current visible Results / Navigator set."
                    .to_string()
            }
            DiffShellState::Loading => {
                if self.diff_is_preview_mode() {
                    "Preparing selectable preview lines for the selected file.".to_string()
                } else {
                    "Preparing hunks, line numbers, and selectable diff lines.".to_string()
                }
            }
            DiffShellState::DetailedReady => {
                if self.diff_has_rows() {
                    "Detailed diff content is ready.".to_string()
                } else {
                    "This diff has no line-level content to render.".to_string()
                }
            }
            DiffShellState::PreviewReady => {
                if self.diff_has_rows() {
                    "Preview content is ready.".to_string()
                } else {
                    "This file has no text lines to display in preview mode.".to_string()
                }
            }
            DiffShellState::Unavailable => {
                if self.diff_is_preview_mode() {
                    "This selection is valid, but the current viewer has no reviewable preview text."
                        .to_string()
                } else {
                    "This selection was compared successfully, but the current viewer cannot render a detailed text diff."
                        .to_string()
                }
            }
            DiffShellState::Error => {
                if self.diff_is_preview_mode() {
                    "The selected preview could not be loaded in this session.".to_string()
                } else {
                    "The selected detailed diff could not be loaded in this session.".to_string()
                }
            }
        }
    }

    /// Returns optional secondary note text for the unified Diff shell.
    pub fn diff_shell_note_text(&self) -> String {
        match self.diff_shell_state() {
            DiffShellState::NoSelection => {
                "Changed files open Detailed Diff. Left Only / Right Only / Equal entries open Preview."
                    .to_string()
            }
            DiffShellState::StaleSelection => {
                if self.running {
                    "Compare is still running. The previous path will be rechecked when results arrive."
                        .to_string()
                } else {
                    "Adjust Search/Status filters or select a visible row to continue.".to_string()
                }
            }
            DiffShellState::Loading => self.diff_context_hint_text(),
            DiffShellState::Unavailable | DiffShellState::Error => self
                .diff_status_technical_reason()
                .map(|reason| abbreviate_middle(reason.trim(), 220, 160, 52))
                .unwrap_or_else(|| self.diff_context_hint_text()),
            DiffShellState::DetailedReady | DiffShellState::PreviewReady => String::new(),
        }
    }

    /// Returns left column label for diff table.
    pub fn diff_left_column_label(&self) -> String {
        match self.selected_row_status_token() {
            "right-only" => "-".to_string(),
            "left-only" | "equal" => "left".to_string(),
            _ => "old".to_string(),
        }
    }

    /// Returns right column label for diff table.
    pub fn diff_right_column_label(&self) -> String {
        match self.selected_row_status_token() {
            "left-only" => "-".to_string(),
            "right-only" | "equal" => "right".to_string(),
            _ => "new".to_string(),
        }
    }

    /// Returns approximate character capacity used to size the scrollable diff table.
    pub fn diff_content_char_capacity(&self) -> i32 {
        let max_chars = self
            .diff_viewer_rows()
            .iter()
            .map(|row| row.content.chars().count())
            .max()
            .unwrap_or(80);
        max_chars.clamp(80, 480) as i32
    }

    /// Returns warning lines rendered as a multiline string.
    pub fn warnings_text(&self) -> String {
        if self.warning_lines.is_empty() {
            return String::new();
        }
        let mut out = Vec::new();
        for warning in &self.warning_lines {
            for (idx, part) in wrap_ui_text(warning, WARNING_WRAP_COLUMNS)
                .iter()
                .enumerate()
            {
                if idx == 0 {
                    out.push(format!("• {part}"));
                } else {
                    out.push(format!("  {part}"));
                }
            }
        }
        out.join("\n")
    }

    /// Returns filtered entry rows with their source index.
    #[allow(dead_code)]
    pub fn filtered_entry_rows_with_index(&self) -> Vec<(usize, CompareEntryRowViewModel)> {
        self.visible_source_indices()
            .into_iter()
            .filter_map(|source_index| {
                self.entry_rows
                    .get(source_index)
                    .cloned()
                    .map(|row| (source_index, row))
            })
            .collect()
    }

    /// Returns filtered navigator rows with UI-focused presentation fields.
    pub fn navigator_row_projections(&self) -> Vec<NavigatorRowProjection> {
        let foundation = self.compare_foundation_for_projections();
        let needle = normalize_filter_needle(&self.entry_filter);
        foundation
            .source_nodes()
            .filter(|node| self.source_node_visible_in_results(node))
            .filter_map(|node| {
                let source_index = node.source_index?;
                let row = self.entry_rows.get(source_index)?.clone();
                let (parent_path_raw, display_name) = split_relative_path_leaf(&node.relative_path);
                let full_parent_path = normalize_navigator_parent_path(&parent_path_raw);
                let parent_path = format_navigator_parent_path(&parent_path_raw);
                let relative_path_lower = node.relative_path.to_lowercase();
                let display_name_lower = display_name.to_lowercase();
                let parent_path_lower = parent_path_raw.to_lowercase();
                let has_match = !needle.is_empty() && relative_path_lower.contains(needle.as_str());
                let mut display_name_matches_filter =
                    !needle.is_empty() && display_name_lower.contains(needle.as_str());
                let mut parent_path_matches_filter =
                    !needle.is_empty() && parent_path_lower.contains(needle.as_str());
                if has_match && !display_name_matches_filter && !parent_path_matches_filter {
                    if parent_path.is_empty() {
                        display_name_matches_filter = true;
                    } else {
                        parent_path_matches_filter = true;
                    }
                }

                Some(NavigatorRowProjection {
                    source_index,
                    secondary_text: navigator_secondary_text(node),
                    tooltip_text: navigator_row_tooltip_text(&display_name, &full_parent_path),
                    row,
                    display_name,
                    parent_path,
                    display_name_matches_filter,
                    parent_path_matches_filter,
                })
            })
            .collect()
    }

    /// Resolves one flat visible-row index from one source row index.
    pub fn navigator_flat_visual_row_index_for_source_index(
        &self,
        source_index: usize,
    ) -> Option<usize> {
        self.navigator_row_projections()
            .iter()
            .position(|projection| projection.source_index == source_index)
    }

    /// Returns true when search text currently forces flat results mode.
    pub fn navigator_search_forces_flat_mode(&self) -> bool {
        !normalize_filter_needle(&self.entry_filter).is_empty()
    }

    /// Returns the effective Results / Navigator mode after search fallback is applied.
    pub fn effective_navigator_view_mode(&self) -> NavigatorViewMode {
        if self.navigator_search_forces_flat_mode() {
            NavigatorViewMode::Flat
        } else {
            self.navigator_runtime_view_mode
        }
    }

    /// Returns the runtime non-search mode token for UI syncing.
    pub fn navigator_runtime_view_mode_text(&self) -> String {
        self.navigator_runtime_view_mode.as_str().to_string()
    }

    /// Returns the persisted default non-search mode token for UI syncing.
    pub fn default_navigator_view_mode_text(&self) -> String {
        self.default_navigator_view_mode.as_str().to_string()
    }

    /// Returns the effective mode token for UI syncing.
    pub fn navigator_effective_view_mode_text(&self) -> String {
        self.effective_navigator_view_mode().as_str().to_string()
    }

    /// Returns outer workspace mode token for UI syncing.
    pub fn workspace_mode_text(&self) -> String {
        self.workspace_mode.as_str().to_string()
    }

    /// Returns whether the top-level sidebar shell should be visible.
    pub fn sidebar_visible(&self) -> bool {
        self.sidebar_visible
    }

    /// Returns compare-focus raw path for UI syncing.
    pub fn compare_focus_path_raw_text(&self) -> String {
        self.compare_focus_path.raw_text()
    }

    /// Returns the current Compare View row projections derived from the visible compare tree.
    pub fn compare_view_row_projections(&self) -> Vec<CompareViewRowProjection> {
        let foundation = self.compare_foundation_for_projections();
        project_compare_tree_rows(
            foundation.as_ref(),
            &self.compare_focus_path,
            self.show_hidden_files,
            &self.compare_view_expansion_overrides,
        )
        .into_iter()
        .map(|row| {
            let node = foundation
                .as_ref()
                .node(row.relative_path.as_str())
                .expect("compare-view node must exist");
            let (left_kind, right_kind) = compare_view_side_kinds(node);
            let left_present = node.side_presence.left;
            let right_present = node.side_presence.right;
            let (status_label, status_tone) = compare_relation_display(node);

            CompareViewRowProjection {
                relative_path: row.relative_path.clone(),
                depth: row.depth,
                left_present,
                left_icon: if left_present {
                    compare_view_icon_token(left_kind, &node.detail).to_string()
                } else {
                    String::new()
                },
                left_name: if left_present {
                    node.display_name.clone()
                } else {
                    String::new()
                },
                status_label: status_label.to_string(),
                status_tone: status_tone.to_string(),
                right_present,
                right_icon: if right_present {
                    compare_view_icon_token(right_kind, &node.detail).to_string()
                } else {
                    String::new()
                },
                right_name: if right_present {
                    node.display_name.clone()
                } else {
                    String::new()
                },
                is_directory: row.is_directory,
                is_expandable: row.is_expandable,
                is_expanded: row.is_expanded,
            }
        })
        .collect()
    }

    /// Resolves one Compare View visual row index by visible relative path.
    pub fn compare_view_visual_row_index_for_path(&self, relative_path: &str) -> Option<usize> {
        let normalized = relative_path.trim();
        if normalized.is_empty() {
            return None;
        }
        self.compare_view_row_projections()
            .iter()
            .position(|row| row.relative_path == normalized)
    }

    /// Returns the focused Compare View visual row index when available.
    pub fn compare_view_focused_row_index(&self) -> Option<usize> {
        self.compare_row_focus_path
            .as_deref()
            .and_then(|path| self.compare_view_visual_row_index_for_path(path))
    }

    /// Resolves activation behavior for one visible Compare View row.
    pub fn compare_view_row_action(&self, relative_path: &str) -> Option<CompareViewRowAction> {
        let normalized = relative_path.trim();
        if normalized.is_empty() {
            return None;
        }
        let row = self
            .compare_view_row_projections()
            .into_iter()
            .find(|row| row.relative_path == normalized)?;
        let foundation = self.compare_foundation_for_projections();
        let node = foundation.as_ref().node(normalized)?;
        if matches!(node.detail, CompareFoundationDetail::TypeMismatch { .. }) {
            return Some(CompareViewRowAction::TypeMismatch);
        }
        if row.is_expandable {
            return Some(CompareViewRowAction::ToggleDirectory);
        }
        node.source_index
            .map(|_| CompareViewRowAction::OpenFileView)
    }

    /// Returns true when the current Compare View target can move to a parent directory.
    pub fn compare_view_can_go_up(&self) -> bool {
        !self.compare_focus_path.is_root()
    }

    /// Returns true when Compare View can be entered for the current compare result set.
    pub fn compare_view_has_targets(&self) -> bool {
        self.compare_foundation.source_entry_count() > 0
    }

    /// Returns breadcrumb labels for the current Compare View target path.
    pub fn compare_view_breadcrumb_labels(&self) -> Vec<String> {
        self.compare_view_breadcrumb_segments()
            .into_iter()
            .map(|(label, _)| label)
            .collect()
    }

    /// Returns breadcrumb target paths for the current Compare View target path.
    pub fn compare_view_breadcrumb_paths(&self) -> Vec<String> {
        self.compare_view_breadcrumb_segments()
            .into_iter()
            .map(|(_, path)| path)
            .collect()
    }

    /// Returns whether Compare Tree horizontal scrolling is currently locked.
    pub fn compare_view_horizontal_scroll_locked(&self) -> bool {
        self.compare_view_horizontal_scroll_locked
    }

    /// Returns the current Compare Tree quick-locate query text.
    pub fn compare_view_quick_locate_query(&self) -> String {
        self.compare_view_quick_locate_query.clone()
    }

    /// Updates Compare Tree horizontal scroll lock state.
    pub fn set_compare_view_horizontal_scroll_locked(&mut self, locked: bool) -> bool {
        if self.compare_view_horizontal_scroll_locked == locked {
            return false;
        }
        self.compare_view_horizontal_scroll_locked = locked;
        self.sync_compare_tree_session_from_top_level();
        true
    }

    /// Toggles Compare Tree horizontal scroll lock state.
    pub fn toggle_compare_view_horizontal_scroll_locked(&mut self) -> bool {
        self.set_compare_view_horizontal_scroll_locked(!self.compare_view_horizontal_scroll_locked)
    }

    /// Updates Compare Tree quick-locate query text.
    pub fn set_compare_view_quick_locate_query(&mut self, query: &str) -> bool {
        let next = query.trim().to_string();
        if self.compare_view_quick_locate_query == next {
            return false;
        }
        self.compare_view_quick_locate_query = next;
        true
    }

    /// Returns whether the current Compare Tree anchor contains any quick-locate match.
    pub fn compare_view_quick_locate_has_match(&self) -> bool {
        let query = self.compare_view_quick_locate_query.trim().to_string();
        let needle = normalize_filter_needle(query.as_str());
        if needle.is_empty() {
            return false;
        }

        let foundation = self.compare_foundation_for_projections();
        compare_tree_search_paths(
            foundation.as_ref(),
            &self.compare_focus_path,
            self.show_hidden_files,
        )
        .into_iter()
        .any(|path| {
            compare_tree_locate_matches(foundation.as_ref(), path.as_str(), needle.as_str())
        })
    }

    /// Returns one formatted root-pair context string for Compare/File view headers.
    pub fn compare_root_pair_text(&self) -> String {
        let (left_root, right_root) = self
            .compare_tree_session
            .as_ref()
            .map(|session| (session.left_root.as_str(), session.right_root.as_str()))
            .unwrap_or((self.left_root.as_str(), self.right_root.as_str()));
        format!(
            "{} ↔ {}",
            abbreviate_middle(
                left_root.trim(),
                ROOT_PAIR_MAX_CHARS,
                ROOT_PAIR_HEAD_CHARS,
                ROOT_PAIR_TAIL_CHARS,
            ),
            abbreviate_middle(
                right_root.trim(),
                ROOT_PAIR_MAX_CHARS,
                ROOT_PAIR_HEAD_CHARS,
                ROOT_PAIR_TAIL_CHARS,
            ),
        )
    }

    /// Returns current Compare View path text.
    pub fn compare_view_current_path_text(&self) -> String {
        let raw = self.compare_focus_path.raw_text();
        if raw.is_empty() {
            "/".to_string()
        } else {
            abbreviate_middle(
                &raw,
                PATH_DISPLAY_MAX_CHARS,
                PATH_DISPLAY_HEAD_CHARS,
                PATH_DISPLAY_TAIL_CHARS,
            )
        }
    }

    /// Returns current Compare View target status label.
    pub fn compare_view_target_status_label(&self) -> String {
        self.compare_view_target_status()
            .map(|(label, _)| label.to_string())
            .unwrap_or_else(|| "Unavailable".to_string())
    }

    /// Returns current Compare View target status tone.
    pub fn compare_view_target_status_tone(&self) -> String {
        self.compare_view_target_status()
            .map(|(_, tone)| tone.to_string())
            .unwrap_or_else(|| "neutral".to_string())
    }

    /// Returns empty-state title for Compare View content.
    pub fn compare_view_empty_title_text(&self) -> String {
        if self.compare_foundation.source_entry_count() == 0 {
            "No compare result".to_string()
        } else {
            "This level has no entries".to_string()
        }
    }

    /// Returns empty-state body text for Compare View content.
    pub fn compare_view_empty_body_text(&self) -> String {
        if self.compare_foundation.source_entry_count() == 0 {
            "Run Compare from the sidebar, then open Compare Tree from Results / Navigator."
                .to_string()
        } else {
            "The current compare target has no visible compare tree rows under the current view settings."
                .to_string()
        }
    }

    /// Returns whether File View should expose Back to Compare View.
    pub fn can_return_to_compare_view(&self) -> bool {
        self.can_return_to_compare_view
    }

    /// Returns true when current file session should render dedicated Compare File View.
    pub fn compare_file_view_active(&self) -> bool {
        self.active_file_session_uses_compare_file_view()
    }

    /// Returns dedicated Compare File View shell state.
    pub fn compare_file_shell_state(&self) -> CompareFileShellState {
        if self.selected_row.is_none() {
            return if self.has_stale_selection() {
                CompareFileShellState::StaleSelection
            } else {
                CompareFileShellState::NoSelection
            };
        }
        if self.diff_loading {
            return CompareFileShellState::Loading;
        }
        if self
            .diff_error_message
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return CompareFileShellState::Error;
        }
        match self.selected_compare_file.as_ref() {
            Some(panel) if !panel.rows.is_empty() => CompareFileShellState::Ready,
            Some(_) => CompareFileShellState::Unavailable,
            None => CompareFileShellState::Unavailable,
        }
    }

    pub fn compare_file_shell_state_label(&self) -> String {
        match self.compare_file_shell_state() {
            CompareFileShellState::NoSelection => "No Selection".to_string(),
            CompareFileShellState::StaleSelection => "Stale".to_string(),
            CompareFileShellState::Loading => "Loading".to_string(),
            CompareFileShellState::Ready => "Compare Ready".to_string(),
            CompareFileShellState::Unavailable => "Unavailable".to_string(),
            CompareFileShellState::Error => "Load Failed".to_string(),
        }
    }

    pub fn compare_file_shell_state_tone(&self) -> String {
        match self.compare_file_shell_state() {
            CompareFileShellState::NoSelection => "neutral".to_string(),
            CompareFileShellState::StaleSelection => "warn".to_string(),
            CompareFileShellState::Loading => "info".to_string(),
            CompareFileShellState::Ready => "success".to_string(),
            CompareFileShellState::Unavailable => "warn".to_string(),
            CompareFileShellState::Error => "error".to_string(),
        }
    }

    pub fn compare_file_shell_title_text(&self) -> String {
        match self.compare_file_shell_state() {
            CompareFileShellState::NoSelection => "No compare file selected".to_string(),
            CompareFileShellState::StaleSelection => "Compare file no longer active".to_string(),
            CompareFileShellState::Loading => "Loading file comparison".to_string(),
            CompareFileShellState::Ready => "Side-by-side comparison ready".to_string(),
            CompareFileShellState::Unavailable => "Compare File View unavailable".to_string(),
            CompareFileShellState::Error => "Failed to load file comparison".to_string(),
        }
    }

    pub fn compare_file_shell_body_text(&self) -> String {
        match self.compare_file_shell_state() {
            CompareFileShellState::NoSelection => {
                "Choose one file from Compare Tree to open the dedicated Compare File View."
                    .to_string()
            }
            CompareFileShellState::StaleSelection => {
                "The previously opened compare file is no longer part of the current visible Results / Navigator set."
                    .to_string()
            }
            CompareFileShellState::Loading => {
                "Projecting aligned rows, line numbers, and inline compare highlights."
                    .to_string()
            }
            CompareFileShellState::Ready => {
                "Aligned base/target rows are ready for side-by-side reading, scrolling, and copy."
                    .to_string()
            }
            CompareFileShellState::Unavailable => {
                "This selection does not currently provide compare-file content in the dedicated renderer."
                    .to_string()
            }
            CompareFileShellState::Error => {
                "The dedicated Compare File View could not be loaded in this session."
                    .to_string()
            }
        }
    }

    pub fn compare_file_shell_note_text(&self) -> String {
        match self.compare_file_shell_state() {
            CompareFileShellState::NoSelection => {
                "Compare-originated file tabs stay attached to the current Compare Tree session."
                    .to_string()
            }
            CompareFileShellState::StaleSelection => {
                "Adjust filters or reopen the file from Compare Tree.".to_string()
            }
            CompareFileShellState::Loading => {
                "Using one shared vertical projection with independent base/target horizontal scroll."
                    .to_string()
            }
            CompareFileShellState::Ready => {
                if self.compare_file_truncated() {
                    "Current compare rows were truncated by the underlying diff payload."
                        .to_string()
                } else {
                    String::new()
                }
            }
            CompareFileShellState::Unavailable => self.diff_warning_text(),
            CompareFileShellState::Error => self
                .diff_error_message
                .clone()
                .unwrap_or_else(|| "Compare File View failed to load.".to_string()),
        }
    }

    pub fn compare_file_summary_text(&self) -> String {
        self.selected_compare_file
            .as_ref()
            .map(|panel| panel.summary_text.clone())
            .unwrap_or_default()
    }

    pub fn compare_file_warning_text(&self) -> String {
        self.selected_compare_file
            .as_ref()
            .and_then(|panel| panel.warning.clone())
            .unwrap_or_else(|| self.diff_warning_text())
    }

    pub fn compare_file_truncated(&self) -> bool {
        self.selected_compare_file
            .as_ref()
            .map(|panel| panel.truncated)
            .unwrap_or(false)
    }

    pub fn compare_file_has_rows(&self) -> bool {
        self.selected_compare_file
            .as_ref()
            .map(|panel| !panel.rows.is_empty())
            .unwrap_or(false)
    }

    pub fn compare_file_helper_text(&self) -> String {
        match self.compare_file_shell_state() {
            CompareFileShellState::Ready => {
                let summary = self.compare_file_summary_text();
                let warning = self.compare_file_warning_text();
                match (
                    summary.trim().is_empty(),
                    warning.trim().is_empty(),
                    self.compare_file_truncated(),
                ) {
                    (false, true, false) => summary,
                    (false, false, false) => format!("{summary}  ·  {warning}"),
                    (false, true, true) => format!("{summary}  ·  truncated"),
                    (false, false, true) => format!("{summary}  ·  {warning}  ·  truncated"),
                    (true, false, false) => warning,
                    (true, false, true) => format!("{warning}  ·  truncated"),
                    (true, true, true) => "Compare payload truncated".to_string(),
                    (true, true, false) => String::new(),
                }
            }
            _ => self.compare_file_shell_note_text(),
        }
    }

    pub fn compare_file_row_projections(&self) -> Vec<CompareFileRowViewModel> {
        self.selected_compare_file
            .as_ref()
            .map(|panel| panel.rows.clone())
            .unwrap_or_default()
    }

    /// Returns current File View title text derived from the selected file path.
    pub fn file_view_title_text(&self) -> String {
        self.selected_relative_path
            .as_deref()
            .and_then(normalize_optional_text)
            .map(|value| split_relative_path_leaf(&value).1)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "No file selected".to_string())
    }

    /// Returns the File View compare-status label.
    pub fn file_view_compare_status_label(&self) -> String {
        self.selected_entry_compare_status()
            .map(|(label, _)| label.to_string())
            .unwrap_or_else(|| "Unavailable".to_string())
    }

    /// Returns the File View compare-status tone.
    pub fn file_view_compare_status_tone(&self) -> String {
        self.selected_entry_compare_status()
            .map(|(_, tone)| tone.to_string())
            .unwrap_or_else(|| "neutral".to_string())
    }

    /// Returns the File View path-context text.
    pub fn file_view_path_context_text(&self) -> String {
        let Some(node) = self.selected_entry_compare_node() else {
            return String::new();
        };

        let relative_path = node.relative_path.trim();
        if relative_path.is_empty() {
            return String::new();
        }

        let left_path = node.side_presence.left.then_some(relative_path);
        let right_path = node.side_presence.right.then_some(relative_path);
        match (left_path, right_path) {
            (Some(left), Some(right)) if left == right => format!(
                "Compare Path · {}",
                abbreviate_middle(
                    left,
                    PATH_DISPLAY_MAX_CHARS,
                    PATH_DISPLAY_HEAD_CHARS,
                    PATH_DISPLAY_TAIL_CHARS,
                )
            ),
            (left, right) => format!(
                "Source Path · {}    Target Path · {}",
                left.map(|value| {
                    abbreviate_middle(
                        value,
                        PATH_DISPLAY_MAX_CHARS,
                        PATH_DISPLAY_HEAD_CHARS,
                        PATH_DISPLAY_TAIL_CHARS,
                    )
                })
                .unwrap_or_else(|| "-".to_string()),
                right
                    .map(|value| {
                        abbreviate_middle(
                            value,
                            PATH_DISPLAY_MAX_CHARS,
                            PATH_DISPLAY_HEAD_CHARS,
                            PATH_DISPLAY_TAIL_CHARS,
                        )
                    })
                    .unwrap_or_else(|| "-".to_string())
            ),
        }
    }

    /// Updates the outer workspace mode.
    pub fn set_workspace_mode(&mut self, mode: WorkspaceMode) {
        self.workspace_mode = mode;
    }

    /// Updates the top-level sidebar shell visibility.
    pub fn set_sidebar_visible(&mut self, visible: bool) -> bool {
        if self.sidebar_visible == visible {
            return false;
        }
        self.sidebar_visible = visible;
        true
    }

    /// Toggles the top-level sidebar shell visibility.
    pub fn toggle_sidebar_visible(&mut self) -> bool {
        self.set_sidebar_visible(!self.sidebar_visible)
    }

    /// Updates compare focus path, clamping to an existing compare target directory or root.
    pub fn set_compare_focus_path(&mut self, focus: CompareFocusPath) -> bool {
        let normalized = self
            .compare_foundation_for_projections()
            .as_ref()
            .clamp_compare_focus_path(&focus);
        let previous_focus = self.compare_focus_path.clone();
        self.compare_focus_path = normalized;
        self.reconcile_compare_focus_visibility();
        if self.compare_focus_path == previous_focus {
            return false;
        }
        self.reconcile_compare_row_focus(None);
        self.bump_compare_view_projection_revision();
        self.sync_compare_tree_session_from_top_level();
        true
    }

    /// Moves compare focus back to the compare root.
    #[allow(dead_code)]
    pub fn reset_compare_focus_path(&mut self) -> bool {
        self.set_compare_focus_path(CompareFocusPath::root())
    }

    /// Moves compare focus to the current focus target's parent directory, or root.
    #[allow(dead_code)]
    pub fn focus_compare_parent(&mut self) -> bool {
        let parent = self
            .compare_foundation_for_projections()
            .as_ref()
            .parent_compare_focus_path(&self.compare_focus_path);
        self.set_compare_focus_path(parent)
    }

    /// Toggles one Compare View directory row when it is expandable.
    pub fn toggle_compare_view_node(&mut self, key: &str) -> bool {
        let foundation = self.compare_foundation_for_projections();
        let Some((normalized_key, path_depth)) =
            compare_tree_toggle_target(foundation.as_ref(), key)
        else {
            return false;
        };

        let default_expanded =
            compare_tree_expansion_state(normalized_key.as_str(), path_depth, &BTreeMap::new());
        let current = self
            .compare_view_expansion_overrides
            .get(normalized_key.as_str())
            .copied()
            .unwrap_or(default_expanded);
        let next = !current;
        if next == default_expanded {
            self.compare_view_expansion_overrides
                .remove(normalized_key.as_str());
        } else {
            self.compare_view_expansion_overrides
                .insert(normalized_key, next);
        }
        self.bump_compare_view_projection_revision();
        self.reconcile_compare_row_focus(Some(key));
        self.sync_compare_tree_session_from_top_level();
        true
    }

    /// Expands compare-tree ancestors needed to reveal one nested path under the current anchor.
    pub fn reveal_compare_view_path(&mut self, relative_path: &str) -> bool {
        let foundation = self.compare_foundation_for_projections();
        let mut changed = false;
        for (key, path_depth) in compare_tree_reveal_targets(
            foundation.as_ref(),
            &self.compare_focus_path,
            relative_path,
        ) {
            let default_expanded =
                compare_tree_expansion_state(key.as_str(), path_depth, &BTreeMap::new());
            if default_expanded {
                changed |= self
                    .compare_view_expansion_overrides
                    .remove(key.as_str())
                    .is_some();
            } else if self
                .compare_view_expansion_overrides
                .get(key.as_str())
                .copied()
                != Some(true)
            {
                self.compare_view_expansion_overrides.insert(key, true);
                changed = true;
            }
        }

        if changed {
            self.bump_compare_view_projection_revision();
            self.sync_compare_tree_session_from_top_level();
        }
        changed
    }

    /// Focuses one Compare View row when it is visible in the current compare tree.
    pub fn set_compare_row_focus_path(&mut self, relative_path: Option<&str>) -> bool {
        let next = self.resolve_compare_row_focus_path(relative_path);
        if self.compare_row_focus_path == next {
            return false;
        }
        self.compare_row_focus_path = next;
        self.sync_compare_tree_session_from_top_level();
        true
    }

    /// Queues one Compare View ensure-visible request when the target row is visible.
    pub fn request_compare_view_scroll_to_path(&mut self, relative_path: &str) -> bool {
        let normalized = relative_path.trim();
        if normalized.is_empty()
            || self
                .compare_view_visual_row_index_for_path(normalized)
                .is_none()
        {
            return false;
        }
        self.compare_view_scroll_target_relative_path = Some(normalized.to_string());
        self.compare_view_scroll_request_revision =
            self.compare_view_scroll_request_revision.wrapping_add(1);
        true
    }

    /// Jumps to the current or next quick-locate match inside the current Compare Tree anchor.
    pub fn locate_next_in_compare_view(&mut self, advance: bool) -> bool {
        let query = self.compare_view_quick_locate_query.trim().to_string();
        let needle = normalize_filter_needle(query.as_str());
        if needle.is_empty() {
            return false;
        }

        let target_path = {
            let foundation = self.compare_foundation_for_projections();
            let candidates = compare_tree_search_paths(
                foundation.as_ref(),
                &self.compare_focus_path,
                self.show_hidden_files,
            );
            if candidates.is_empty() {
                return false;
            }

            let current_index = self
                .compare_row_focus_path
                .as_deref()
                .and_then(|current| candidates.iter().position(|path| path == current));
            let target_index = if advance {
                candidates
                    .iter()
                    .enumerate()
                    .find(|(index, path)| {
                        *index > current_index.unwrap_or(usize::MAX)
                            && compare_tree_locate_matches(
                                foundation.as_ref(),
                                path.as_str(),
                                needle.as_str(),
                            )
                    })
                    .or_else(|| {
                        candidates.iter().enumerate().find(|(_, path)| {
                            compare_tree_locate_matches(
                                foundation.as_ref(),
                                path.as_str(),
                                needle.as_str(),
                            )
                        })
                    })
                    .map(|(index, _)| index)
            } else {
                candidates
                    .iter()
                    .enumerate()
                    .find(|(index, path)| {
                        current_index.is_none_or(|current| *index >= current)
                            && compare_tree_locate_matches(
                                foundation.as_ref(),
                                path.as_str(),
                                needle.as_str(),
                            )
                    })
                    .or_else(|| {
                        candidates.iter().enumerate().find(|(_, path)| {
                            compare_tree_locate_matches(
                                foundation.as_ref(),
                                path.as_str(),
                                needle.as_str(),
                            )
                        })
                    })
                    .map(|(index, _)| index)
            };

            target_index.and_then(|index| candidates.get(index).cloned())
        };

        let Some(target_path) = target_path else {
            return false;
        };

        let mut changed = false;
        changed |= self.reveal_compare_view_path(target_path.as_str());
        changed |= self.set_compare_row_focus_path(Some(target_path.as_str()));
        changed |= self.request_compare_view_scroll_to_path(target_path.as_str());
        changed
    }

    /// Jumps to the previous quick-locate match inside the current Compare Tree anchor.
    pub fn locate_previous_in_compare_view(&mut self) -> bool {
        let query = self.compare_view_quick_locate_query.trim().to_string();
        let needle = normalize_filter_needle(query.as_str());
        if needle.is_empty() {
            return false;
        }

        let target_path = {
            let foundation = self.compare_foundation_for_projections();
            let candidates = compare_tree_search_paths(
                foundation.as_ref(),
                &self.compare_focus_path,
                self.show_hidden_files,
            );
            if candidates.is_empty() {
                return false;
            }

            let current_index = self
                .compare_row_focus_path
                .as_deref()
                .and_then(|current| candidates.iter().position(|path| path == current));
            let target_index = candidates
                .iter()
                .enumerate()
                .rev()
                .find(|(index, path)| {
                    current_index.is_none_or(|current| *index < current)
                        && compare_tree_locate_matches(
                            foundation.as_ref(),
                            path.as_str(),
                            needle.as_str(),
                        )
                })
                .or_else(|| {
                    candidates.iter().enumerate().rev().find(|(_, path)| {
                        compare_tree_locate_matches(
                            foundation.as_ref(),
                            path.as_str(),
                            needle.as_str(),
                        )
                    })
                })
                .map(|(index, _)| index);

            target_index.and_then(|index| candidates.get(index).cloned())
        };

        let Some(target_path) = target_path else {
            return false;
        };

        let mut changed = false;
        changed |= self.reveal_compare_view_path(target_path.as_str());
        changed |= self.set_compare_row_focus_path(Some(target_path.as_str()));
        changed |= self.request_compare_view_scroll_to_path(target_path.as_str());
        changed
    }

    /// Removes compare-tree expansion overrides that no longer map to expandable directories.
    pub fn prune_compare_view_expansion_overrides(&mut self) -> bool {
        let foundation = self.compare_foundation_for_projections().into_owned();
        let previous = self.compare_view_expansion_overrides.clone();
        self.compare_view_expansion_overrides
            .retain(
                |key, expanded| match compare_tree_toggle_target(&foundation, key) {
                    Some((_, path_depth)) => {
                        *expanded != compare_tree_expansion_state(key, path_depth, &BTreeMap::new())
                    }
                    None => false,
                },
            );
        let changed = previous != self.compare_view_expansion_overrides;
        if changed {
            self.sync_compare_tree_session_from_top_level();
        }
        changed
    }

    /// Updates the persisted non-search default mode.
    pub fn set_default_navigator_view_mode(&mut self, mode: NavigatorViewMode) {
        self.default_navigator_view_mode = mode;
    }

    /// Updates the non-search runtime mode.
    pub fn set_navigator_runtime_view_mode(&mut self, mode: NavigatorViewMode) {
        self.navigator_runtime_view_mode = mode;
    }

    /// Updates the flat-mode search filter and refresh revision.
    pub fn set_entry_filter(&mut self, filter: String) {
        if self.entry_filter == filter {
            return;
        }
        self.entry_filter = filter;
        self.bump_navigator_flat_projection_revision();
    }

    /// Returns visible tree row projections for tree mode rendering.
    pub fn navigator_tree_row_projections(&self) -> Vec<NavigatorTreeRowProjection> {
        self.navigator_tree_projection().rows
    }

    /// Resolves one tree visible-row index from one source row index.
    pub fn navigator_tree_visual_row_index_for_source_index(
        &self,
        source_index: usize,
    ) -> Option<usize> {
        self.navigator_tree_row_projections()
            .iter()
            .position(|projection| projection.source_index == Some(source_index))
    }

    /// Queues one flat ensure-visible request when the source row is visible.
    pub fn request_navigator_flat_scroll_to_source_index(&mut self, source_index: usize) -> bool {
        if self
            .navigator_flat_visual_row_index_for_source_index(source_index)
            .is_none()
        {
            return false;
        }
        self.navigator_flat_scroll_target_source_index = Some(source_index);
        self.navigator_flat_scroll_request_revision =
            self.navigator_flat_scroll_request_revision.wrapping_add(1);
        true
    }

    /// Queues one tree ensure-visible request when the source row is visible.
    pub fn request_navigator_tree_scroll_to_source_index(&mut self, source_index: usize) -> bool {
        if self
            .navigator_tree_visual_row_index_for_source_index(source_index)
            .is_none()
        {
            return false;
        }
        self.navigator_tree_scroll_target_source_index = Some(source_index);
        self.navigator_tree_scroll_request_revision =
            self.navigator_tree_scroll_request_revision.wrapping_add(1);
        true
    }

    /// Returns true when a directory node toggle was applied.
    pub fn toggle_navigator_tree_node(&mut self, key: &str) -> bool {
        let foundation = self.compare_foundation_for_projections();
        let Some((normalized_key, path_depth)) =
            navigator_tree_toggle_target(foundation.as_ref(), key)
        else {
            return false;
        };

        let default_expanded = path_depth <= 1;
        let current = self
            .navigator_tree_expansion_overrides
            .get(normalized_key.as_str())
            .copied()
            .unwrap_or(default_expanded);
        let next = !current;
        if next == default_expanded {
            self.navigator_tree_expansion_overrides
                .remove(normalized_key.as_str());
        } else {
            self.navigator_tree_expansion_overrides
                .insert(normalized_key, next);
        }
        self.bump_navigator_tree_projection_revision();
        true
    }

    /// Expands all ancestor directories required to reveal one file path in tree mode.
    pub fn reveal_navigator_tree_path(&mut self, relative_path: &str) -> bool {
        let foundation = self.compare_foundation_for_projections();
        let mut changed = false;
        for (key, path_depth) in navigator_tree_reveal_targets(foundation.as_ref(), relative_path) {
            let default_expanded = path_depth <= 1;
            if default_expanded {
                changed |= self
                    .navigator_tree_expansion_overrides
                    .remove(key.as_str())
                    .is_some();
            } else if self
                .navigator_tree_expansion_overrides
                .get(key.as_str())
                .copied()
                != Some(true)
            {
                self.navigator_tree_expansion_overrides.insert(key, true);
                changed = true;
            }
        }

        if changed {
            self.bump_navigator_tree_projection_revision();
        }
        changed
    }

    /// Removes expansion overrides that no longer map to expandable directories.
    pub fn prune_navigator_tree_expansion_overrides(&mut self) -> bool {
        let foundation = self.compare_foundation_for_projections().into_owned();
        let previous = self.navigator_tree_expansion_overrides.clone();
        self.navigator_tree_expansion_overrides
            .retain(
                |key, expanded| match navigator_tree_toggle_target(&foundation, key) {
                    Some((_, path_depth)) => *expanded != (path_depth <= 1),
                    None => false,
                },
            );
        previous != self.navigator_tree_expansion_overrides
    }

    /// Updates status filter scope in canonical form.
    pub fn set_entry_status_filter(&mut self, filter: &str) {
        let normalized = normalize_status_filter_token(filter);
        if self.entry_status_filter == normalized {
            return;
        }
        self.entry_status_filter = normalized;
        self.bump_navigator_projection_revisions();
    }

    /// Updates the hidden-files preference and navigator revisions.
    pub fn set_show_hidden_files(&mut self, show_hidden_files: bool) {
        if self.show_hidden_files == show_hidden_files {
            return;
        }
        self.show_hidden_files = show_hidden_files;
        self.reconcile_compare_focus_visibility();
        let preferred_focus = self.compare_row_focus_path.clone();
        self.reconcile_compare_row_focus(preferred_focus.as_deref());
        if let Some(path) = self.compare_row_focus_path.clone() {
            self.request_compare_view_scroll_to_path(path.as_str());
        }
        self.bump_navigator_projection_revisions();
        self.bump_compare_view_projection_revision();
        self.sync_compare_tree_session_from_top_level();
    }

    /// Marks both flat/tree navigator projections dirty after compare data changes.
    pub fn mark_navigator_projection_revisions(&mut self) {
        self.bump_navigator_projection_revisions();
    }

    /// Returns true when one source row index is currently visible by filter.
    pub fn is_row_visible_in_filter(&self, index: usize) -> bool {
        self.compare_foundation_for_projections()
            .as_ref()
            .source_node(index)
            .map(|node| self.source_node_visible_in_results(node))
            .unwrap_or(false)
    }

    /// Returns true when one source row remains part of the active navigator membership set.
    pub fn is_row_member_in_active_results(&self, index: usize) -> bool {
        match self.effective_navigator_view_mode() {
            NavigatorViewMode::Flat => self.is_row_visible_in_filter(index),
            NavigatorViewMode::Tree => self
                .navigator_tree_projection()
                .selectable_source_indices
                .contains(&index),
        }
    }

    /// Resolves one source row index by normalized relative path.
    pub fn row_index_for_relative_path(&self, relative_path: &str) -> Option<usize> {
        let normalized = relative_path.trim();
        if normalized.is_empty() {
            return None;
        }
        self.compare_foundation_for_projections()
            .as_ref()
            .source_index_for_relative_path(normalized)
    }

    /// Returns collection summary text for Results / Navigator.
    pub fn results_collection_text(&self) -> String {
        let status_filter = normalize_status_filter_token(&self.entry_status_filter);
        let foundation = self.compare_foundation_for_projections();
        let visible = foundation
            .source_nodes()
            .filter(|node| self.source_node_visible_in_results(node))
            .count();
        let hidden_by_settings = if self.show_hidden_files {
            0
        } else {
            foundation
                .source_nodes()
                .filter(|node| {
                    self.source_node_matches_filter_controls(node, status_filter.as_str())
                        && is_hidden_relative_path(&node.relative_path)
                })
                .count()
        };
        let total =
            summary_metric_usize(&self.summary_text, "total=").unwrap_or(self.entry_rows.len());
        let query = self.entry_filter.trim();
        let mut parts = vec![format!("Showing {visible} / {total}")];
        if hidden_by_settings > 0 {
            parts.push(format!("{hidden_by_settings} hidden by Settings"));
        }
        if !query.is_empty() {
            parts.push(format!(
                "Search: \"{}\"",
                abbreviate_middle(&sanitize_inline_query(query), 30, 18, 8)
            ));
        }
        if status_filter == "all" {
            if query.is_empty() {
                parts.push("All results".to_string());
            }
        } else {
            parts.push(status_filter_label(status_filter.as_str()).to_string());
        }
        parts.join(" · ")
    }

    /// Returns compact compare summary text for sidebar status section.
    pub fn compact_summary_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }
        let mut parts = Vec::new();
        if let Some(value) = self.compare_mode_label() {
            parts.push(value);
        }
        if let Some(value) = summary_metric(&self.summary_text, "total=") {
            parts.push(format!("Total {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "different=") {
            parts.push(format!("Changed {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "left_only=") {
            parts.push(format!("Left {value}"));
        }
        if let Some(value) = summary_metric(&self.summary_text, "right_only=") {
            parts.push(format!("Right {value}"));
        }
        if let Some(value) = self.compare_deferred_count().filter(|value| *value > 0) {
            parts.push(format!("{value} deferred"));
        }
        if let Some(value) = self.compare_oversized_count().filter(|value| *value > 0) {
            parts.push(format!("{value} oversized"));
        }
        if self.truncated {
            parts.push("Truncated".to_string());
        }
        if parts.is_empty() {
            if self.error_message.is_some() {
                return "Compare failed".to_string();
            }
            if !self.warning_lines.is_empty() {
                return format_warning_count(self.warning_lines.len());
            }
            return abbreviate_middle(&self.summary_text, 96, 56, 36);
        }
        parts.join(" · ")
    }

    /// Returns key compare metrics in short desktop-friendly format.
    pub fn compare_metrics_text(&self) -> String {
        if self.summary_text.trim().is_empty() {
            return String::new();
        }
        let total = summary_metric(&self.summary_text, "total=").unwrap_or_else(|| "0".to_string());
        let changed =
            summary_metric(&self.summary_text, "different=").unwrap_or_else(|| "0".to_string());
        let left =
            summary_metric(&self.summary_text, "left_only=").unwrap_or_else(|| "0".to_string());
        let right =
            summary_metric(&self.summary_text, "right_only=").unwrap_or_else(|| "0".to_string());
        format!("Total {total} · Changed {changed} · Left {left} · Right {right}")
    }

    /// Returns true when compare summary indicates deferred detail entries.
    pub fn compare_has_deferred(&self) -> bool {
        summary_metric_usize(&self.summary_text, "deferred=").unwrap_or(0) > 0
    }

    /// Returns true when compare summary indicates oversized text entries.
    pub fn compare_has_oversized(&self) -> bool {
        summary_metric_usize(&self.summary_text, "oversized_text=").unwrap_or(0) > 0
    }

    /// Returns true when Compare Status has any compare report detail to expose.
    pub fn compare_status_has_detail(&self) -> bool {
        !self.summary_text.trim().is_empty()
            || !self.warning_lines.is_empty()
            || self
                .error_message
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
    }

    /// Returns one collapsed note line for Compare Status.
    pub fn compare_status_note_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }

        let mut parts = Vec::new();
        if let Some(value) = self.compare_mode_note_text() {
            parts.push(value);
        }
        if let Some(value) = self.compare_deferred_count().filter(|value| *value > 0) {
            parts.push(format!("{value} deferred"));
        }
        if let Some(value) = self.compare_oversized_count().filter(|value| *value > 0) {
            parts.push(format!("{value} oversized"));
        }
        if !self.warning_lines.is_empty() {
            parts.push(format_warning_count(self.warning_lines.len()));
        }
        if self.truncated {
            parts.push("Truncated output".to_string());
        }

        parts.join(" · ")
    }

    /// Returns concise copy-ready text for Compare Status.
    pub fn compare_summary_copy_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }

        let mut lines = vec!["Compare Summary".to_string()];
        if let Some(status) = normalize_optional_text(&self.status_text) {
            lines.push(status);
        }
        if let Some(metrics) = normalize_optional_text(&self.compare_metrics_text()) {
            lines.push(metrics);
        }
        if let Some(note) = normalize_optional_text(&self.compare_status_note_text()) {
            lines.push(note);
        }
        if let Some(error) = self
            .error_message
            .as_deref()
            .and_then(normalize_optional_text)
        {
            lines.push(format!("Error: {error}"));
        }

        lines.join("\n")
    }

    /// Returns structured copy-ready detail text for Compare Status.
    pub fn compare_detail_copy_text(&self) -> String {
        if !self.compare_status_has_detail() {
            return String::new();
        }

        let mut blocks = Vec::new();
        if let Some(status) = normalize_optional_text(&self.status_text) {
            blocks.push(format!("Status\n{status}"));
        }
        if let Some(metrics) = normalize_optional_text(&self.compare_metrics_text()) {
            blocks.push(format!("Results\n{metrics}"));
        }

        let mut diagnostics = Vec::new();
        if let Some(mode) = self.compare_mode_label() {
            diagnostics.push(mode);
        }
        if let Some(value) = self.compare_deferred_count().filter(|value| *value > 0) {
            diagnostics.push(format!("{value} deferred detail entries"));
        }
        if let Some(value) = self.compare_oversized_count().filter(|value| *value > 0) {
            diagnostics.push(format!("{value} oversized text entries"));
        }
        if self.truncated {
            diagnostics.push("Truncated compare output".to_string());
        }
        if !diagnostics.is_empty() {
            blocks.push(format!("Detail\n{}", diagnostics.join("\n")));
        }

        if let Some(summary) = normalize_optional_text(&self.compact_summary_text()) {
            blocks.push(format!("Summary\n{summary}"));
        }

        if !self.warning_lines.is_empty() {
            blocks.push(format!(
                "Warnings\n{}",
                self.warning_lines
                    .iter()
                    .map(|warning| format!("• {}", warning.trim()))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if let Some(error) = self
            .error_message
            .as_deref()
            .and_then(normalize_optional_text)
        {
            blocks.push(format!("Error\n{error}"));
        }

        format!("Compare Detail\n\n{}", blocks.join("\n\n"))
    }

    /// Returns selected relative path text for UI rendering.
    pub fn selected_relative_path_text(&self) -> String {
        let raw = self.selected_relative_path.clone().unwrap_or_default();
        abbreviate_middle(
            &raw,
            PATH_DISPLAY_MAX_CHARS,
            PATH_DISPLAY_HEAD_CHARS,
            PATH_DISPLAY_TAIL_CHARS,
        )
    }

    /// Returns selected compare row status token for UI rendering.
    pub fn selected_row_status_text(&self) -> String {
        self.selected_row_status_token().to_string()
    }

    /// Returns detailed diff warning text for UI rendering.
    pub fn diff_warning_text(&self) -> String {
        self.diff_warning.clone().unwrap_or_default()
    }

    /// Returns flattened detailed diff rows for viewer rendering.
    pub fn diff_viewer_rows(&self) -> Vec<DiffViewerRow> {
        let mut out = Vec::new();
        let Some(diff) = &self.selected_diff else {
            return out;
        };

        for hunk in &diff.hunks {
            out.push(DiffViewerRow {
                old_line_no: String::new(),
                new_line_no: String::new(),
                marker: "@@".to_string(),
                content: hunk.header(),
                row_kind: "hunk".to_string(),
            });
            for line in &hunk.lines {
                out.push(DiffViewerRow {
                    old_line_no: line
                        .old_line_no
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    new_line_no: line
                        .new_line_no
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    marker: line.marker().to_string(),
                    content: line.content.clone(),
                    row_kind: line.kind_tag().to_string(),
                });
            }
        }

        out
    }

    /// Returns true when current detailed diff has at least one rendered row.
    pub fn diff_has_rows(&self) -> bool {
        self.selected_diff
            .as_ref()
            .map(|diff| {
                !diff.hunks.is_empty() && diff.hunks.iter().any(|hunk| !hunk.lines.is_empty())
            })
            .unwrap_or(false)
    }

    /// Clears detailed diff panel state without changing compare state.
    pub fn clear_diff_panel(&mut self) {
        self.diff_loading = false;
        self.diff_error_message = None;
        self.selected_diff = None;
        self.diff_warning = None;
        self.diff_truncated = false;
    }

    /// Clears dedicated Compare File View state without changing compare selection.
    pub fn clear_compare_file_panel(&mut self) {
        self.selected_compare_file = None;
    }

    /// Clears AI analysis panel state without changing compare/diff state.
    pub fn clear_analysis_panel(&mut self) {
        self.analysis_loading = false;
        self.analysis_error_message = None;
        self.analysis_result = None;
    }

    /// Returns normalized Analysis panel state for header and body rendering.
    pub fn analysis_panel_state(&self) -> AnalysisPanelState {
        if self.selected_row.is_none() {
            return if self.has_stale_selection() {
                AnalysisPanelState::StaleSelection
            } else {
                AnalysisPanelState::NoSelection
            };
        }
        if self.analysis_loading {
            return AnalysisPanelState::Loading;
        }
        if self
            .analysis_error_message
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return AnalysisPanelState::Error;
        }
        if self.analysis_result.is_some() {
            return AnalysisPanelState::Success;
        }
        if self.analysis_waiting_for_diff_context() {
            return AnalysisPanelState::WaitingForDiff;
        }
        if self.analysis_can_start_now() {
            return AnalysisPanelState::Ready;
        }
        if self.analysis_selection_unavailable() {
            return AnalysisPanelState::Unavailable;
        }
        AnalysisPanelState::Ready
    }

    /// Returns short state badge text for Analysis shell.
    pub fn analysis_state_label(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "No Selection".to_string(),
            AnalysisPanelState::StaleSelection => "Stale".to_string(),
            AnalysisPanelState::WaitingForDiff => "Waiting".to_string(),
            AnalysisPanelState::Ready => "Ready".to_string(),
            AnalysisPanelState::Unavailable => "Unavailable".to_string(),
            AnalysisPanelState::Loading => "Analyzing".to_string(),
            AnalysisPanelState::Error => "Failed".to_string(),
            AnalysisPanelState::Success => "Ready".to_string(),
        }
    }

    /// Returns stable token for Analysis shell state branching in UI layer.
    pub fn analysis_state_token(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "no-selection".to_string(),
            AnalysisPanelState::StaleSelection => "stale-selection".to_string(),
            AnalysisPanelState::WaitingForDiff => "waiting".to_string(),
            AnalysisPanelState::Ready => "ready".to_string(),
            AnalysisPanelState::Unavailable => "unavailable".to_string(),
            AnalysisPanelState::Loading => "loading".to_string(),
            AnalysisPanelState::Error => "error".to_string(),
            AnalysisPanelState::Success => "success".to_string(),
        }
    }

    /// Returns tone for Analysis shell state surfaces.
    pub fn analysis_state_tone(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "neutral".to_string(),
            AnalysisPanelState::StaleSelection => "warn".to_string(),
            AnalysisPanelState::WaitingForDiff => "info".to_string(),
            AnalysisPanelState::Ready => "neutral".to_string(),
            AnalysisPanelState::Unavailable => "warn".to_string(),
            AnalysisPanelState::Loading => "info".to_string(),
            AnalysisPanelState::Error => "error".to_string(),
            AnalysisPanelState::Success => "success".to_string(),
        }
    }

    /// Returns compact header summary text for Analysis shell.
    pub fn analysis_header_summary_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => {
                "Choose one changed text file from Results / Navigator.".to_string()
            }
            AnalysisPanelState::StaleSelection => {
                "The previous selection is no longer part of the current Results / Navigator set."
                    .to_string()
            }
            AnalysisPanelState::WaitingForDiff => {
                if !self.analysis_hint_text().trim().is_empty() {
                    abbreviate_middle(&self.analysis_hint_text(), 116, 88, 22)
                } else {
                    "Diff context is loading for the selected file.".to_string()
                }
            }
            AnalysisPanelState::Ready => {
                "Diff context is ready. Run Analyze to generate a review conclusion.".to_string()
            }
            AnalysisPanelState::Unavailable => {
                if !self.analysis_hint_text().trim().is_empty() {
                    abbreviate_middle(&self.analysis_hint_text(), 116, 88, 22)
                } else {
                    "Analysis is unavailable for this selection.".to_string()
                }
            }
            AnalysisPanelState::Loading => {
                "Building a structured review conclusion for the selected diff.".to_string()
            }
            AnalysisPanelState::Error => {
                "The last analysis did not complete for the current file.".to_string()
            }
            AnalysisPanelState::Success => self.analysis_summary_text(),
        }
    }

    /// Returns weak technical context text for Analysis header/helper strip.
    pub fn analysis_technical_context_text(&self) -> String {
        let mut parts = vec![
            format!("Provider {}", self.analysis_provider_mode_text()),
            if self.analysis_remote_mode() {
                if self.analysis_remote_config_ready() {
                    "remote ready".to_string()
                } else {
                    "remote config incomplete".to_string()
                }
            } else {
                "local deterministic".to_string()
            },
            format!("timeout {}s", self.analysis_timeout_text()),
        ];

        if self.diff_truncated {
            parts.push("diff context truncated".to_string());
        }

        if let Some(warning) = self
            .diff_warning
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            parts.push(abbreviate_middle(warning, 96, 72, 18));
        }

        parts.join(" · ")
    }

    /// Returns provider readiness badge label for Analysis header.
    pub fn analysis_provider_status_label(&self) -> String {
        if self.analysis_remote_mode() {
            if self.analysis_remote_config_ready() {
                "remote ready".to_string()
            } else {
                "remote config".to_string()
            }
        } else {
            "local mock".to_string()
        }
    }

    /// Returns provider readiness badge tone for Analysis header.
    pub fn analysis_provider_status_tone(&self) -> String {
        if self.analysis_remote_mode() {
            if self.analysis_remote_config_ready() {
                "info".to_string()
            } else {
                "warn".to_string()
            }
        } else {
            "neutral".to_string()
        }
    }

    /// Returns state surface title for Analysis mode.
    pub fn analysis_state_title_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => "No file selected".to_string(),
            AnalysisPanelState::StaleSelection => "Selection no longer active".to_string(),
            AnalysisPanelState::WaitingForDiff => {
                "Analysis is waiting for diff context".to_string()
            }
            AnalysisPanelState::Ready => "Analysis ready to start".to_string(),
            AnalysisPanelState::Unavailable => {
                if self.analysis_remote_mode() && !self.analysis_remote_config_ready() {
                    "Analysis requires provider configuration".to_string()
                } else {
                    "Analysis unavailable for this selection".to_string()
                }
            }
            AnalysisPanelState::Loading => "Analysis in progress".to_string(),
            AnalysisPanelState::Error => "Analysis failed".to_string(),
            AnalysisPanelState::Success => "Review conclusion ready".to_string(),
        }
    }

    /// Returns state surface body text for Analysis mode.
    pub fn analysis_state_body_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => {
                "Select one row in Results / Navigator to open the file-level Analysis view."
                    .to_string()
            }
            AnalysisPanelState::StaleSelection => {
                "The previously focused file is not part of the current visible Results / Navigator set."
                    .to_string()
            }
            AnalysisPanelState::WaitingForDiff => {
                if self.diff_loading {
                    "Detailed diff or preview is still loading for the selected file.".to_string()
                } else {
                    "Load detailed diff or preview context before requesting analysis.".to_string()
                }
            }
            AnalysisPanelState::Ready => {
                "The selected file already has reviewable diff context. Run Analyze when you want a structured risk review."
                    .to_string()
            }
            AnalysisPanelState::Unavailable => {
                if !self.analysis_hint_text().trim().is_empty() {
                    self.analysis_hint_text()
                } else {
                    "Analysis is unavailable for the current selection.".to_string()
                }
            }
            AnalysisPanelState::Loading => {
                "The provider is reviewing the current diff context and assembling summary, risk, and next-step guidance."
                    .to_string()
            }
            AnalysisPanelState::Error => {
                "A review conclusion could not be generated for the current diff context in this session."
                    .to_string()
            }
            AnalysisPanelState::Success => {
                "Summary, risk level, key points, and review suggestions are ready below."
                    .to_string()
            }
        }
    }

    /// Returns optional secondary note text for Analysis state surface.
    pub fn analysis_state_note_text(&self) -> String {
        match self.analysis_panel_state() {
            AnalysisPanelState::NoSelection => {
                "Analysis only runs for changed text files with loadable diff context.".to_string()
            }
            AnalysisPanelState::StaleSelection => {
                if self.running {
                    "Compare is still running. The previous path will be rechecked when results arrive."
                        .to_string()
                } else {
                    "Adjust Search/Status filters or select a visible row to continue.".to_string()
                }
            }
            AnalysisPanelState::WaitingForDiff => self.analysis_technical_context_text(),
            AnalysisPanelState::Ready => self.analysis_technical_context_text(),
            AnalysisPanelState::Unavailable => {
                if self.analysis_remote_mode() && !self.analysis_remote_config_ready() {
                    "Complete endpoint, API key, and model in Settings -> Provider before using the remote provider."
                        .to_string()
                } else {
                    self.analysis_technical_context_text()
                }
            }
            AnalysisPanelState::Loading => self.analysis_technical_context_text(),
            AnalysisPanelState::Error => self
                .analysis_error_message
                .as_ref()
                .map(|value| abbreviate_middle(value.trim(), 220, 168, 40))
                .unwrap_or_else(|| self.analysis_technical_context_text()),
            AnalysisPanelState::Success => self.analysis_result_notes_text(),
        }
    }

    /// Returns AI analysis hint text for UI rendering.
    pub fn analysis_hint_text(&self) -> String {
        self.analysis_hint.clone().unwrap_or_default()
    }

    /// Returns AI analysis title text for UI rendering.
    pub fn analysis_title_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.title.clone())
            .unwrap_or_default()
    }

    /// Returns AI risk level text for UI rendering.
    pub fn analysis_risk_level_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.risk_level.clone())
            .unwrap_or_default()
    }

    /// Returns AI rationale text for UI rendering.
    pub fn analysis_rationale_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.rationale.clone())
            .unwrap_or_default()
    }

    /// Returns AI key points text for UI rendering.
    pub fn analysis_key_points_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.key_points_text())
            .unwrap_or_default()
    }

    /// Returns AI review suggestions text for UI rendering.
    pub fn analysis_review_suggestions_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| result.review_suggestions_text())
            .unwrap_or_default()
    }

    /// Returns summary excerpt for Analysis success content.
    pub fn analysis_summary_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| {
                let title = result.title.trim();
                let rationale = result.rationale.trim();
                if !rationale.is_empty() {
                    sentence_excerpt(rationale, 168)
                } else if !title.is_empty() {
                    title.to_string()
                } else {
                    "Review summary unavailable.".to_string()
                }
            })
            .unwrap_or_default()
    }

    /// Returns primary assessment text for Analysis success content.
    pub fn analysis_core_judgment_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| {
                let rationale = result.rationale.trim();
                if !rationale.is_empty() {
                    rationale.to_string()
                } else {
                    "No core judgment was returned for this analysis.".to_string()
                }
            })
            .unwrap_or_default()
    }

    /// Returns risk badge label for Analysis success content.
    pub fn analysis_risk_label_text(&self) -> String {
        self.analysis_result
            .as_ref()
            .map(|result| format!("{} risk", title_case_token(&result.risk_level)))
            .unwrap_or_default()
    }

    /// Returns risk badge tone for Analysis success content.
    pub fn analysis_risk_tone(&self) -> String {
        match self.analysis_risk_level_text().as_str() {
            "high" => "error".to_string(),
            "medium" => "warn".to_string(),
            "low" => "success".to_string(),
            _ => "neutral".to_string(),
        }
    }

    /// Returns short risk guidance text for Analysis success content.
    pub fn analysis_risk_guidance_text(&self) -> String {
        match self.analysis_risk_level_text().as_str() {
            "high" => "Prioritize a careful review before merging.".to_string(),
            "medium" => "Review the changed logic paths and edge cases closely.".to_string(),
            "low" => {
                "No immediate high-risk signal surfaced from the current diff context.".to_string()
            }
            _ => "Risk signal unavailable for this analysis.".to_string(),
        }
    }

    /// Returns notes/annotations block for Analysis success content.
    pub fn analysis_result_notes_text(&self) -> String {
        let mut notes = Vec::new();
        if self.diff_truncated {
            notes.push("The analysis was generated from truncated diff context.".to_string());
        }
        if let Some(warning) = self
            .diff_warning
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            notes.push(warning.to_string());
        }
        if notes.is_empty() && !self.analysis_remote_mode() && self.analysis_result.is_some() {
            notes.push(
                "This result came from the deterministic mock provider for local review."
                    .to_string(),
            );
        }
        notes.join("\n")
    }

    /// Returns copy-ready text for the Summary section.
    pub fn analysis_summary_copy_text(&self) -> String {
        compose_section_copy_text(
            "Summary",
            normalize_optional_text(&self.analysis_title_text()),
            normalize_optional_text(&self.analysis_summary_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Risk Level section.
    pub fn analysis_risk_copy_text(&self) -> String {
        compose_section_copy_text(
            "Risk Level",
            normalize_optional_text(&self.analysis_risk_label_text()),
            normalize_optional_text(&self.analysis_risk_guidance_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Core Judgment section.
    pub fn analysis_core_judgment_copy_text(&self) -> String {
        compose_section_copy_text(
            "Core Judgment",
            None,
            normalize_optional_text(&self.analysis_core_judgment_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Key Points section.
    pub fn analysis_key_points_copy_text(&self) -> String {
        compose_section_copy_text(
            "Key Points",
            None,
            normalize_optional_text(&self.analysis_key_points_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Review Suggestions section.
    pub fn analysis_review_suggestions_copy_text(&self) -> String {
        compose_section_copy_text(
            "Review Suggestions",
            None,
            normalize_optional_text(&self.analysis_review_suggestions_text()),
        )
        .unwrap_or_default()
    }

    /// Returns copy-ready text for the Notes section.
    pub fn analysis_notes_copy_text(&self) -> String {
        compose_section_copy_text(
            "Notes",
            None,
            normalize_optional_text(&self.analysis_result_notes_text()),
        )
        .unwrap_or_default()
    }

    /// Returns one copy-ready export for the current Analysis conclusion.
    pub fn analysis_full_copy_text(&self) -> String {
        let mut blocks = Vec::new();
        if let Some(path) = self
            .selected_relative_path
            .as_deref()
            .and_then(normalize_optional_text)
        {
            blocks.push(format!("File\n{path}"));
        }

        for section in [
            self.analysis_summary_copy_text(),
            self.analysis_risk_copy_text(),
            self.analysis_core_judgment_copy_text(),
            self.analysis_key_points_copy_text(),
            self.analysis_review_suggestions_copy_text(),
            self.analysis_notes_copy_text(),
        ] {
            if !section.trim().is_empty() {
                blocks.push(section);
            }
        }

        blocks.join("\n\n")
    }

    /// Returns human-readable AI provider mode.
    pub fn analysis_provider_mode_text(&self) -> String {
        match self.analysis_provider_kind {
            AiProviderKind::Mock => "Mock".to_string(),
            AiProviderKind::OpenAiCompatible => "OpenAI-compatible".to_string(),
        }
    }

    /// Returns true when remote provider mode is selected.
    pub fn analysis_remote_mode(&self) -> bool {
        self.analysis_provider_kind == AiProviderKind::OpenAiCompatible
    }

    /// Returns true when remote provider required config is complete.
    pub fn analysis_remote_config_ready(&self) -> bool {
        !self.analysis_openai_endpoint.trim().is_empty()
            && !self.analysis_openai_api_key.trim().is_empty()
            && !self.analysis_openai_model.trim().is_empty()
    }

    /// Returns request timeout text for UI rendering.
    pub fn analysis_timeout_text(&self) -> String {
        self.analysis_request_timeout_secs.to_string()
    }

    /// Returns settings error text for UI rendering.
    pub fn settings_error_text(&self) -> String {
        self.settings_error_message.clone().unwrap_or_default()
    }

    /// Builds one AI config snapshot from current UI state.
    pub fn analysis_ai_config(&self) -> AiConfig {
        let mut config = AiConfig::default();
        config.provider_kind = self.analysis_provider_kind;
        config.openai_endpoint = normalize_optional_text(&self.analysis_openai_endpoint);
        config.openai_api_key = normalize_optional_text(&self.analysis_openai_api_key);
        config.openai_model = normalize_optional_text(&self.analysis_openai_model);
        config.request_timeout_secs = self.analysis_request_timeout_secs.max(1);
        config
    }

    fn analysis_can_start_now(&self) -> bool {
        self.selected_row.is_some()
            && self.analysis_available
            && !self.diff_loading
            && self.selected_diff.is_some()
            && (!self.analysis_remote_mode() || self.analysis_remote_config_ready())
    }

    fn compare_view_target_status(&self) -> Option<(&'static str, &'static str)> {
        let foundation = self.compare_foundation_for_projections();
        let node = foundation.as_ref().node(
            self.compare_focus_path
                .as_relative_path()
                .unwrap_or_default(),
        )?;
        Some(compare_relation_display(node))
    }

    fn selected_entry_compare_node(&self) -> Option<CompareFoundationNode> {
        let path = self.selected_relative_path.as_deref()?.trim();
        if path.is_empty() {
            return None;
        }
        self.compare_foundation_for_projections()
            .as_ref()
            .node(path)
            .cloned()
    }

    fn selected_entry_compare_status(&self) -> Option<(&'static str, &'static str)> {
        let node = self.selected_entry_compare_node()?;
        Some(compare_file_view_status(&node))
    }

    fn compare_view_breadcrumb_segments(&self) -> Vec<(String, String)> {
        let foundation = self.compare_foundation_for_projections();
        let mut segments = vec![(COMPARE_ROOT_BREADCRUMB_LABEL.to_string(), String::new())];
        let raw = self.compare_focus_path.raw_text();
        if raw.is_empty() {
            return segments;
        }

        let mut current = String::new();
        for component in raw
            .split('/')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(component);
            let label = foundation
                .as_ref()
                .node(current.as_str())
                .map(|node| node.display_name.clone())
                .unwrap_or_else(|| component.to_string());
            segments.push((label, current.clone()));
        }
        segments
    }

    fn resolve_compare_row_focus_path(&self, preferred: Option<&str>) -> Option<String> {
        let rows = self.compare_view_row_projections();
        if rows.is_empty() {
            return None;
        }

        let preferred = preferred
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        if let Some(path) = preferred {
            if rows.iter().any(|row| row.relative_path == path) {
                return Some(path);
            }
        }

        if let Some(current) = self
            .compare_row_focus_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if rows.iter().any(|row| row.relative_path == current) {
                return Some(current.to_string());
            }
        }

        rows.first().map(|row| row.relative_path.clone())
    }

    fn reconcile_compare_row_focus(&mut self, preferred: Option<&str>) -> bool {
        let next = self.resolve_compare_row_focus_path(preferred);
        if self.compare_row_focus_path == next {
            return false;
        }
        self.compare_row_focus_path = next;
        true
    }

    fn reconcile_compare_focus_visibility(&mut self) -> bool {
        if self.show_hidden_files {
            return false;
        }

        let foundation = self.compare_foundation_for_projections();
        let mut next_focus = foundation.clamp_compare_focus_path(&self.compare_focus_path);
        while next_focus
            .as_relative_path()
            .is_some_and(is_hidden_relative_path)
        {
            next_focus = foundation.parent_compare_focus_path(&next_focus);
        }

        if self.compare_focus_path == next_focus {
            return false;
        }
        self.compare_focus_path = next_focus;
        true
    }

    fn bump_compare_view_projection_revision(&mut self) {
        self.compare_view_projection_revision =
            self.compare_view_projection_revision.wrapping_add(1);
    }

    fn bump_navigator_flat_projection_revision(&mut self) {
        self.navigator_flat_projection_revision =
            self.navigator_flat_projection_revision.wrapping_add(1);
    }

    fn bump_navigator_tree_projection_revision(&mut self) {
        self.navigator_tree_projection_revision =
            self.navigator_tree_projection_revision.wrapping_add(1);
    }

    fn bump_navigator_projection_revisions(&mut self) {
        self.bump_navigator_flat_projection_revision();
        self.bump_navigator_tree_projection_revision();
    }

    fn navigator_tree_projection(&self) -> NavigatorTreeProjection {
        let foundation = self.compare_foundation_for_projections();
        let status_filter = normalize_status_filter_token(&self.entry_status_filter);
        project_navigator_tree_rows(
            foundation.as_ref(),
            self.show_hidden_files,
            status_filter.as_str(),
            &self.navigator_tree_expansion_overrides,
        )
    }

    #[cfg(test)]
    fn compare_foundation_for_projections(&self) -> Cow<'_, CompareFoundation> {
        if self.compare_foundation.source_entry_count() == self.entry_rows.len() {
            Cow::Borrowed(&self.compare_foundation)
        } else {
            Cow::Owned(crate::compare_foundation::foundation_from_legacy_rows(
                &self.entry_rows,
            ))
        }
    }

    #[cfg(not(test))]
    fn compare_foundation_for_projections(&self) -> Cow<'_, CompareFoundation> {
        Cow::Borrowed(&self.compare_foundation)
    }

    fn source_node_matches_filter_controls(
        &self,
        node: &CompareFoundationNode,
        status_filter: &str,
    ) -> bool {
        let needle = normalize_filter_needle(&self.entry_filter);
        (needle.is_empty() || node.relative_path.to_lowercase().contains(needle.as_str()))
            && status_filter_matches(node.base_status.as_str(), status_filter)
    }

    fn source_node_visible_in_results(&self, node: &CompareFoundationNode) -> bool {
        let status_filter = normalize_status_filter_token(&self.entry_status_filter);
        self.source_node_matches_filter_controls(node, status_filter.as_str())
            && (self.show_hidden_files || !is_hidden_relative_path(&node.relative_path))
    }

    #[allow(dead_code)]
    fn visible_source_indices(&self) -> Vec<usize> {
        self.compare_foundation_for_projections()
            .as_ref()
            .source_nodes()
            .filter(|node| self.source_node_visible_in_results(node))
            .filter_map(|node| node.source_index)
            .collect()
    }

    pub fn set_compare_foundation(&mut self, foundation: CompareFoundation) {
        let previous_focus = self.compare_row_focus_path.clone();
        self.compare_foundation = foundation;
        self.compare_focus_path = self
            .compare_foundation
            .clamp_compare_focus_path(&self.compare_focus_path);
        self.reconcile_compare_focus_visibility();
        self.prune_compare_view_expansion_overrides();
        if let Some(path) = previous_focus.as_deref() {
            self.reveal_compare_view_path(path);
        }
        self.reconcile_compare_row_focus(previous_focus.as_deref());
        self.bump_compare_view_projection_revision();
        self.sync_compare_tree_session_from_top_level();
    }

    pub fn clear_compare_foundation(&mut self) {
        self.compare_foundation = CompareFoundation::default();
        self.compare_focus_path = CompareFocusPath::root();
        self.compare_row_focus_path = None;
        self.compare_view_expansion_overrides.clear();
        self.compare_view_quick_locate_query.clear();
        self.bump_compare_view_projection_revision();
        self.sync_compare_tree_session_from_top_level();
    }

    fn compare_mode_token(&self) -> Option<String> {
        summary_metric(&self.summary_text, "mode=")
            .and_then(|value| normalize_optional_text(&value))
    }

    fn compare_mode_label(&self) -> Option<String> {
        match self.compare_mode_token()?.as_str() {
            "summary-first" => Some("Summary-first mode".to_string()),
            "large" => Some("Large mode".to_string()),
            "normal" => None,
            other => Some(title_case_token(other)),
        }
    }

    fn compare_mode_note_text(&self) -> Option<String> {
        match self.compare_mode_token()?.as_str() {
            "summary-first" => Some("Summary-first mode".to_string()),
            "large" => Some("Large-directory protection".to_string()),
            "normal" => None,
            other => Some(title_case_token(other)),
        }
    }

    fn compare_deferred_count(&self) -> Option<usize> {
        summary_metric_usize(&self.summary_text, "deferred=")
    }

    fn compare_oversized_count(&self) -> Option<usize> {
        summary_metric_usize(&self.summary_text, "oversized_text=")
    }
}

fn compare_relation_display(node: &CompareFoundationNode) -> (&'static str, &'static str) {
    if matches!(node.detail, CompareFoundationDetail::TypeMismatch { .. }) {
        return ("Mismatch", "warn");
    }

    match node.base_status {
        CompareBaseStatus::LeftOnly => ("Left", "left"),
        CompareBaseStatus::RightOnly => ("Right", "right"),
        CompareBaseStatus::Equal => ("Equal", "equal"),
        CompareBaseStatus::Different | CompareBaseStatus::Pending | CompareBaseStatus::Skipped => {
            ("Diff", "different")
        }
    }
}

fn compare_file_view_status(node: &CompareFoundationNode) -> (&'static str, &'static str) {
    if matches!(node.detail, CompareFoundationDetail::TypeMismatch { .. }) {
        return ("Type mismatch", "warn");
    }

    match node.base_status {
        CompareBaseStatus::LeftOnly => ("Deleted", "left"),
        CompareBaseStatus::RightOnly => ("Added", "right"),
        CompareBaseStatus::Equal => ("Identical", "equal"),
        CompareBaseStatus::Different | CompareBaseStatus::Pending | CompareBaseStatus::Skipped => {
            ("Modified", "different")
        }
    }
}

fn compare_view_side_kinds(node: &CompareFoundationNode) -> (CompareNodeKind, CompareNodeKind) {
    match node.detail {
        CompareFoundationDetail::TypeMismatch { left, right } => (left, right),
        _ => (node.kind, node.kind),
    }
}

fn compare_view_icon_token(
    kind: CompareNodeKind,
    detail: &CompareFoundationDetail,
) -> &'static str {
    match kind {
        CompareNodeKind::Root | CompareNodeKind::Directory => "DIR",
        CompareNodeKind::File => {
            if matches!(detail, CompareFoundationDetail::FileComparison { .. }) {
                "BIN"
            } else {
                "TXT"
            }
        }
        CompareNodeKind::Symlink | CompareNodeKind::Other => "BIN",
    }
}

fn wrap_ui_text(text: &str, max_columns: usize) -> Vec<String> {
    if text.trim().is_empty() || max_columns == 0 {
        return vec![text.to_string()];
    }

    let mut remaining = text.trim().to_string();
    let mut out = Vec::new();
    while remaining.chars().count() > max_columns {
        let mut split_byte = None;
        let mut chars_seen = 0usize;
        for (idx, ch) in remaining.char_indices() {
            chars_seen += 1;
            if chars_seen > max_columns {
                break;
            }
            if ch.is_whitespace() || ch == '/' || ch == '\\' || ch == ',' || ch == ';' {
                split_byte = Some(idx + ch.len_utf8());
            }
        }
        let split_at = split_byte.unwrap_or_else(|| {
            remaining
                .char_indices()
                .nth(max_columns)
                .map(|(idx, _)| idx)
                .unwrap_or(remaining.len())
        });
        let (head, tail) = remaining.split_at(split_at);
        out.push(head.trim_end().to_string());
        remaining = tail.trim_start().to_string();
    }
    if !remaining.is_empty() {
        out.push(remaining);
    }
    out
}

fn abbreviate_middle(text: &str, max_chars: usize, head_chars: usize, tail_chars: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars || chars.len() <= head_chars + tail_chars + 1 {
        return text.to_string();
    }
    let head = chars[..head_chars].iter().collect::<String>();
    let tail = chars[chars.len() - tail_chars..].iter().collect::<String>();
    format!("{head}…{tail}")
}

fn normalize_optional_text(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn compose_section_copy_text(
    section_label: &str,
    title: Option<String>,
    body: Option<String>,
) -> Option<String> {
    let mut lines = vec![section_label.to_string()];
    if let Some(value) = title.and_then(|value| normalize_optional_text(&value)) {
        lines.push(value);
    }
    if let Some(value) = body.and_then(|value| normalize_optional_text(&value)) {
        lines.push(value);
    }
    (lines.len() > 1).then(|| lines.join("\n"))
}

fn sentence_excerpt(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut boundary = None;
    for (idx, ch) in trimmed.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            boundary = Some(idx + ch.len_utf8());
            break;
        }
    }

    let excerpt = boundary
        .map(|idx| trimmed[..idx].trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(trimmed);

    if excerpt.chars().count() <= max_chars {
        excerpt.to_string()
    } else {
        abbreviate_middle(excerpt, max_chars, max_chars.saturating_sub(20), 18)
    }
}

fn title_case_token(token: &str) -> String {
    let mut chars = token.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn normalize_status_filter_token(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "all" => "all".to_string(),
        "different" => "different".to_string(),
        "equal" => "equal".to_string(),
        "left-only" => "left-only".to_string(),
        "right-only" => "right-only".to_string(),
        _ => "all".to_string(),
    }
}

fn normalize_filter_needle(raw: &str) -> String {
    raw.trim().to_lowercase()
}

fn compare_tree_locate_matches(
    foundation: &CompareFoundation,
    relative_path: &str,
    needle: &str,
) -> bool {
    let Some(node) = foundation.node(relative_path) else {
        return false;
    };
    normalize_filter_needle(node.relative_path.as_str()).contains(needle)
        || normalize_filter_needle(node.display_name.as_str()).contains(needle)
}

fn status_filter_matches(status: &str, filter: &str) -> bool {
    filter == "all" || status.eq_ignore_ascii_case(filter)
}

fn status_filter_label(filter: &str) -> &'static str {
    match filter {
        "different" => "Diff",
        "equal" => "Equal",
        "left-only" => "Left only",
        "right-only" => "Right only",
        _ => "All results",
    }
}

fn sanitize_inline_query(query: &str) -> String {
    query
        .trim()
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('"', "'")
}

fn file_session_id(relative_path: &str) -> String {
    format!("file:{}", relative_path.trim())
}

fn split_relative_path_leaf(relative_path: &str) -> (String, String) {
    let trimmed = relative_path
        .trim_matches(|ch| ch == '/' || ch == '\\')
        .trim();
    if trimmed.is_empty() {
        return (String::new(), "compare root".to_string());
    }

    match trimmed
        .char_indices()
        .rev()
        .find(|(_, ch)| *ch == '/' || *ch == '\\')
    {
        Some((idx, _)) => (trimmed[..idx].to_string(), trimmed[idx + 1..].to_string()),
        None => (String::new(), trimmed.to_string()),
    }
}

fn is_hidden_relative_path(relative_path: &str) -> bool {
    relative_path
        .trim_matches(|ch| ch == '/' || ch == '\\')
        .split(['/', '\\'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .any(|part| part.starts_with('.'))
}

fn normalize_navigator_parent_path(parent_path: &str) -> String {
    parent_path
        .trim_matches(|ch| ch == '/' || ch == '\\')
        .replace('\\', "/")
}

fn format_navigator_parent_path(parent_path: &str) -> String {
    let normalized = normalize_navigator_parent_path(parent_path);
    if normalized.is_empty() {
        String::new()
    } else {
        abbreviate_middle(
            &normalized,
            NAVIGATOR_PARENT_PATH_MAX_CHARS,
            NAVIGATOR_PARENT_PATH_HEAD_CHARS,
            NAVIGATOR_PARENT_PATH_TAIL_CHARS,
        )
    }
}

fn navigator_row_tooltip_text(display_name: &str, full_parent_path: &str) -> String {
    if full_parent_path.is_empty() {
        display_name.to_string()
    } else {
        format!("{display_name}\n{full_parent_path}")
    }
}

fn navigator_secondary_text(node: &CompareFoundationNode) -> String {
    let detail_text = node.detail.legacy_text(node.kind);
    let diff_blocked_reason = node.capabilities.diff_blocked_reason_text();
    let secondary = match node.base_status {
        CompareBaseStatus::LeftOnly => match node.kind {
            CompareNodeKind::Directory => "Directory only on left".to_string(),
            CompareNodeKind::Symlink => "Symlink only on left".to_string(),
            CompareNodeKind::Other => "Special entry only on left".to_string(),
            _ => navigator_single_side_file_secondary_text("left", &node.relative_path),
        },
        CompareBaseStatus::RightOnly => match node.kind {
            CompareNodeKind::Directory => "Directory only on right".to_string(),
            CompareNodeKind::Symlink => "Symlink only on right".to_string(),
            CompareNodeKind::Other => "Special entry only on right".to_string(),
            _ => navigator_single_side_file_secondary_text("right", &node.relative_path),
        },
        CompareBaseStatus::Equal => navigator_equal_secondary_text(node),
        CompareBaseStatus::Different => navigator_different_secondary_text(node),
        CompareBaseStatus::Pending => navigator_pending_secondary_text(node),
        CompareBaseStatus::Skipped => sentence_excerpt(
            diff_blocked_reason
                .as_deref()
                .unwrap_or(detail_text.as_str()),
            NAVIGATOR_SECONDARY_MAX_CHARS,
        ),
    };

    if secondary.trim().is_empty() {
        "Compare detail unavailable".to_string()
    } else {
        secondary
    }
}

fn navigator_equal_secondary_text(node: &CompareFoundationNode) -> String {
    match (node.kind, &node.detail) {
        (CompareNodeKind::Directory, _) => "Directory on both sides".to_string(),
        (CompareNodeKind::Symlink, _) => "Symlink compare deferred".to_string(),
        (CompareNodeKind::File, CompareFoundationDetail::TextDiffSummary { .. }) => {
            "Text matched".to_string()
        }
        (CompareNodeKind::File, CompareFoundationDetail::FileComparison { .. }) => {
            let base = navigator_file_comparison_matched_text(&node.relative_path);
            format!(
                "{base}{}",
                navigator_file_compare_sizes_suffix(&node.detail)
            )
        }
        (CompareNodeKind::File, CompareFoundationDetail::TextDetailDeferred { .. }) => {
            format!(
                "Deferred text detail{}",
                navigator_text_detail_reason_suffix(&node.detail)
            )
        }
        (CompareNodeKind::File, CompareFoundationDetail::Message { .. }) => {
            "File matched".to_string()
        }
        _ => format!("{} matched", navigator_entry_kind_label(node.kind)),
    }
}

fn navigator_different_secondary_text(node: &CompareFoundationNode) -> String {
    let detail_text = node.detail.legacy_text(node.kind);
    match &node.detail {
        CompareFoundationDetail::TextDiffSummary { .. } => {
            if let Some(summary) = navigator_text_diff_summary(&node.detail) {
                format!("Text diff · {summary}")
            } else {
                "Text diff".to_string()
            }
        }
        CompareFoundationDetail::TypeMismatch { .. } => format!(
            "Type mismatch{}",
            navigator_type_mismatch_suffix(&node.detail)
        ),
        CompareFoundationDetail::FileComparison { .. } => {
            let base = navigator_file_comparison_differs_text(&node.relative_path);
            format!(
                "{base}{}",
                navigator_file_compare_sizes_suffix(&node.detail)
            )
        }
        CompareFoundationDetail::TextDetailDeferred { .. } => format!(
            "Deferred text diff{}",
            navigator_text_detail_reason_suffix(&node.detail)
        ),
        CompareFoundationDetail::Message { .. } => {
            sentence_excerpt(&detail_text, NAVIGATOR_SECONDARY_MAX_CHARS)
        }
        _ => sentence_excerpt(
            node.capabilities
                .diff_blocked_reason_text()
                .as_deref()
                .unwrap_or(detail_text.as_str()),
            NAVIGATOR_SECONDARY_MAX_CHARS,
        ),
    }
}

fn navigator_pending_secondary_text(node: &CompareFoundationNode) -> String {
    match node.kind {
        CompareNodeKind::Symlink => "Symlink deferred".to_string(),
        CompareNodeKind::Directory => "Directory deferred".to_string(),
        _ => sentence_excerpt(
            &node.detail.legacy_text(node.kind),
            NAVIGATOR_SECONDARY_MAX_CHARS,
        ),
    }
}

fn navigator_single_side_file_secondary_text(_side: &str, relative_path: &str) -> String {
    if let Some(kind) = navigator_capability_file_kind(relative_path) {
        return format!("{kind} · no text preview");
    }

    "Text-only preview".to_string()
}

fn navigator_file_comparison_matched_text(relative_path: &str) -> String {
    if let Some(kind) = navigator_capability_file_kind(relative_path) {
        return format!("{kind} · no text preview");
    }

    "Text-only preview".to_string()
}

fn navigator_file_comparison_differs_text(relative_path: &str) -> String {
    if let Some(kind) = navigator_capability_file_kind(relative_path) {
        return format!("{kind} · no text diff");
    }

    "No text diff".to_string()
}

fn navigator_capability_file_kind(relative_path: &str) -> Option<&'static str> {
    let extension = Path::new(relative_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_ascii_lowercase());
    match extension.as_deref() {
        Some(
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "ico" | "tif" | "tiff" | "avif"
            | "heic" | "heif" | "icns" | "psd",
        ) => Some("Image"),
        Some(
            "pdf" | "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "jar" | "war" | "ear"
            | "bin" | "dat" | "db" | "sqlite" | "sqlite3" | "mp3" | "wav" | "flac" | "ogg" | "m4a"
            | "mp4" | "mov" | "avi" | "mkv" | "exe" | "dll" | "so" | "dylib" | "class" | "wasm"
            | "ttf" | "otf" | "woff" | "woff2" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx",
        ) => Some("Binary"),
        _ => None,
    }
}

fn navigator_entry_kind_label(kind: CompareNodeKind) -> &'static str {
    kind.display_label()
}

fn navigator_text_diff_summary(detail: &CompareFoundationDetail) -> Option<String> {
    let CompareFoundationDetail::TextDiffSummary {
        hunk_count,
        added_lines,
        removed_lines,
        context_lines,
    } = detail
    else {
        return None;
    };

    let hunks = (*hunk_count > 0).then(|| format!("{hunk_count}h"));
    let added = (*added_lines > 0).then(|| format!("+{added_lines}"));
    let removed = (*removed_lines > 0).then(|| format!("-{removed_lines}"));
    let context = (*context_lines > 0).then(|| format!("{context_lines}ctx"));
    let parts = [hunks, added, removed, context]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn navigator_file_compare_sizes_suffix(detail: &CompareFoundationDetail) -> String {
    match detail {
        CompareFoundationDetail::FileComparison {
            left_size,
            right_size,
            ..
        }
        | CompareFoundationDetail::TextDetailDeferred {
            left_size,
            right_size,
            ..
        } => format!(" · {left_size}B / {right_size}B"),
        _ => String::new(),
    }
}

fn navigator_text_detail_reason_suffix(detail: &CompareFoundationDetail) -> String {
    match detail {
        CompareFoundationDetail::TextDetailDeferred { reason, .. } => {
            format!(" · {}", reason.label())
        }
        _ => String::new(),
    }
}

fn navigator_type_mismatch_suffix(detail: &CompareFoundationDetail) -> String {
    match detail {
        CompareFoundationDetail::TypeMismatch { left, right } => {
            format!(" · {} vs {}", left.as_str(), right.as_str())
        }
        _ => String::new(),
    }
}

fn format_warning_count(count: usize) -> String {
    match count {
        0 => String::new(),
        1 => "1 warning".to_string(),
        value => format!("{value} warnings"),
    }
}

fn summary_metric(summary_text: &str, key: &str) -> Option<String> {
    summary_text
        .split_whitespace()
        .find_map(|part| part.trim_matches('|').strip_prefix(key))
        .map(|value| value.trim_matches('|').to_string())
}

fn summary_metric_usize(summary_text: &str, key: &str) -> Option<usize> {
    summary_metric(summary_text, key).and_then(|value| value.parse::<usize>().ok())
}

/// One flattened row displayed in the unified diff viewer list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffViewerRow {
    /// Old-side line number text.
    pub old_line_no: String,
    /// New-side line number text.
    pub new_line_no: String,
    /// Unified diff marker (`+`, `-`, ` `, `@@`).
    pub marker: String,
    /// Row content text.
    pub content: String,
    /// Row style kind (`hunk`, `added`, `removed`, `context`).
    pub row_kind: String,
}

#[cfg(test)]
#[path = "tests/state_foundation_tests.rs"]
mod foundation_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rows() -> Vec<CompareEntryRowViewModel> {
        vec![
            CompareEntryRowViewModel {
                relative_path: "src/main.rs".to_string(),
                status: "different".to_string(),
                detail: "text summary".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "text-diff".to_string(),
                can_load_diff: true,
                diff_blocked_reason: None,
                can_load_analysis: true,
                analysis_blocked_reason: None,
            },
            CompareEntryRowViewModel {
                relative_path: "assets/logo.png".to_string(),
                status: "different".to_string(),
                detail: "file compare: left=10B right=12B".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "file-comparison".to_string(),
                can_load_diff: false,
                diff_blocked_reason: Some("binary candidate".to_string()),
                can_load_analysis: false,
                analysis_blocked_reason: Some("binary candidate".to_string()),
            },
        ]
    }

    fn sample_preview_panel(summary: &str, content: &str) -> DiffPanelViewModel {
        DiffPanelViewModel {
            relative_path: "assets/preview.js".to_string(),
            summary_text: summary.to_string(),
            hunks: vec![crate::view_models::DiffHunkViewModel {
                old_start: 1,
                old_len: 1,
                new_start: 1,
                new_len: 0,
                lines: vec![crate::view_models::DiffLineViewModel {
                    old_line_no: Some(1),
                    new_line_no: None,
                    kind: "Context".to_string(),
                    content: content.to_string(),
                }],
            }],
            warning: None,
            truncated: false,
        }
    }

    #[test]
    fn invalid_status_filter_falls_back_to_all() {
        let mut state = AppState::default();
        state.set_entry_status_filter("unexpected-status");
        assert_eq!(state.entry_status_filter, "all");
    }

    #[test]
    fn navigator_effective_view_mode_forces_flat_while_search_is_active() {
        let mut state = AppState::default();
        assert_eq!(
            state.effective_navigator_view_mode(),
            NavigatorViewMode::Tree
        );

        state.entry_filter = "main.rs".to_string();
        assert!(state.navigator_search_forces_flat_mode());
        assert_eq!(
            state.effective_navigator_view_mode(),
            NavigatorViewMode::Flat
        );

        state.entry_filter.clear();
        assert_eq!(
            state.effective_navigator_view_mode(),
            NavigatorViewMode::Tree
        );

        state.set_navigator_runtime_view_mode(NavigatorViewMode::Flat);
        assert_eq!(
            state.effective_navigator_view_mode(),
            NavigatorViewMode::Flat
        );
    }

    #[test]
    fn compact_summary_text_extracts_key_metrics() {
        let state = AppState {
            summary_text: "mode=normal total=120 equal=100 different=8 left_only=7 right_only=5 pending=0 skipped=0 deferred=3 oversized_text=2".to_string(),
            truncated: true,
            ..AppState::default()
        };
        let text = state.compact_summary_text();
        assert!(text.contains("Total 120"));
        assert!(text.contains("Changed 8"));
        assert!(text.contains("Left 7"));
        assert!(text.contains("Right 5"));
        assert!(text.contains("3 deferred"));
        assert!(text.contains("2 oversized"));
        assert!(text.contains("Truncated"));
    }

    #[test]
    fn compare_metrics_text_formats_core_counts() {
        let state = AppState {
            summary_text: "mode=normal total=42 equal=35 different=4 left_only=2 right_only=1 pending=0 skipped=0 deferred=0 oversized_text=0".to_string(),
            ..AppState::default()
        };
        assert_eq!(
            state.compare_metrics_text(),
            "Total 42 · Changed 4 · Left 2 · Right 1"
        );
    }

    #[test]
    fn compare_copy_texts_are_summary_first_and_structured() {
        let state = AppState {
            status_text: "Compare finished: 42 entries".to_string(),
            summary_text: "mode=summary-first total=42 equal=35 different=4 left_only=2 right_only=1 pending=0 skipped=0 deferred=3 oversized_text=1".to_string(),
            warning_lines: vec!["large-directory guard applied".to_string()],
            truncated: true,
            ..AppState::default()
        };

        let summary = state.compare_summary_copy_text();
        assert!(summary.contains("Compare Summary"));
        assert!(summary.contains("Compare finished: 42 entries"));
        assert!(summary.contains("Total 42 · Changed 4 · Left 2 · Right 1"));
        assert!(summary.contains(
            "Summary-first mode · 3 deferred · 1 oversized · 1 warning · Truncated output"
        ));

        let detail = state.compare_detail_copy_text();
        assert!(detail.contains("Compare Detail"));
        assert!(detail.contains("Status\nCompare finished: 42 entries"));
        assert!(detail.contains("Results\nTotal 42 · Changed 4 · Left 2 · Right 1"));
        assert!(detail.contains("Detail\nSummary-first mode"));
        assert!(detail.contains("3 deferred detail entries"));
        assert!(detail.contains("Warnings\n• large-directory guard applied"));
    }

    #[test]
    fn compare_flags_reflect_summary_metrics() {
        let state = AppState {
            summary_text: "mode=normal total=6 equal=2 different=1 left_only=1 right_only=2 pending=0 skipped=0 deferred=2 oversized_text=1".to_string(),
            ..AppState::default()
        };
        assert!(state.compare_has_deferred());
        assert!(state.compare_has_oversized());
    }

    #[test]
    fn warnings_text_wraps_long_lines_for_ui() {
        let state = AppState {
            warning_lines: vec![
                "large directory guard: entries=20000 total_bytes=3221225472 hard_entries=50000 hard_total_bytes=2147483648".to_string(),
            ],
            ..AppState::default()
        };
        let text = state.warnings_text();
        assert!(text.contains("• "));
        assert!(text.contains('\n'));
        assert!(text.contains("entries=20000"));
    }

    #[test]
    fn selected_relative_path_is_abbreviated_when_too_long() {
        let long_path = format!("{}/{}", "a".repeat(120), "b".repeat(120));
        let state = AppState {
            selected_relative_path: Some(long_path),
            ..AppState::default()
        };
        let display = state.selected_relative_path_text();
        assert!(display.contains('…'));
        assert!(display.len() < 200);
    }

    #[test]
    fn diff_shell_state_tracks_no_selection_loading_and_ready() {
        let mut state = AppState::default();
        assert_eq!(state.diff_shell_state(), DiffShellState::NoSelection);

        state.selected_relative_path = Some("src/main.rs".to_string());
        assert_eq!(state.diff_shell_state(), DiffShellState::StaleSelection);

        state.selected_row = Some(0);
        state.entry_rows = sample_rows();
        state.diff_loading = true;
        assert_eq!(state.diff_shell_state(), DiffShellState::Loading);

        state.diff_loading = false;
        state.selected_diff = Some(DiffPanelViewModel {
            relative_path: "src/main.rs".to_string(),
            summary_text: "hunks=1 +2 -1 ctx=3".to_string(),
            hunks: vec![crate::view_models::DiffHunkViewModel {
                old_start: 1,
                old_len: 1,
                new_start: 1,
                new_len: 1,
                lines: vec![crate::view_models::DiffLineViewModel {
                    old_line_no: Some(1),
                    new_line_no: Some(1),
                    kind: "Added".to_string(),
                    content: "line".to_string(),
                }],
            }],
            warning: None,
            truncated: false,
        });
        assert_eq!(state.diff_shell_state(), DiffShellState::DetailedReady);
    }

    #[test]
    fn diff_shell_state_marks_preview_and_unavailable() {
        let mut state = AppState {
            selected_row: Some(0),
            entry_rows: vec![CompareEntryRowViewModel {
                relative_path: "assets/p.js".to_string(),
                status: "left-only".to_string(),
                detail: "only on left".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "none".to_string(),
                can_load_diff: true,
                diff_blocked_reason: None,
                can_load_analysis: false,
                analysis_blocked_reason: Some("not changed".to_string()),
            }],
            selected_relative_path: Some("assets/p.js".to_string()),
            ..AppState::default()
        };

        state.selected_diff = Some(sample_preview_panel("left-only preview lines=4", "line"));
        assert_eq!(state.diff_shell_state(), DiffShellState::PreviewReady);
        assert!(state.diff_context_hint_text().contains("source left-only"));

        state.selected_diff = Some(sample_preview_panel(
            "single-side preview unavailable",
            "[preview unavailable] binary content is not supported",
        ));
        assert_eq!(state.diff_shell_state(), DiffShellState::Unavailable);
    }

    #[test]
    fn diff_context_header_fields_use_status_specific_labels() {
        let state = AppState {
            selected_row: Some(0),
            entry_rows: vec![CompareEntryRowViewModel {
                relative_path: "docs/readme.md".to_string(),
                status: "equal".to_string(),
                detail: "equal".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "none".to_string(),
                can_load_diff: true,
                diff_blocked_reason: None,
                can_load_analysis: false,
                analysis_blocked_reason: Some("not changed".to_string()),
            }],
            selected_relative_path: Some("docs/readme.md".to_string()),
            selected_diff: Some(sample_preview_panel("equal preview lines=10", "line")),
            ..AppState::default()
        };

        assert_eq!(state.diff_mode_label(), "Preview");
        assert_eq!(state.diff_result_status_label(), "Equal");
        assert_eq!(state.diff_left_column_label(), "left");
        assert_eq!(state.diff_right_column_label(), "right");
        assert!(state.diff_context_hint_text().contains("type .md"));
    }

    #[test]
    fn clear_analysis_panel_resets_loading_error_and_result() {
        let mut state = AppState {
            analysis_loading: true,
            analysis_error_message: Some("error".to_string()),
            analysis_result: Some(AnalysisResultViewModel {
                title: "title".to_string(),
                risk_level: "low".to_string(),
                rationale: "ok".to_string(),
                key_points: vec!["k".to_string()],
                review_suggestions: vec!["s".to_string()],
            }),
            ..AppState::default()
        };
        state.clear_analysis_panel();
        assert!(!state.analysis_loading);
        assert!(state.analysis_error_message.is_none());
        assert!(state.analysis_result.is_none());
    }

    #[test]
    fn analysis_panel_state_distinguishes_no_selection_waiting_ready_and_success() {
        let mut state = AppState::default();
        assert_eq!(
            state.analysis_panel_state(),
            AnalysisPanelState::NoSelection
        );

        state.selected_row = Some(0);
        state.entry_rows = sample_rows();
        assert_eq!(
            state.analysis_panel_state(),
            AnalysisPanelState::WaitingForDiff
        );

        state.analysis_available = true;
        state.selected_diff = Some(sample_preview_panel("preview", "line"));
        assert_eq!(state.analysis_panel_state(), AnalysisPanelState::Ready);
        assert_eq!(state.analysis_state_title_text(), "Analysis ready to start");

        state.analysis_result = Some(AnalysisResultViewModel {
            title: "Risk review for src/main.rs".to_string(),
            risk_level: "medium".to_string(),
            rationale: "The change touches branching logic and should be reviewed carefully."
                .to_string(),
            key_points: vec!["Branching changed".to_string()],
            review_suggestions: vec!["Add coverage".to_string()],
        });
        assert_eq!(state.analysis_panel_state(), AnalysisPanelState::Success);
    }

    #[test]
    fn analysis_panel_state_marks_stale_and_unavailable_selection() {
        let stale_state = AppState {
            selected_relative_path: Some("assets/logo.jpg".to_string()),
            ..AppState::default()
        };
        assert_eq!(
            stale_state.analysis_panel_state(),
            AnalysisPanelState::StaleSelection
        );

        let unavailable_state = AppState {
            selected_row: Some(0),
            entry_rows: vec![CompareEntryRowViewModel {
                relative_path: "assets/logo.jpg".to_string(),
                status: "left-only".to_string(),
                detail: "only on left".to_string(),
                entry_kind: "file".to_string(),
                detail_kind: "none".to_string(),
                can_load_diff: true,
                diff_blocked_reason: None,
                can_load_analysis: false,
                analysis_blocked_reason: Some(
                    "AI analysis is only available for changed file entries".to_string(),
                ),
            }],
            selected_relative_path: Some("assets/logo.jpg".to_string()),
            selected_diff: Some(sample_preview_panel(
                "single-side preview unavailable",
                "[preview unavailable] binary content is not supported",
            )),
            analysis_hint: Some(
                "AI analysis is only available for changed file entries".to_string(),
            ),
            ..AppState::default()
        };
        assert_eq!(
            unavailable_state.analysis_panel_state(),
            AnalysisPanelState::Unavailable
        );
        assert_eq!(
            unavailable_state.analysis_state_title_text(),
            "Analysis unavailable for this selection"
        );
    }

    #[test]
    fn analysis_result_notes_include_truncation_and_warning() {
        let state = AppState {
            selected_row: Some(0),
            entry_rows: sample_rows(),
            analysis_result: Some(AnalysisResultViewModel {
                title: "Risk review".to_string(),
                risk_level: "high".to_string(),
                rationale: "This change updates error handling. It also introduces unwrap."
                    .to_string(),
                key_points: vec!["unwrap added".to_string()],
                review_suggestions: vec!["Check panic paths".to_string()],
            }),
            diff_truncated: true,
            diff_warning: Some("input excerpt trimmed to fit provider limit".to_string()),
            ..AppState::default()
        };

        assert_eq!(state.analysis_risk_tone(), "error");
        assert!(
            state
                .analysis_summary_text()
                .starts_with("This change updates")
        );
        assert!(
            state
                .analysis_result_notes_text()
                .contains("truncated diff context")
        );
        assert!(
            state
                .analysis_result_notes_text()
                .contains("input excerpt trimmed")
        );
    }

    #[test]
    fn analysis_copy_text_exports_structured_sections() {
        let state = AppState {
            selected_row: Some(0),
            entry_rows: sample_rows(),
            selected_relative_path: Some("src/main.rs".to_string()),
            analysis_result: Some(AnalysisResultViewModel {
                title: "Regression risk in startup path".to_string(),
                risk_level: "high".to_string(),
                rationale: "The patch removes validation and shifts initialization order."
                    .to_string(),
                key_points: vec![
                    "Validation branch deleted".to_string(),
                    "Startup sequencing changed".to_string(),
                ],
                review_suggestions: vec!["Re-run startup coverage".to_string()],
            }),
            diff_warning: Some("context excerpt trimmed".to_string()),
            ..AppState::default()
        };

        assert!(
            state
                .analysis_summary_copy_text()
                .starts_with("Summary\nRegression risk")
        );
        assert!(state.analysis_risk_copy_text().contains("High risk"));
        assert!(
            state
                .analysis_full_copy_text()
                .contains("File\nsrc/main.rs")
        );
        assert!(
            state
                .analysis_full_copy_text()
                .contains("Review Suggestions")
        );
        assert!(
            state
                .analysis_full_copy_text()
                .contains("context excerpt trimmed")
        );
    }

    #[test]
    fn remote_config_ready_requires_endpoint_key_and_model() {
        let mut state = AppState {
            analysis_provider_kind: AiProviderKind::OpenAiCompatible,
            ..AppState::default()
        };
        assert!(!state.analysis_remote_config_ready());

        state.analysis_openai_endpoint = "http://localhost:11434/v1".to_string();
        assert!(!state.analysis_remote_config_ready());
        state.analysis_openai_api_key = "token".to_string();
        assert!(state.analysis_remote_config_ready());
    }

    #[test]
    fn analysis_ai_config_reflects_provider_fields() {
        let state = AppState {
            analysis_provider_kind: AiProviderKind::OpenAiCompatible,
            analysis_openai_endpoint: " http://localhost:11434/v1 ".to_string(),
            analysis_openai_api_key: " sk-test ".to_string(),
            analysis_openai_model: " qwen2.5-coder ".to_string(),
            analysis_request_timeout_secs: 42,
            ..AppState::default()
        };
        let config = state.analysis_ai_config();
        assert_eq!(config.provider_kind, AiProviderKind::OpenAiCompatible);
        assert_eq!(
            config.openai_endpoint.as_deref(),
            Some("http://localhost:11434/v1")
        );
        assert_eq!(config.openai_api_key.as_deref(), Some("sk-test"));
        assert_eq!(config.openai_model.as_deref(), Some("qwen2.5-coder"));
        assert_eq!(config.request_timeout_secs, 42);
    }
}
