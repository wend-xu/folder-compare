//! Presenter layer for compare, filtering, and detailed diff orchestration.

use crate::bridge;
use crate::commands::UiCommand;
use crate::commands::{run_ai_analysis, run_compare, run_text_diff};
use crate::compare_file::{build_preview_compare_file, map_text_diff_result_to_compare_file};
use crate::compare_foundation::CompareFocusPath;
use crate::settings::{
    self, AppPreferences, BehaviorSettings, DefaultResultsView, ProviderSettings,
};
use crate::state::{
    AppState, FileSessionMode, NavigatorViewMode, WorkspaceMode, WorkspaceSessionConfirmationEffect,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const BACKGROUND_START_DELAY: Duration = Duration::from_millis(4);
type StateChangeNotifier = Arc<dyn Fn() + Send + Sync>;

enum DiffLoadPlan {
    Noop,
    SyncOnly,
    StandardBackground {
        left_root: String,
        right_root: String,
        row: crate::view_models::CompareEntryRowViewModel,
        state_ref: Arc<Mutex<AppState>>,
    },
    CompareFileBackground {
        left_root: String,
        right_root: String,
        row: crate::view_models::CompareEntryRowViewModel,
        state_ref: Arc<Mutex<AppState>>,
    },
}

/// Presenter that manages compare-oriented UI state.
#[derive(Clone)]
pub struct Presenter {
    state: Arc<Mutex<AppState>>,
    state_change_notifier: Arc<Mutex<Option<StateChangeNotifier>>>,
}

impl Presenter {
    /// Creates a presenter from state.
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
            state_change_notifier: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns a snapshot copy of current app state.
    pub fn state_snapshot(&self) -> AppState {
        self.state.lock().expect("state mutex poisoned").clone()
    }

    /// Registers one optional notifier used to push async state completions back to the UI thread.
    pub fn set_state_change_notifier(&self, notifier: StateChangeNotifier) {
        let mut slot = self
            .state_change_notifier
            .lock()
            .expect("state change notifier mutex poisoned");
        *slot = Some(notifier);
    }

    /// Handles one UI command.
    pub fn handle_command(&self, command: UiCommand) {
        match command {
            UiCommand::Initialize => {
                {
                    let mut state = self.state.lock().expect("state mutex poisoned");
                    state.status_text = "Ready".to_string();
                }
                // Keep settings I/O outside the state mutex so edition-2024
                // temporary drop-order changes cannot extend the lock lifetime.
                let loaded_settings = settings::load_app_preferences();
                let mut state = self.state.lock().expect("state mutex poisoned");
                match loaded_settings {
                    Ok(Some(saved)) => {
                        let default_navigator_view_mode =
                            navigator_view_mode_from_settings(saved.behavior.default_results_view);
                        state.analysis_provider_kind = saved.provider.provider_kind;
                        state.analysis_openai_endpoint = saved.provider.openai_endpoint;
                        state.analysis_openai_api_key = saved.provider.openai_api_key;
                        state.analysis_openai_model = saved.provider.openai_model;
                        state.analysis_request_timeout_secs = saved.provider.timeout_secs.max(1);
                        state.show_hidden_files = saved.behavior.show_hidden_files;
                        state.set_default_navigator_view_mode(default_navigator_view_mode);
                        state.set_navigator_runtime_view_mode(default_navigator_view_mode);
                        state.settings_error_message = None;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        state.settings_error_message =
                            Some(format!("Failed to load settings: {err}"));
                        state.status_text = "Settings load failed".to_string();
                    }
                }
            }
            UiCommand::UpdateLeftRoot(path) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.left_root = path;
            }
            UiCommand::UpdateRightRoot(path) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.right_root = path;
            }
            UiCommand::RunCompare => self.execute_compare(),
            UiCommand::UpdateEntryFilter(filter) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.set_entry_filter(filter);
                Self::reconcile_selected_row_membership(&mut state);
                state.reconcile_file_sessions_with_active_results();
            }
            UiCommand::UpdateEntryStatusFilter(filter) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.set_entry_status_filter(&filter);
                Self::reconcile_selected_row_membership(&mut state);
                state.reconcile_file_sessions_with_active_results();
            }
            UiCommand::SetNavigatorViewModeTree => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                Self::apply_runtime_navigator_view_mode(&mut state, NavigatorViewMode::Tree);
                state.reconcile_file_sessions_with_active_results();
            }
            UiCommand::SetNavigatorViewModeFlat => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                Self::apply_runtime_navigator_view_mode(&mut state, NavigatorViewMode::Flat);
                state.reconcile_file_sessions_with_active_results();
            }
            UiCommand::ToggleSidebarVisibility => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.toggle_sidebar_visible();
            }
            UiCommand::ToggleNavigatorTreeNode(key) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.toggle_navigator_tree_node(&key);
            }
            UiCommand::SelectRow(index) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                let selected_index = usize::try_from(index)
                    .ok()
                    .filter(|value| *value < state.entry_rows.len());
                Self::select_results_row(&mut state, selected_index);
            }
            UiCommand::LoadSelectedDiff => self.execute_load_selected_diff(),
            UiCommand::LocateAndOpen(relative_path) => {
                self.execute_locate_and_open(relative_path);
            }
            UiCommand::OpenCompareView(relative_path) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.request_compare_session_reset(relative_path.as_str(), None);
            }
            UiCommand::SelectWorkspaceSession(session_id) => {
                let should_load_selected_diff = {
                    let mut state = self.state.lock().expect("state mutex poisoned");
                    if !state.activate_workspace_session(session_id.as_str()) {
                        return;
                    }
                    state.active_file_session_needs_diff_reload()
                };
                if should_load_selected_diff {
                    self.execute_load_selected_diff();
                }
            }
            UiCommand::CloseWorkspaceSession(session_id) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.close_workspace_session(session_id.as_str());
            }
            UiCommand::ConfirmWorkspaceSessionAction => {
                let effect = {
                    let mut state = self.state.lock().expect("state mutex poisoned");
                    state.confirm_workspace_session_action()
                };
                if matches!(effect, WorkspaceSessionConfirmationEffect::LoadSelectedDiff) {
                    self.execute_load_selected_diff();
                }
            }
            UiCommand::CancelWorkspaceSessionAction => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.cancel_workspace_session_action();
            }
            UiCommand::CompareViewUpOneLevel => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                let child_focus = state.compare_focus_path_raw_text();
                if state.focus_compare_parent() {
                    let parent_focus = state.compare_focus_path_raw_text();
                    Self::enter_compare_view(
                        &mut state,
                        parent_focus.as_str(),
                        Some(child_focus.as_str()),
                    );
                }
            }
            UiCommand::NavigateCompareView(relative_path) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                let current_focus = state.compare_focus_path_raw_text();
                let preferred_row_focus =
                    preferred_compare_child_focus(current_focus.as_str(), relative_path.as_str());
                Self::enter_compare_view(
                    &mut state,
                    relative_path.as_str(),
                    preferred_row_focus.as_deref(),
                );
            }
            UiCommand::UpdateCompareViewQuickLocate(query) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.set_compare_view_quick_locate_query(query.as_str());
                if !query.trim().is_empty() {
                    state.locate_next_in_compare_view(false);
                }
            }
            UiCommand::RetreatCompareViewQuickLocate => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.locate_previous_in_compare_view();
            }
            UiCommand::AdvanceCompareViewQuickLocate => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.locate_next_in_compare_view(true);
            }
            UiCommand::ToggleCompareTreeNode(relative_path) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                let toggled = state.toggle_compare_view_node(relative_path.as_str());
                let focused = state.set_compare_row_focus_path(Some(relative_path.as_str()));
                let scrolled = state.request_compare_view_scroll_to_path(relative_path.as_str());
                if !toggled && !focused && !scrolled {
                    return;
                }
            }
            UiCommand::FocusCompareRow(relative_path) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                let changed = state.set_compare_row_focus_path(Some(relative_path.as_str()));
                let scrolled = state.request_compare_view_scroll_to_path(relative_path.as_str());
                if !changed && !scrolled {
                    return;
                }
            }
            UiCommand::ToggleCompareViewScrollLock => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.toggle_compare_view_horizontal_scroll_locked();
            }
            UiCommand::OpenFileViewFromCompare(relative_path) => {
                self.open_file_view_from_compare(relative_path);
            }
            UiCommand::SetFileViewModeDiff => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.set_file_view_mode(FileSessionMode::Diff);
            }
            UiCommand::SetFileViewModeAnalysis => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.set_file_view_mode(FileSessionMode::Analysis);
            }
            UiCommand::LoadAiAnalysis => self.execute_load_ai_analysis(),
            UiCommand::SetAiProviderModeMock => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_provider_kind = fc_ai::AiProviderKind::Mock;
                state.clear_analysis_panel();
                state.analysis_hint =
                    Some("Using mock provider. No remote request will be sent.".to_string());
                state.sync_active_file_session_from_top_level();
            }
            UiCommand::SetAiProviderModeOpenAiCompatible => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_provider_kind = fc_ai::AiProviderKind::OpenAiCompatible;
                state.clear_analysis_panel();
                state.analysis_hint =
                    Some("Using remote provider. Configure endpoint/api key/model.".to_string());
                state.sync_active_file_session_from_top_level();
            }
            UiCommand::UpdateAiEndpoint(value) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_openai_endpoint = value;
                state.analysis_error_message = None;
                state.sync_active_file_session_from_top_level();
            }
            UiCommand::UpdateAiApiKey(value) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_openai_api_key = value;
                state.analysis_error_message = None;
                state.sync_active_file_session_from_top_level();
            }
            UiCommand::UpdateAiModel(value) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_openai_model = value;
                state.analysis_error_message = None;
                state.sync_active_file_session_from_top_level();
            }
            UiCommand::SaveAppSettings {
                provider_kind,
                endpoint,
                api_key,
                model,
                timeout_secs_text,
                show_hidden_files,
                default_results_view,
            } => {
                let timeout_secs = match parse_timeout_secs(&timeout_secs_text) {
                    Ok(value) => value,
                    Err(err) => {
                        let mut state = self.state.lock().expect("state mutex poisoned");
                        state.settings_error_message = Some(err);
                        return;
                    }
                };

                let endpoint = endpoint.trim().to_string();
                let api_key = api_key.trim().to_string();
                let model = model.trim().to_string();
                let settings = {
                    let mut state = self.state.lock().expect("state mutex poisoned");
                    state.analysis_provider_kind = provider_kind;
                    state.analysis_openai_endpoint = endpoint;
                    state.analysis_openai_api_key = api_key;
                    state.analysis_openai_model = model;
                    state.analysis_request_timeout_secs = timeout_secs;
                    state.set_show_hidden_files(show_hidden_files);
                    state.set_default_navigator_view_mode(default_results_view);
                    Self::apply_runtime_navigator_view_mode(&mut state, default_results_view);
                    state.reconcile_file_sessions_with_active_results();
                    state.clear_analysis_panel();
                    state.analysis_hint = Some(match provider_kind {
                        fc_ai::AiProviderKind::Mock => {
                            "Using mock provider. No remote request will be sent.".to_string()
                        }
                        fc_ai::AiProviderKind::OpenAiCompatible => {
                            "Using remote provider. Configure endpoint/api key/model.".to_string()
                        }
                    });
                    state.sync_active_file_session_from_top_level();

                    AppPreferences {
                        provider: ProviderSettings {
                            provider_kind,
                            openai_endpoint: state.analysis_openai_endpoint.clone(),
                            openai_api_key: state.analysis_openai_api_key.clone(),
                            openai_model: state.analysis_openai_model.clone(),
                            timeout_secs: state.analysis_request_timeout_secs,
                        },
                        behavior: BehaviorSettings {
                            show_hidden_files: state.show_hidden_files,
                            default_results_view: settings_default_results_view(
                                state.default_navigator_view_mode,
                            ),
                        },
                    }
                };
                // Keep settings I/O outside the state mutex so edition-2024
                // temporary drop-order changes cannot extend the lock lifetime.
                let save_result = settings::save_app_preferences(&settings);
                let mut state = self.state.lock().expect("state mutex poisoned");
                match save_result {
                    Ok(_) => {
                        state.settings_error_message = None;
                        state.status_text = "Settings saved".to_string();
                    }
                    Err(err) => {
                        state.settings_error_message =
                            Some(format!("Failed to save settings: {err}"));
                        state.status_text = "Settings save failed".to_string();
                    }
                }
            }
            UiCommand::ClearSettingsError => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.settings_error_message = None;
            }
        }
    }

    fn notify_state_changed(&self) {
        Self::notify_state_changed_with(&self.state_change_notifier);
    }

    fn notify_state_changed_with(slot: &Arc<Mutex<Option<StateChangeNotifier>>>) {
        let notifier = slot
            .lock()
            .expect("state change notifier mutex poisoned")
            .clone();
        if let Some(notifier) = notifier {
            notifier();
        }
    }

    fn clear_file_view_state(state: &mut AppState) {
        state.clear_diff_panel();
        state.clear_compare_file_panel();
        state.analysis_available = false;
        state.clear_analysis_panel();
    }

    fn set_analysis_no_selection_hint(state: &mut AppState) {
        state.analysis_hint = Some("Select one changed text file to analyze.".to_string());
    }

    fn set_analysis_stale_selection_hint(state: &mut AppState) {
        state.analysis_hint = Some(
            "Previous selection is no longer active in the current Results / Navigator set."
                .to_string(),
        );
    }

    fn set_analysis_compare_restore_hint(state: &mut AppState) {
        state.analysis_hint =
            Some("Previous selection will be rechecked after compare finishes.".to_string());
    }

    fn apply_row_selection(
        state: &mut AppState,
        selected_index: Option<usize>,
        opened_from_compare_view: bool,
    ) {
        state.selected_row = selected_index;
        let selected_row_vm = state
            .selected_row
            .and_then(|value| state.entry_rows.get(value))
            .cloned();
        if selected_row_vm.is_some() {
            state.set_workspace_mode(WorkspaceMode::FileView);
            state.file_view_mode = FileSessionMode::Diff;
        }
        state.can_return_to_compare_view = selected_row_vm.is_some() && opened_from_compare_view;
        state.selected_relative_path = selected_row_vm
            .as_ref()
            .map(|row| row.relative_path.clone());
        Self::clear_file_view_state(state);
        state.analysis_hint = Some(match selected_row_vm {
            Some(row) if !row.can_load_analysis => row
                .analysis_blocked_reason
                .unwrap_or_else(|| "selected row does not support AI analysis".to_string()),
            Some(_) => "Load detailed diff, then click Analyze.".to_string(),
            None => "Select one changed text file to analyze.".to_string(),
        });
        state.sync_active_file_session_from_top_level();
    }

    fn select_results_row(state: &mut AppState, selected_index: Option<usize>) {
        let selected_row_vm = selected_index
            .and_then(|index| state.entry_rows.get(index))
            .cloned();
        if state.has_compare_tree_session() {
            let Some(row) = selected_row_vm.as_ref() else {
                return;
            };
            if row.entry_kind != "file" {
                return;
            }
            state.request_standard_file_view_after_compare_session_close(
                row.relative_path.as_str(),
                selected_index,
            );
            return;
        }
        Self::apply_row_selection(state, selected_index, false);
    }

    fn enter_compare_view(
        state: &mut AppState,
        relative_path: &str,
        preferred_row_focus: Option<&str>,
    ) {
        state.ensure_compare_tree_session();
        state.set_compare_focus_path(CompareFocusPath::relative(relative_path));
        if let Some(preferred) = preferred_row_focus {
            state.reveal_compare_view_path(preferred);
            state.set_compare_row_focus_path(Some(preferred));
        }
        if let Some(path) = state.compare_row_focus_path.clone() {
            state.reveal_compare_view_path(path.as_str());
            state.set_compare_row_focus_path(Some(path.as_str()));
            state.request_compare_view_scroll_to_path(path.as_str());
        }
        state.activate_workspace_session("compare-tree");
    }

    fn open_file_session(
        state: &mut AppState,
        relative_path: &str,
        selected_index: Option<usize>,
        opened_from_compare_view: bool,
    ) -> bool {
        let Some(result) = state.open_or_activate_file_session(relative_path) else {
            return false;
        };
        if !result.activated_existing {
            Self::apply_row_selection(state, selected_index, opened_from_compare_view);
        } else {
            state.can_return_to_compare_view = state.has_compare_tree_session();
        }
        state.sync_active_file_session_from_top_level();
        !result.has_cached_view_state && state.active_file_session_needs_diff_reload()
    }

    fn selection_path_at(state: &AppState, index: usize) -> Option<String> {
        state
            .entry_rows
            .get(index)
            .map(|row| row.relative_path.clone())
    }

    fn reveal_selected_path_in_tree(state: &mut AppState) {
        let selected_path = state
            .selected_row
            .and_then(|index| Self::selection_path_at(state, index))
            .or_else(|| state.selected_relative_path.clone());
        if let Some(path) = selected_path {
            state.reveal_navigator_tree_path(&path);
        }
    }

    fn request_selected_row_scroll_in_flat(state: &mut AppState) {
        if let Some(index) = state.selected_row {
            state.request_navigator_flat_scroll_to_source_index(index);
        }
    }

    fn request_selected_row_scroll_in_tree(state: &mut AppState) {
        if let Some(index) = state.selected_row {
            state.request_navigator_tree_scroll_to_source_index(index);
        }
    }

    fn apply_runtime_navigator_view_mode(state: &mut AppState, mode: NavigatorViewMode) {
        state.set_navigator_runtime_view_mode(mode);
        if matches!(
            state.effective_navigator_view_mode(),
            NavigatorViewMode::Tree
        ) {
            // Non-search flat -> tree continuity intentionally reuses the same
            // reveal/ensure-visible behavior as a surviving locate target.
            Self::reveal_selected_path_in_tree(state);
        }
        Self::reconcile_selected_row_membership(state);
        match state.effective_navigator_view_mode() {
            NavigatorViewMode::Flat => Self::request_selected_row_scroll_in_flat(state),
            NavigatorViewMode::Tree => Self::request_selected_row_scroll_in_tree(state),
        }
    }

    fn restore_selection_after_compare(
        state: &mut AppState,
        restore_relative_path: Option<&str>,
    ) -> bool {
        let Some(path) = restore_relative_path
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
        else {
            return false;
        };

        if matches!(
            state.effective_navigator_view_mode(),
            NavigatorViewMode::Tree
        ) {
            state.reveal_navigator_tree_path(&path);
        }
        let restore_index = state
            .entry_rows
            .iter()
            .position(|row| row.relative_path == path);
        match restore_index {
            Some(index) if state.is_row_member_in_active_results(index) => {
                state.selected_row = Some(index);
                state.selected_relative_path = Some(path);
                Self::clear_file_view_state(state);
                state.sync_active_file_session_from_top_level();
                true
            }
            Some(_) | None => {
                state.selected_row = None;
                state.selected_relative_path = Some(path);
                Self::clear_file_view_state(state);
                Self::set_analysis_stale_selection_hint(state);
                state.sync_active_file_session_from_top_level();
                false
            }
        }
    }

    fn prepare_selected_diff_load(&self) -> DiffLoadPlan {
        let mut state = self.state.lock().expect("state mutex poisoned");
        if state.workspace_session_confirmation_open() {
            return DiffLoadPlan::Noop;
        }
        if state.diff_loading {
            return DiffLoadPlan::Noop;
        }
        let use_compare_file_view = state.active_file_session_uses_compare_file_view();
        let has_loaded_file_content = if use_compare_file_view {
            state.selected_compare_file.is_some()
        } else {
            state.selected_diff.is_some()
        };
        if state.selected_row.is_some()
            && (has_loaded_file_content || state.diff_error_message.is_some())
        {
            return DiffLoadPlan::Noop;
        }

        let selected_row = state
            .selected_row
            .and_then(|idx| state.entry_rows.get(idx).cloned());
        let Some(row) = selected_row else {
            state.diff_loading = false;
            state.diff_error_message = Some(if use_compare_file_view {
                "select one compare row before loading Compare File View".to_string()
            } else {
                "select one compare row before loading detailed diff".to_string()
            });
            state.selected_diff = None;
            state.selected_compare_file = None;
            state.diff_warning = None;
            state.diff_truncated = false;
            state.analysis_available = false;
            state.clear_analysis_panel();
            Self::set_analysis_no_selection_hint(&mut state);
            state.status_text = if use_compare_file_view {
                "Compare File View unavailable".to_string()
            } else {
                "Detailed diff unavailable".to_string()
            };
            state.sync_active_file_session_from_top_level();
            return DiffLoadPlan::SyncOnly;
        };

        state.selected_relative_path = Some(row.relative_path.clone());
        state.diff_error_message = None;
        state.selected_diff = None;
        state.selected_compare_file = None;
        state.diff_warning = None;
        state.diff_truncated = false;
        state.analysis_available = false;
        state.clear_analysis_panel();
        let is_preview_mode =
            row.status == "left-only" || row.status == "right-only" || row.status == "equal";
        if !row.can_load_diff {
            let reason = row
                .diff_blocked_reason
                .clone()
                .unwrap_or_else(|| "selected row does not support detailed text diff".to_string());
            state.diff_loading = false;
            state.diff_warning = Some(reason);
            state.analysis_hint = Some(if use_compare_file_view {
                "Compare File View is unavailable for this selection.".to_string()
            } else {
                "Detailed diff is unavailable; AI analysis is disabled.".to_string()
            });
            state.status_text = if use_compare_file_view {
                "Compare File View unavailable for selected row".to_string()
            } else if is_preview_mode {
                "File preview unavailable for selected row".to_string()
            } else {
                "Detailed diff unavailable for selected row".to_string()
            };
            state.sync_active_file_session_from_top_level();
            return DiffLoadPlan::SyncOnly;
        }

        state.diff_loading = true;
        state.analysis_hint = Some(if use_compare_file_view {
            "Compare File View is loading...".to_string()
        } else if is_preview_mode {
            "File preview is loading...".to_string()
        } else {
            "Detailed diff is loading...".to_string()
        });
        state.sync_active_file_session_from_top_level();
        if use_compare_file_view {
            DiffLoadPlan::CompareFileBackground {
                left_root: state.left_root.clone(),
                right_root: state.right_root.clone(),
                row,
                state_ref: Arc::clone(&self.state),
            }
        } else {
            DiffLoadPlan::StandardBackground {
                left_root: state.left_root.clone(),
                right_root: state.right_root.clone(),
                row,
                state_ref: Arc::clone(&self.state),
            }
        }
    }

    fn execute_compare(&self) {
        let state_change_notifier = Arc::clone(&self.state_change_notifier);
        let presenter_for_restore = self.clone();
        let (left_root, right_root, restore_relative_path, restore_compare_focus_path, state_ref) = {
            let mut state = self.state.lock().expect("state mutex poisoned");
            if state.running {
                return;
            }
            state.sync_active_file_session_from_top_level();
            let restore_relative_path = state.selected_relative_path.clone();
            state.running = true;
            state.error_message = None;
            state.status_text = "Comparing...".to_string();
            state.selected_row = None;
            if let Some(path) = restore_relative_path.clone() {
                state.selected_relative_path = Some(path);
                Self::set_analysis_compare_restore_hint(&mut state);
            } else if state
                .selected_relative_path
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
            {
                Self::set_analysis_stale_selection_hint(&mut state);
            } else {
                state.selected_relative_path = None;
                Self::set_analysis_no_selection_hint(&mut state);
            }
            Self::clear_file_view_state(&mut state);
            state.sync_active_file_session_from_top_level();
            (
                state.left_root.clone(),
                state.right_root.clone(),
                restore_relative_path,
                state.compare_focus_path.clone(),
                Arc::clone(&self.state),
            )
        };
        self.notify_state_changed();

        thread::spawn(move || {
            // Give UI one short frame to render loading state before heavy work.
            thread::sleep(BACKGROUND_START_DELAY);

            let result = bridge::build_compare_request(&left_root, &right_root)
                .and_then(run_compare)
                .map(bridge::map_compare_report);
            let mut should_reload_restored_selection = false;

            {
                let mut state = state_ref.lock().expect("state mutex poisoned");
                state.running = false;
                match result {
                    Ok(vm) => {
                        let count = vm.entry_rows.len();
                        state.summary_text = vm.summary_text;
                        state.set_compare_foundation(vm.compare_foundation);
                        state.entry_rows = vm.entry_rows;
                        state.set_compare_focus_path(restore_compare_focus_path.clone());
                        state.prune_navigator_tree_expansion_overrides();
                        state.mark_navigator_projection_revisions();
                        state.warning_lines = vm.warnings;
                        state.truncated = vm.truncated;
                        state.error_message = None;
                        state.status_text = format!("Compare finished: {} entries", count);
                        state.mark_file_sessions_for_compare_restore();
                        if state.has_compare_tree_session() {
                            should_reload_restored_selection =
                                state.active_file_session_needs_diff_reload();
                        } else {
                            should_reload_restored_selection =
                                Self::restore_selection_after_compare(
                                    &mut state,
                                    restore_relative_path.as_deref(),
                                );
                        }
                        if !should_reload_restored_selection
                            && restore_relative_path.is_none()
                            && !state.has_compare_tree_session()
                        {
                            if state
                                .selected_relative_path
                                .as_deref()
                                .map(str::trim)
                                .is_some_and(|value| !value.is_empty())
                            {
                                Self::set_analysis_stale_selection_hint(&mut state);
                            } else {
                                Self::set_analysis_no_selection_hint(&mut state);
                            }
                        }
                    }
                    Err(message) => {
                        state.summary_text.clear();
                        state.clear_compare_foundation();
                        state.entry_rows.clear();
                        state.prune_navigator_tree_expansion_overrides();
                        state.mark_navigator_projection_revisions();
                        state.warning_lines.clear();
                        state.truncated = false;
                        state.error_message = Some(message);
                        state.status_text = "Compare failed".to_string();
                        state.mark_file_sessions_for_compare_restore();
                        if state
                            .selected_relative_path
                            .as_deref()
                            .map(str::trim)
                            .is_some_and(|value| !value.is_empty())
                        {
                            Self::set_analysis_stale_selection_hint(&mut state);
                        } else {
                            Self::set_analysis_no_selection_hint(&mut state);
                        }
                    }
                }
            }
            if should_reload_restored_selection {
                presenter_for_restore.execute_load_selected_diff();
            } else {
                Self::notify_state_changed_with(&state_change_notifier);
            }
        });
    }

    fn execute_load_selected_diff(&self) {
        let state_change_notifier = Arc::clone(&self.state_change_notifier);
        let plan = self.prepare_selected_diff_load();
        match plan {
            DiffLoadPlan::Noop => return,
            DiffLoadPlan::SyncOnly => {
                self.notify_state_changed();
                return;
            }
            DiffLoadPlan::StandardBackground {
                left_root,
                right_root,
                row,
                state_ref,
            } => {
                self.notify_state_changed();
                thread::spawn(move || {
                    // Give UI one short frame to render loading state before heavy work.
                    thread::sleep(BACKGROUND_START_DELAY);

                    let result = if row.status == "left-only"
                        || row.status == "right-only"
                        || row.status == "equal"
                    {
                        let preview_vm =
                            bridge::map_single_side_file_preview(&left_root, &right_root, &row);
                        Ok((row, preview_vm))
                    } else {
                        let relative_path = row.relative_path.clone();
                        bridge::build_text_diff_request(&left_root, &right_root, &row)
                            .and_then(run_text_diff)
                            .map(|diff_result| {
                                (
                                    row,
                                    bridge::map_text_diff_result(&relative_path, diff_result),
                                )
                            })
                    };

                    {
                        let mut state = state_ref.lock().expect("state mutex poisoned");
                        state.diff_loading = false;
                        match result {
                            Ok((row, diff_vm)) => {
                                let is_preview_mode = row.status == "left-only"
                                    || row.status == "right-only"
                                    || row.status == "equal";
                                state.selected_relative_path = Some(diff_vm.relative_path.clone());
                                state.diff_warning = diff_vm.warning.clone();
                                state.diff_truncated = diff_vm.truncated;
                                state.diff_error_message = None;
                                state.selected_diff = Some(diff_vm);
                                state.selected_compare_file = None;
                                state.analysis_available = row.can_load_analysis;
                                state.analysis_error_message = None;
                                state.analysis_result = None;
                                state.analysis_loading = false;
                                state.analysis_hint = Some(if row.can_load_analysis {
                                    "Click Analyze to run AI risk review.".to_string()
                                } else {
                                    row.analysis_blocked_reason.unwrap_or_else(|| {
                                        "selected row does not support AI analysis".to_string()
                                    })
                                });
                                state.status_text = if is_preview_mode {
                                    "File preview loaded".to_string()
                                } else {
                                    "Detailed diff loaded".to_string()
                                };
                                state.sync_active_file_session_from_top_level();
                            }
                            Err(message) => {
                                state.diff_error_message = Some(message);
                                state.selected_diff = None;
                                state.selected_compare_file = None;
                                state.diff_warning = None;
                                state.diff_truncated = false;
                                state.analysis_available = false;
                                state.clear_analysis_panel();
                                state.analysis_hint = Some(
                                    "Detailed diff is unavailable; AI analysis is disabled."
                                        .to_string(),
                                );
                                state.status_text = "Detailed diff unavailable".to_string();
                                state.sync_active_file_session_from_top_level();
                            }
                        }
                    }
                    Self::notify_state_changed_with(&state_change_notifier);
                });
            }
            DiffLoadPlan::CompareFileBackground {
                left_root,
                right_root,
                row,
                state_ref,
            } => {
                self.notify_state_changed();
                thread::spawn(move || {
                    // Give UI one short frame to render loading state before heavy work.
                    thread::sleep(BACKGROUND_START_DELAY);

                    let result = if row.status == "left-only"
                        || row.status == "right-only"
                        || row.status == "equal"
                    {
                        build_preview_compare_file(&left_root, &right_root, &row)
                    } else {
                        let relative_path = row.relative_path.clone();
                        bridge::build_text_diff_request(&left_root, &right_root, &row)
                            .and_then(run_text_diff)
                            .map(|diff_result| {
                                map_text_diff_result_to_compare_file(&relative_path, diff_result)
                            })
                    };

                    {
                        let mut state = state_ref.lock().expect("state mutex poisoned");
                        state.diff_loading = false;
                        match result {
                            Ok(compare_vm) => {
                                state.selected_relative_path =
                                    Some(compare_vm.relative_path.clone());
                                state.diff_warning = compare_vm.warning.clone();
                                state.diff_truncated = compare_vm.truncated;
                                state.diff_error_message = None;
                                state.selected_diff = None;
                                state.selected_compare_file = Some(compare_vm);
                                state.analysis_available = false;
                                state.analysis_error_message = None;
                                state.analysis_result = None;
                                state.analysis_loading = false;
                                state.analysis_hint = Some(
                                    "Dedicated Compare File View is active for this tab."
                                        .to_string(),
                                );
                                state.status_text = "Compare file loaded".to_string();
                                state.sync_active_file_session_from_top_level();
                            }
                            Err(message) => {
                                state.diff_error_message = Some(message);
                                state.selected_diff = None;
                                state.selected_compare_file = None;
                                state.diff_warning = None;
                                state.diff_truncated = false;
                                state.analysis_available = false;
                                state.clear_analysis_panel();
                                state.analysis_hint = Some(
                                    "Compare File View is unavailable for this selection."
                                        .to_string(),
                                );
                                state.status_text = "Compare File View unavailable".to_string();
                                state.sync_active_file_session_from_top_level();
                            }
                        }
                    }
                    Self::notify_state_changed_with(&state_change_notifier);
                });
            }
        }
    }

    fn execute_locate_and_open(&self, relative_path: String) {
        let should_load_selected_diff = {
            let mut state = self.state.lock().expect("state mutex poisoned");
            let relative_path = relative_path.trim().to_string();
            if relative_path.is_empty() {
                return;
            }

            state.set_entry_filter(String::new());
            state.set_navigator_runtime_view_mode(NavigatorViewMode::Tree);
            state.reveal_navigator_tree_path(&relative_path);

            match state
                .row_index_for_relative_path(&relative_path)
                .filter(|index| state.is_row_member_in_active_results(*index))
            {
                Some(index) => {
                    state.request_navigator_tree_scroll_to_source_index(index);
                    if state.has_compare_tree_session() {
                        state.request_standard_file_view_after_compare_session_close(
                            &relative_path,
                            Some(index),
                        );
                        false
                    } else {
                        Self::apply_row_selection(&mut state, Some(index), false);
                        true
                    }
                }
                None => {
                    state.selected_row = None;
                    state.selected_relative_path = Some(relative_path);
                    state.can_return_to_compare_view = false;
                    Self::clear_file_view_state(&mut state);
                    Self::set_analysis_stale_selection_hint(&mut state);
                    state.sync_active_file_session_from_top_level();
                    false
                }
            }
        };
        self.notify_state_changed();

        if should_load_selected_diff {
            self.execute_load_selected_diff();
        }
    }

    fn execute_load_ai_analysis(&self) {
        let state_change_notifier = Arc::clone(&self.state_change_notifier);
        let (selected_row, selected_diff, diff_warning, diff_truncated, ai_config, state_ref) = {
            let mut state = self.state.lock().expect("state mutex poisoned");
            if state.analysis_loading {
                return;
            }

            let selected_row = state
                .selected_row
                .and_then(|idx| state.entry_rows.get(idx).cloned());
            let Some(row) = selected_row.as_ref() else {
                state.analysis_error_message =
                    Some("select one compare row before running AI analysis".to_string());
                state.sync_active_file_session_from_top_level();
                return;
            };
            if !row.can_load_analysis {
                state.analysis_error_message =
                    Some(row.analysis_blocked_reason.clone().unwrap_or_else(|| {
                        "selected row does not support AI analysis".to_string()
                    }));
                state.sync_active_file_session_from_top_level();
                return;
            }
            if state.diff_loading {
                state.analysis_error_message =
                    Some("wait until detailed diff loading completes".to_string());
                state.sync_active_file_session_from_top_level();
                return;
            }
            if state.selected_diff.is_none() {
                state.analysis_error_message =
                    Some("load detailed diff before running AI analysis".to_string());
                state.sync_active_file_session_from_top_level();
                return;
            }
            if state.analysis_remote_mode() && !state.analysis_remote_config_ready() {
                state.analysis_error_message = Some(
                    "remote provider configuration is incomplete (endpoint/api key/model required)"
                        .to_string(),
                );
                state.sync_active_file_session_from_top_level();
                return;
            }

            state.analysis_loading = true;
            state.analysis_error_message = None;
            state.analysis_result = None;
            state.analysis_hint = Some(format!(
                "Running AI analysis with {} provider...",
                state.analysis_provider_mode_text()
            ));
            state.sync_active_file_session_from_top_level();
            (
                selected_row,
                state.selected_diff.clone(),
                state.diff_warning.clone(),
                state.diff_truncated,
                state.analysis_ai_config(),
                Arc::clone(&self.state),
            )
        };
        self.notify_state_changed();

        thread::spawn(move || {
            // Give UI one short frame to render loading state before heavy work.
            thread::sleep(BACKGROUND_START_DELAY);

            let result = selected_row
                .ok_or_else(|| "select one compare row before running AI analysis".to_string())
                .and_then(|row| {
                    selected_diff
                        .as_ref()
                        .ok_or_else(|| "load detailed diff before running AI analysis".to_string())
                        .and_then(|diff| {
                            bridge::build_analyze_diff_request(
                                &row,
                                diff,
                                diff_warning.as_deref(),
                                diff_truncated,
                                ai_config.clone(),
                            )
                        })
                })
                .and_then(run_ai_analysis)
                .map(bridge::map_analyze_diff_response);

            {
                let mut state = state_ref.lock().expect("state mutex poisoned");
                state.analysis_loading = false;
                match result {
                    Ok(analysis_vm) => {
                        state.analysis_error_message = None;
                        state.analysis_result = Some(analysis_vm);
                        state.analysis_hint = Some(format!(
                            "AI analysis loaded from {} provider.",
                            state.analysis_provider_mode_text()
                        ));
                        state.status_text = "AI analysis loaded".to_string();
                        state.sync_active_file_session_from_top_level();
                    }
                    Err(message) => {
                        state.analysis_error_message = Some(message);
                        state.analysis_result = None;
                        state.status_text = "AI analysis unavailable".to_string();
                        state.sync_active_file_session_from_top_level();
                    }
                }
            }
            Self::notify_state_changed_with(&state_change_notifier);
        });
    }

    fn reconcile_selected_row_membership(state: &mut AppState) {
        if let Some(selected_row) = state.selected_row {
            if !state.is_row_member_in_active_results(selected_row) {
                let stale_path = Self::selection_path_at(state, selected_row);
                state.selected_row = None;
                state.selected_relative_path = stale_path;
                state.can_return_to_compare_view = false;
                Self::clear_file_view_state(state);
                Self::set_analysis_stale_selection_hint(state);
                state.sync_active_file_session_from_top_level();
            }
        }
    }

    fn open_file_view_from_compare(&self, relative_path: String) {
        let should_load_selected_diff = {
            let mut state = self.state.lock().expect("state mutex poisoned");
            let relative_path = relative_path.trim().to_string();
            if relative_path.is_empty() {
                return;
            }

            match state.row_index_for_relative_path(&relative_path) {
                Some(index) => {
                    Self::open_file_session(&mut state, &relative_path, Some(index), true)
                }
                None => {
                    let _ = state.open_or_activate_file_session(&relative_path);
                    state.selected_row = None;
                    state.selected_relative_path = Some(relative_path);
                    state.can_return_to_compare_view = true;
                    Self::clear_file_view_state(&mut state);
                    Self::set_analysis_stale_selection_hint(&mut state);
                    state.sync_active_file_session_from_top_level();
                    false
                }
            }
        };
        self.notify_state_changed();

        if should_load_selected_diff {
            self.execute_load_selected_diff();
        }
    }
}

fn preferred_compare_child_focus(current_focus: &str, target_focus: &str) -> Option<String> {
    let current_parts = current_focus
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let target_parts = target_focus
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if current_parts.len() <= target_parts.len()
        || !current_parts.starts_with(target_parts.as_slice())
    {
        return None;
    }

    Some(current_parts[..target_parts.len() + 1].join("/"))
}

fn parse_timeout_secs(raw: &str) -> Result<u64, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err("Timeout is required and must be a positive integer.".to_string());
    }
    let parsed = text
        .parse::<u64>()
        .map_err(|_| "Timeout must be a positive integer (seconds).".to_string())?;
    if parsed == 0 {
        return Err("Timeout must be greater than 0.".to_string());
    }
    Ok(parsed)
}

fn navigator_view_mode_from_settings(value: DefaultResultsView) -> NavigatorViewMode {
    match value {
        DefaultResultsView::Tree => NavigatorViewMode::Tree,
        DefaultResultsView::Flat => NavigatorViewMode::Flat,
    }
}

fn settings_default_results_view(value: NavigatorViewMode) -> DefaultResultsView {
    match value {
        NavigatorViewMode::Tree => DefaultResultsView::Tree,
        NavigatorViewMode::Flat => DefaultResultsView::Flat,
    }
}

#[cfg(test)]
#[path = "tests/presenter_foundation_tests.rs"]
mod foundation_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    fn wait_until<F>(presenter: &Presenter, predicate: F) -> AppState
    where
        F: Fn(&AppState) -> bool,
    {
        for _ in 0..200 {
            let snapshot = presenter.state_snapshot();
            if predicate(&snapshot) {
                return snapshot;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out waiting for presenter state condition");
    }

    #[test]
    fn run_compare_with_invalid_input_sets_error() {
        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot("".to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot("".to_string()));
        presenter.handle_command(UiCommand::RunCompare);
        let snapshot = wait_until(&presenter, |state| !state.running);

        assert!(!snapshot.running);
        assert!(snapshot.error_message.is_some());
        assert!(snapshot.entry_rows.is_empty());
    }

    #[test]
    fn run_compare_with_valid_paths_populates_rows() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("a.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("a.txt"), "right\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        let snapshot = wait_until(&presenter, |state| !state.running);

        assert!(!snapshot.running);
        assert!(snapshot.error_message.is_none());
        assert!(!snapshot.summary_text.is_empty());
        assert!(!snapshot.entry_rows.is_empty());
        assert!(snapshot.status_text.contains("Compare finished"));
        assert!(
            snapshot
                .entry_rows
                .iter()
                .any(|row| row.relative_path.contains("a.txt"))
        );
    }

    #[test]
    fn select_row_updates_state() {
        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::SelectRow(3));
        assert_eq!(presenter.state_snapshot().selected_row, None);
        presenter.handle_command(UiCommand::SelectRow(-1));
        assert_eq!(presenter.state_snapshot().selected_row, None);
    }

    #[test]
    fn filter_keeps_base_entries_and_reduces_visible_entries() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("a.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("a.txt"), "right\n").expect("right file should be written");
        fs::write(left.path().join("b.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("b.txt"), "right\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::UpdateEntryFilter("a.txt".to_string()));

        let snapshot = presenter.state_snapshot();
        assert_eq!(snapshot.entry_rows.len(), 2);
        assert_eq!(snapshot.filtered_entry_rows_with_index().len(), 1);
        assert_eq!(
            snapshot.filtered_entry_rows_with_index()[0].1.relative_path,
            "a.txt"
        );
    }

    #[test]
    fn load_selected_diff_for_diffable_row_populates_panel() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.txt"), "a\nleft\n").expect("left file should be written");
        fs::write(right.path().join("doc.txt"), "a\nright\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);

        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.selected_diff.is_some());
        assert!(!snapshot.diff_viewer_rows().is_empty());
        assert_eq!(snapshot.selected_relative_path.as_deref(), Some("doc.txt"));
    }

    #[test]
    fn filter_hides_selected_row_and_clears_diff_panel() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("a.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("a.txt"), "right\n").expect("right file should be written");
        fs::write(left.path().join("b.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("b.txt"), "right\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        let different_index = presenter
            .state_snapshot()
            .entry_rows
            .iter()
            .position(|row| row.status == "different")
            .expect("different row should exist");
        presenter.handle_command(UiCommand::SelectRow(different_index as i32));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);
        assert!(presenter.state_snapshot().selected_diff.is_some());

        presenter.handle_command(UiCommand::UpdateEntryFilter("b.txt".to_string()));
        let snapshot = presenter.state_snapshot();
        assert_eq!(snapshot.selected_row, None);
        assert!(snapshot.selected_diff.is_none());
        assert_eq!(snapshot.selected_relative_path.as_deref(), Some("a.txt"));
        assert_eq!(
            snapshot.analysis_hint.as_deref(),
            Some("Previous selection is no longer active in the current Results / Navigator set.")
        );
        assert_eq!(
            snapshot.diff_shell_state(),
            crate::state::DiffShellState::StaleSelection
        );
        assert_eq!(
            snapshot.analysis_panel_state(),
            crate::state::AnalysisPanelState::StaleSelection
        );
    }

    #[test]
    fn status_filter_hides_selected_row_and_clears_diff_panel() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("a.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("a.txt"), "right\n").expect("right file should be written");
        fs::write(left.path().join("b.txt"), "same\n").expect("left file should be written");
        fs::write(right.path().join("b.txt"), "same\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        let different_index = presenter
            .state_snapshot()
            .entry_rows
            .iter()
            .position(|row| row.status == "different")
            .expect("different row should exist");
        presenter.handle_command(UiCommand::SelectRow(different_index as i32));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);
        assert!(presenter.state_snapshot().selected_diff.is_some());

        presenter.handle_command(UiCommand::UpdateEntryStatusFilter("equal".to_string()));
        let snapshot = presenter.state_snapshot();
        assert_eq!(snapshot.selected_row, None);
        assert!(snapshot.selected_diff.is_none());
        assert_eq!(snapshot.selected_relative_path.as_deref(), Some("a.txt"));
        assert_eq!(snapshot.entry_status_filter, "equal");
        assert_eq!(snapshot.filtered_entry_rows_with_index().len(), 1);
        assert_eq!(
            snapshot.diff_shell_state(),
            crate::state::DiffShellState::StaleSelection
        );
    }

    #[test]
    fn filter_keeps_selected_row_when_it_remains_visible() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("alpha.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("alpha.txt"), "right\n").expect("right file should be written");
        fs::write(left.path().join("beta.txt"), "same\n").expect("left file should be written");
        fs::write(right.path().join("beta.txt"), "same\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        let selected_index = presenter
            .state_snapshot()
            .entry_rows
            .iter()
            .position(|row| row.relative_path == "alpha.txt")
            .expect("alpha row should exist");
        presenter.handle_command(UiCommand::SelectRow(selected_index as i32));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);

        presenter.handle_command(UiCommand::UpdateEntryFilter("alpha".to_string()));
        let snapshot = presenter.state_snapshot();
        assert_eq!(snapshot.selected_row, Some(selected_index));
        assert_eq!(
            snapshot.selected_relative_path.as_deref(),
            Some("alpha.txt")
        );
        assert!(snapshot.selected_diff.is_some());
    }

    #[test]
    fn status_filter_keeps_selected_row_when_it_remains_visible() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("alpha.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("alpha.txt"), "right\n").expect("right file should be written");
        fs::write(left.path().join("beta.txt"), "same\n").expect("left file should be written");
        fs::write(right.path().join("beta.txt"), "same\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        let selected_index = presenter
            .state_snapshot()
            .entry_rows
            .iter()
            .position(|row| row.status == "different")
            .expect("different row should exist");
        presenter.handle_command(UiCommand::SelectRow(selected_index as i32));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);

        presenter.handle_command(UiCommand::UpdateEntryStatusFilter("different".to_string()));
        let snapshot = presenter.state_snapshot();
        assert_eq!(snapshot.selected_row, Some(selected_index));
        assert!(snapshot.selected_diff.is_some());
    }

    #[test]
    fn collapsing_selected_file_ancestor_keeps_file_view_open() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir(left.path().join("src")).expect("left src should be created");
        fs::create_dir(right.path().join("src")).expect("right src should be created");
        fs::write(left.path().join("src/main.rs"), "left\n").expect("left file should be written");
        fs::write(right.path().join("src/main.rs"), "right\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);

        let selected_index = presenter
            .state_snapshot()
            .entry_rows
            .iter()
            .position(|row| row.relative_path == "src/main.rs")
            .expect("nested file row should exist");
        presenter.handle_command(UiCommand::SelectRow(selected_index as i32));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);

        presenter.handle_command(UiCommand::ToggleNavigatorTreeNode("src".to_string()));
        let snapshot = presenter.state_snapshot();
        assert_eq!(snapshot.selected_row, Some(selected_index));
        assert_eq!(
            snapshot.selected_relative_path.as_deref(),
            Some("src/main.rs")
        );
        assert!(snapshot.selected_diff.is_some());
        assert_eq!(
            snapshot.diff_shell_state(),
            crate::state::DiffShellState::DetailedReady
        );
    }

    #[test]
    fn load_selected_diff_for_non_diffable_row_sets_unavailable_state() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir(left.path().join("left_only_dir"))
            .expect("left-only directory should be created");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);

        assert!(snapshot.error_message.is_none());
        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.diff_warning.is_some());
        assert!(snapshot.selected_diff.is_none());
        assert_eq!(
            snapshot.status_text,
            "File preview unavailable for selected row"
        );
    }

    #[test]
    fn load_selected_diff_for_left_only_file_opens_single_side_preview() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("left_only.txt"), "line-1\nline-2\n")
            .expect("left file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);

        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.selected_diff.is_some());
        assert!(
            snapshot
                .selected_diff
                .as_ref()
                .map(|value| value.summary_text.contains("left-only preview"))
                .unwrap_or(false)
        );
        assert_eq!(snapshot.analysis_available, false);
    }

    #[test]
    fn load_selected_diff_for_equal_file_opens_equal_preview() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("equal.txt"), "same\nline\n")
            .expect("left file should be written");
        fs::write(right.path().join("equal.txt"), "same\nline\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);

        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.selected_diff.is_some());
        assert!(
            snapshot
                .selected_diff
                .as_ref()
                .map(|value| value.summary_text.contains("equal preview"))
                .unwrap_or(false)
        );
        assert_eq!(snapshot.analysis_available, false);
    }

    #[test]
    fn opening_compare_originated_file_session_loads_dedicated_compare_file_view() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir_all(left.path().join("src")).expect("left src directory should exist");
        fs::create_dir_all(right.path().join("src")).expect("right src directory should exist");
        fs::write(left.path().join("src/main.txt"), "你好\nleft value\n")
            .expect("left file should be written");
        fs::write(right.path().join("src/main.txt"), "你好\nright value\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);

        presenter.handle_command(UiCommand::OpenCompareView("src".to_string()));
        presenter.handle_command(UiCommand::OpenFileViewFromCompare(
            "src/main.txt".to_string(),
        ));
        let snapshot = wait_until(&presenter, |state| {
            !state.diff_loading && state.selected_compare_file.is_some()
        });

        assert!(snapshot.compare_file_view_active());
        assert!(snapshot.selected_compare_file.is_some());
        assert!(snapshot.selected_diff.is_none());
        assert!(snapshot.compare_file_has_rows());
        assert_eq!(snapshot.status_text, "Compare file loaded");
        assert!(
            snapshot
                .compare_file_row_projections()
                .iter()
                .any(|row| row.row_kind == "modified")
        );
        assert!(
            snapshot
                .compare_file_row_projections()
                .iter()
                .flat_map(|row| row.left_segments.iter().chain(row.right_segments.iter()))
                .any(|segment| segment.tone == "emphasis")
        );
    }

    #[test]
    fn rerun_compare_restores_previous_diff_selection_when_path_still_exists() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.txt"), "a\nleft\n").expect("left file should be written");
        fs::write(right.path().join("doc.txt"), "a\nright\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::UpdateEntryFilter("doc".to_string()));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);
        assert!(presenter.state_snapshot().selected_diff.is_some());

        presenter.handle_command(UiCommand::RunCompare);
        let snapshot = wait_until(&presenter, |state| !state.running && !state.diff_loading);
        assert_eq!(snapshot.selected_relative_path.as_deref(), Some("doc.txt"));
        assert!(snapshot.selected_row.is_some());
        assert!(snapshot.selected_diff.is_some());
        assert!(snapshot.diff_error_message.is_none());
        assert_eq!(snapshot.entry_filter, "doc");
    }

    #[test]
    fn rerun_compare_keeps_stale_selection_when_path_no_longer_exists() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        let left_file = left.path().join("doc.txt");
        let right_file = right.path().join("doc.txt");
        fs::write(&left_file, "a\nleft\n").expect("left file should be written");
        fs::write(&right_file, "a\nright\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);

        fs::remove_file(&left_file).expect("left file should be removed");
        fs::remove_file(&right_file).expect("right file should be removed");

        presenter.handle_command(UiCommand::RunCompare);
        let snapshot = wait_until(&presenter, |state| !state.running);
        assert_eq!(snapshot.selected_row, None);
        assert_eq!(snapshot.selected_relative_path.as_deref(), Some("doc.txt"));
        assert!(snapshot.selected_diff.is_none());
        assert_eq!(
            snapshot.diff_shell_state(),
            crate::state::DiffShellState::StaleSelection
        );
    }

    #[test]
    fn locate_and_open_from_search_restores_tree_context_and_opens_diff() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir_all(left.path().join("src/bin"))
            .expect("left nested directory should be created");
        fs::create_dir_all(right.path().join("src/bin"))
            .expect("right nested directory should be created");
        fs::write(left.path().join("src/bin/main.rs"), "fn old() {}\n")
            .expect("left file should be written");
        fs::write(right.path().join("src/bin/main.rs"), "fn new() {}\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::UpdateEntryFilter("main".to_string()));

        presenter.handle_command(UiCommand::LocateAndOpen("src/bin/main.rs".to_string()));
        let snapshot = wait_until(&presenter, |state| {
            !state.diff_loading
                && state.entry_filter.is_empty()
                && state.selected_relative_path.as_deref() == Some("src/bin/main.rs")
                && state.selected_diff.is_some()
        });

        assert_eq!(
            snapshot.navigator_runtime_view_mode,
            NavigatorViewMode::Tree
        );
        assert_eq!(
            snapshot.effective_navigator_view_mode(),
            NavigatorViewMode::Tree
        );
        assert_eq!(
            snapshot.selected_row,
            snapshot.row_index_for_relative_path("src/bin/main.rs")
        );
        assert_eq!(
            snapshot.navigator_tree_expansion_overrides.get("src/bin"),
            Some(&true)
        );
        assert!(
            snapshot
                .navigator_tree_row_projections()
                .iter()
                .any(|row| row.key == "src/bin/main.rs")
        );
        assert_eq!(snapshot.navigator_tree_scroll_request_revision, 1);
        assert_eq!(
            snapshot.navigator_tree_scroll_target_source_index,
            snapshot.row_index_for_relative_path("src/bin/main.rs")
        );
    }

    #[test]
    fn locate_and_open_from_flat_restores_tree_context_and_opens_diff() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir_all(left.path().join("src/bin"))
            .expect("left nested directory should be created");
        fs::create_dir_all(right.path().join("src/bin"))
            .expect("right nested directory should be created");
        fs::write(left.path().join("src/bin/main.rs"), "fn old() {}\n")
            .expect("left file should be written");
        fs::write(right.path().join("src/bin/main.rs"), "fn new() {}\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SetNavigatorViewModeFlat);

        presenter.handle_command(UiCommand::LocateAndOpen("src/bin/main.rs".to_string()));
        let snapshot = wait_until(&presenter, |state| {
            !state.diff_loading
                && state.selected_relative_path.as_deref() == Some("src/bin/main.rs")
                && state.selected_diff.is_some()
        });

        assert_eq!(
            snapshot.navigator_runtime_view_mode,
            NavigatorViewMode::Tree
        );
        assert_eq!(
            snapshot.effective_navigator_view_mode(),
            NavigatorViewMode::Tree
        );
        assert_eq!(
            snapshot.selected_row,
            snapshot.row_index_for_relative_path("src/bin/main.rs")
        );
        assert_eq!(
            snapshot.navigator_tree_expansion_overrides.get("src/bin"),
            Some(&true)
        );
        assert_eq!(snapshot.navigator_tree_scroll_request_revision, 1);
        assert_eq!(
            snapshot.navigator_tree_scroll_target_source_index,
            snapshot.row_index_for_relative_path("src/bin/main.rs")
        );
    }

    #[test]
    fn navigate_compare_view_reanchors_directory_without_resetting_compare_session() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir_all(left.path().join("src/bin"))
            .expect("left nested directory should be created");
        fs::create_dir_all(right.path().join("src/bin"))
            .expect("right nested directory should be created");
        fs::write(left.path().join("src/bin/main.rs"), "fn old() {}\n")
            .expect("left file should be written");
        fs::write(right.path().join("src/bin/main.rs"), "fn new() {}\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);

        presenter.handle_command(UiCommand::OpenCompareView("src".to_string()));
        presenter.handle_command(UiCommand::OpenFileViewFromCompare(
            "src/bin/main.rs".to_string(),
        ));
        wait_until(&presenter, |state| {
            !state.diff_loading && state.selected_compare_file.is_some()
        });

        presenter.handle_command(UiCommand::NavigateCompareView("src/bin".to_string()));
        let snapshot = wait_until(&presenter, |state| {
            state.active_session_id.as_deref() == Some("compare-tree")
                && state.compare_focus_path_raw_text() == "src/bin"
        });

        assert!(snapshot.compare_tree_session.is_some());
        assert_eq!(snapshot.file_sessions.len(), 1);
        assert_eq!(snapshot.compare_focus_path_raw_text(), "src/bin");
        assert_eq!(
            snapshot.compare_row_focus_path.as_deref(),
            Some("src/bin/main.rs")
        );
        assert_eq!(
            snapshot.compare_view_scroll_target_relative_path.as_deref(),
            Some("src/bin/main.rs")
        );
        assert_eq!(snapshot.workspace_mode_text(), "compare-view");
        assert_eq!(
            snapshot.compare_view_breadcrumb_paths(),
            vec!["".to_string(), "src".to_string(), "src/bin".to_string()]
        );

        presenter.handle_command(UiCommand::NavigateCompareView(String::new()));
        let root_snapshot = wait_until(&presenter, |state| {
            state.active_session_id.as_deref() == Some("compare-tree")
                && state.compare_focus_path_raw_text().is_empty()
        });

        assert_eq!(root_snapshot.file_sessions.len(), 1);
        assert_eq!(root_snapshot.compare_focus_path_raw_text(), "");
        assert_eq!(
            root_snapshot.compare_view_breadcrumb_paths(),
            vec!["".to_string()]
        );
    }

    #[test]
    fn compare_view_quick_locate_reveals_matches_without_filtering_results() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir_all(left.path().join("alpha/deep"))
            .expect("left alpha/deep directory should be created");
        fs::create_dir_all(right.path().join("alpha/deep"))
            .expect("right alpha/deep directory should be created");
        fs::create_dir_all(left.path().join("beta/deep"))
            .expect("left beta/deep directory should be created");
        fs::create_dir_all(right.path().join("beta/deep"))
            .expect("right beta/deep directory should be created");
        fs::write(left.path().join("alpha/deep/first-main.txt"), "old alpha\n")
            .expect("left alpha file should be written");
        fs::write(
            right.path().join("alpha/deep/first-main.txt"),
            "new alpha\n",
        )
        .expect("right alpha file should be written");
        fs::write(left.path().join("beta/deep/second-main.txt"), "old beta\n")
            .expect("left beta file should be written");
        fs::write(right.path().join("beta/deep/second-main.txt"), "new beta\n")
            .expect("right beta file should be written");
        fs::write(left.path().join("notes.txt"), "same\n").expect("left notes should be written");
        fs::write(right.path().join("notes.txt"), "same\n").expect("right notes should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);

        presenter.handle_command(UiCommand::NavigateCompareView(String::new()));
        wait_until(&presenter, |state| {
            state.active_session_id.as_deref() == Some("compare-tree")
        });

        presenter.handle_command(UiCommand::UpdateCompareViewQuickLocate("main".to_string()));
        let snapshot = wait_until(&presenter, |state| {
            state.compare_row_focus_path.as_deref() == Some("alpha/deep/first-main.txt")
        });

        assert_eq!(snapshot.entry_filter, "");
        assert_eq!(snapshot.compare_focus_path_raw_text(), "");
        assert_eq!(snapshot.compare_view_quick_locate_query(), "main");
        assert_eq!(
            snapshot.compare_view_scroll_target_relative_path.as_deref(),
            Some("alpha/deep/first-main.txt")
        );
        assert!(
            snapshot
                .compare_view_row_projections()
                .iter()
                .any(|row| row.relative_path == "notes.txt")
        );
        assert_eq!(
            snapshot.compare_view_expansion_overrides.get("alpha/deep"),
            Some(&true)
        );

        presenter.handle_command(UiCommand::AdvanceCompareViewQuickLocate);
        let advanced_snapshot = wait_until(&presenter, |state| {
            state.compare_row_focus_path.as_deref() == Some("beta/deep/second-main.txt")
        });

        assert_eq!(advanced_snapshot.entry_filter, "");
        assert_eq!(advanced_snapshot.compare_focus_path_raw_text(), "");
        assert!(
            advanced_snapshot
                .compare_view_row_projections()
                .iter()
                .any(|row| row.relative_path == "notes.txt")
        );
        assert_eq!(
            advanced_snapshot
                .compare_view_scroll_target_relative_path
                .as_deref(),
            Some("beta/deep/second-main.txt")
        );
        assert_eq!(
            advanced_snapshot
                .compare_view_expansion_overrides
                .get("beta/deep"),
            Some(&true)
        );

        presenter.handle_command(UiCommand::RetreatCompareViewQuickLocate);
        let retreat_snapshot = wait_until(&presenter, |state| {
            state.compare_row_focus_path.as_deref() == Some("alpha/deep/first-main.txt")
        });

        assert_eq!(retreat_snapshot.entry_filter, "");
        assert_eq!(retreat_snapshot.compare_focus_path_raw_text(), "");
        assert_eq!(
            retreat_snapshot
                .compare_view_scroll_target_relative_path
                .as_deref(),
            Some("alpha/deep/first-main.txt")
        );
    }

    #[test]
    fn diff_error_does_not_pollute_compare_error() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        let left_file = left.path().join("doc.txt");
        let right_file = right.path().join("doc.txt");
        fs::write(&left_file, "a\nleft\n").expect("left file should be written");
        fs::write(&right_file, "a\nright\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        fs::remove_file(&right_file).expect("right file should be removed");
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);

        assert!(snapshot.error_message.is_none());
        assert!(snapshot.diff_error_message.is_some());
    }

    #[test]
    fn run_compare_sets_running_true_before_background_completion() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("a.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("a.txt"), "right\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);

        assert!(presenter.state_snapshot().running);
        let snapshot = wait_until(&presenter, |state| !state.running);
        assert!(snapshot.error_message.is_none());
    }

    #[test]
    fn load_diff_sets_loading_true_before_background_completion() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.txt"), "a\nleft\n").expect("left file should be written");
        fs::write(right.path().join("doc.txt"), "a\nright\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);

        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        assert!(presenter.state_snapshot().diff_loading);
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);
        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.selected_diff.is_some());
    }

    #[test]
    fn load_ai_analysis_sets_loading_true_before_background_completion() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.rs"), "fn old() {}\n")
            .expect("left file should be written");
        fs::write(
            right.path().join("doc.rs"),
            "fn new() {\n    unsafe { panic!(\"boom\"); }\n}\n",
        )
        .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);

        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);

        presenter.handle_command(UiCommand::LoadAiAnalysis);
        assert!(presenter.state_snapshot().analysis_loading);
        let snapshot = wait_until(&presenter, |state| !state.analysis_loading);
        assert!(snapshot.analysis_error_message.is_none());
        assert!(snapshot.analysis_result.is_some());
    }

    #[test]
    fn load_ai_analysis_for_non_analyzable_row_sets_analysis_error_only() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("left_only.txt"), "left\n")
            .expect("left file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadAiAnalysis);

        let snapshot = presenter.state_snapshot();
        assert!(snapshot.error_message.is_none());
        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.analysis_error_message.is_some());
        assert!(snapshot.analysis_result.is_none());
    }

    #[test]
    fn remote_provider_missing_config_sets_analysis_error() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.txt"), "a\nleft\n").expect("left file should be written");
        fs::write(right.path().join("doc.txt"), "a\nright\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);

        presenter.handle_command(UiCommand::SetAiProviderModeOpenAiCompatible);
        presenter.handle_command(UiCommand::LoadAiAnalysis);
        let snapshot = presenter.state_snapshot();
        assert!(snapshot.error_message.is_none());
        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.analysis_error_message.is_some());
        assert!(
            snapshot
                .analysis_error_message
                .as_deref()
                .unwrap_or_default()
                .contains("incomplete")
        );
    }

    #[test]
    fn switching_back_to_mock_after_remote_config_error_restores_analysis() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.rs"), "fn old() {}\n")
            .expect("left file should be written");
        fs::write(right.path().join("doc.rs"), "fn new() {}\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);

        presenter.handle_command(UiCommand::SetAiProviderModeOpenAiCompatible);
        presenter.handle_command(UiCommand::LoadAiAnalysis);
        assert!(presenter.state_snapshot().analysis_error_message.is_some());

        presenter.handle_command(UiCommand::SetAiProviderModeMock);
        presenter.handle_command(UiCommand::LoadAiAnalysis);
        let snapshot = wait_until(&presenter, |state| !state.analysis_loading);
        assert!(snapshot.analysis_error_message.is_none());
        assert!(snapshot.analysis_result.is_some());
    }

    #[test]
    fn analysis_error_does_not_pollute_compare_or_diff_error() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.txt"), "a\nleft\n").expect("left file should be written");
        fs::write(right.path().join("doc.txt"), "a\nright\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadAiAnalysis);

        let snapshot = presenter.state_snapshot();
        assert!(snapshot.error_message.is_none());
        assert!(snapshot.diff_error_message.is_none());
        assert!(snapshot.analysis_error_message.is_some());
    }

    #[test]
    fn rerun_compare_clears_previous_analysis_panel_state() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.txt"), "fn old() {}\n")
            .expect("left file should be written");
        fs::write(right.path().join("doc.txt"), "fn new() {}\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        wait_until(&presenter, |state| !state.diff_loading);
        presenter.handle_command(UiCommand::LoadAiAnalysis);
        wait_until(&presenter, |state| !state.analysis_loading);
        assert!(presenter.state_snapshot().analysis_result.is_some());

        presenter.handle_command(UiCommand::RunCompare);
        let snapshot = wait_until(&presenter, |state| !state.running && !state.diff_loading);
        assert!(snapshot.analysis_result.is_none());
        assert!(snapshot.analysis_error_message.is_none());
        assert!(!snapshot.analysis_loading);
        assert!(snapshot.analysis_available);
        assert_eq!(
            snapshot.analysis_panel_state(),
            crate::state::AnalysisPanelState::Ready
        );
    }

    #[test]
    fn compare_completion_does_not_reset_new_input_value() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("a.txt"), "left\n").expect("left file should be written");
        fs::write(right.path().join("a.txt"), "right\n").expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);

        presenter.handle_command(UiCommand::UpdateLeftRoot(
            "/tmp/user-typing-left".to_string(),
        ));
        let snapshot = wait_until(&presenter, |state| !state.running);
        assert_eq!(snapshot.left_root, "/tmp/user-typing-left");
    }

    #[test]
    fn rerun_compare_prunes_invalid_expanded_paths_and_restores_valid_ones() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::create_dir_all(left.path().join("src/bin"))
            .expect("left nested directory should be created");
        fs::create_dir_all(right.path().join("src/bin"))
            .expect("right nested directory should be created");
        fs::create_dir_all(left.path().join("old")).expect("left old directory should be created");
        fs::create_dir_all(right.path().join("old"))
            .expect("right old directory should be created");
        fs::write(left.path().join("src/bin/main.rs"), "fn old() {}\n")
            .expect("left file should be written");
        fs::write(right.path().join("src/bin/main.rs"), "fn new() {}\n")
            .expect("right file should be written");
        fs::write(left.path().join("old/remove.txt"), "left\n")
            .expect("left old file should be written");
        fs::write(right.path().join("old/remove.txt"), "right\n")
            .expect("right old file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);

        presenter.handle_command(UiCommand::ToggleNavigatorTreeNode("src/bin".to_string()));
        presenter.handle_command(UiCommand::ToggleNavigatorTreeNode("old".to_string()));
        let before_rerun = presenter.state_snapshot();
        assert_eq!(
            before_rerun
                .navigator_tree_expansion_overrides
                .get("src/bin"),
            Some(&true)
        );
        assert_eq!(
            before_rerun.navigator_tree_expansion_overrides.get("old"),
            Some(&false)
        );

        fs::remove_file(left.path().join("old/remove.txt"))
            .expect("left old file should be removed");
        fs::remove_file(right.path().join("old/remove.txt"))
            .expect("right old file should be removed");
        fs::remove_dir(left.path().join("old")).expect("left old dir should be removed");
        fs::remove_dir(right.path().join("old")).expect("right old dir should be removed");

        presenter.handle_command(UiCommand::RunCompare);
        let snapshot = wait_until(&presenter, |state| !state.running);
        assert_eq!(
            snapshot.navigator_tree_expansion_overrides.get("src/bin"),
            Some(&true)
        );
        assert!(
            !snapshot
                .navigator_tree_expansion_overrides
                .contains_key("old")
        );
        assert!(
            snapshot
                .navigator_tree_row_projections()
                .iter()
                .any(|row| row.key == "src/bin/main.rs")
        );
    }

    #[test]
    fn diff_completion_does_not_reset_new_input_value() {
        let left = tempfile::tempdir().expect("left tempdir should be created");
        let right = tempfile::tempdir().expect("right tempdir should be created");
        fs::write(left.path().join("doc.txt"), "a\nleft\n").expect("left file should be written");
        fs::write(right.path().join("doc.txt"), "a\nright\n")
            .expect("right file should be written");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot(left.path().display().to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot(
            right.path().display().to_string(),
        ));
        presenter.handle_command(UiCommand::RunCompare);
        wait_until(&presenter, |state| !state.running);
        presenter.handle_command(UiCommand::SelectRow(0));
        presenter.handle_command(UiCommand::LoadSelectedDiff);

        presenter.handle_command(UiCommand::UpdateRightRoot(
            "/tmp/user-typing-right".to_string(),
        ));
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);
        assert_eq!(snapshot.right_root, "/tmp/user-typing-right");
    }

    #[test]
    fn save_settings_with_invalid_timeout_sets_error() {
        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::SaveAppSettings {
            provider_kind: fc_ai::AiProviderKind::OpenAiCompatible,
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4o-mini".to_string(),
            timeout_secs_text: "0".to_string(),
            show_hidden_files: true,
            default_results_view: NavigatorViewMode::Tree,
        });
        let snapshot = presenter.state_snapshot();
        assert!(snapshot.settings_error_message.is_some());
    }

    #[test]
    fn initialize_loads_settings_from_disk() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let _settings_guard = crate::settings::TestSettingsDirGuard::new(temp.path());
        crate::settings::save_app_preferences(&AppPreferences {
            provider: ProviderSettings {
                provider_kind: fc_ai::AiProviderKind::OpenAiCompatible,
                openai_endpoint: "https://api.example.com/v1".to_string(),
                openai_api_key: "sk-test".to_string(),
                openai_model: "gpt-4o-mini".to_string(),
                timeout_secs: 55,
            },
            behavior: BehaviorSettings {
                show_hidden_files: false,
                default_results_view: DefaultResultsView::Flat,
            },
        })
        .expect("settings should be saved");

        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::Initialize);
        let snapshot = presenter.state_snapshot();
        assert_eq!(
            snapshot.analysis_provider_kind,
            fc_ai::AiProviderKind::OpenAiCompatible
        );
        assert_eq!(
            snapshot.analysis_openai_endpoint,
            "https://api.example.com/v1"
        );
        assert_eq!(snapshot.analysis_openai_api_key, "sk-test");
        assert_eq!(snapshot.analysis_openai_model, "gpt-4o-mini");
        assert_eq!(snapshot.analysis_request_timeout_secs, 55);
        assert!(!snapshot.show_hidden_files);
        assert_eq!(
            snapshot.default_navigator_view_mode,
            NavigatorViewMode::Flat
        );
        assert_eq!(
            snapshot.navigator_runtime_view_mode,
            NavigatorViewMode::Flat
        );
    }
}
