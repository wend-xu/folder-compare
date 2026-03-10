//! Presenter layer for compare, filtering, and detailed diff orchestration.

use crate::bridge;
use crate::commands::UiCommand;
use crate::commands::{run_ai_analysis, run_compare, run_text_diff};
use crate::state::AppState;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const BACKGROUND_START_DELAY: Duration = Duration::from_millis(10);

/// Presenter that manages compare-oriented UI state.
#[derive(Clone)]
pub struct Presenter {
    state: Arc<Mutex<AppState>>,
}

impl Presenter {
    /// Creates a presenter from state.
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self { state }
    }

    /// Returns a snapshot copy of current app state.
    pub fn state_snapshot(&self) -> AppState {
        self.state.lock().expect("state mutex poisoned").clone()
    }

    /// Handles one UI command.
    pub fn handle_command(&self, command: UiCommand) {
        match command {
            UiCommand::Initialize => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.status_text = "Ready".to_string();
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
                state.entry_filter = filter;
                Self::reconcile_selected_row_visibility(&mut state);
            }
            UiCommand::UpdateEntryStatusFilter(filter) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.set_entry_status_filter(&filter);
                Self::reconcile_selected_row_visibility(&mut state);
            }
            UiCommand::SelectRow(index) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.selected_row = usize::try_from(index)
                    .ok()
                    .filter(|value| *value < state.entry_rows.len());
                let selected_row_vm = state
                    .selected_row
                    .and_then(|value| state.entry_rows.get(value))
                    .cloned();
                state.selected_relative_path = selected_row_vm
                    .as_ref()
                    .map(|row| row.relative_path.clone());
                state.clear_diff_panel();
                state.analysis_available = false;
                state.clear_analysis_panel();
                state.analysis_hint = Some(match selected_row_vm {
                    Some(row) if !row.can_load_analysis => row
                        .analysis_blocked_reason
                        .unwrap_or_else(|| "selected row does not support AI analysis".to_string()),
                    Some(_) => "Load detailed diff, then click Analyze.".to_string(),
                    None => "Select one changed text file to analyze.".to_string(),
                });
            }
            UiCommand::LoadSelectedDiff => self.execute_load_selected_diff(),
            UiCommand::LoadAiAnalysis => self.execute_load_ai_analysis(),
            UiCommand::SetAiProviderModeMock => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_provider_kind = fc_ai::AiProviderKind::Mock;
                state.clear_analysis_panel();
                state.analysis_hint =
                    Some("Using mock provider. No remote request will be sent.".to_string());
            }
            UiCommand::SetAiProviderModeOpenAiCompatible => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_provider_kind = fc_ai::AiProviderKind::OpenAiCompatible;
                state.clear_analysis_panel();
                state.analysis_hint =
                    Some("Using remote provider. Configure endpoint/api key/model.".to_string());
            }
            UiCommand::UpdateAiEndpoint(value) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_openai_endpoint = value;
                state.analysis_error_message = None;
            }
            UiCommand::UpdateAiApiKey(value) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_openai_api_key = value;
                state.analysis_error_message = None;
            }
            UiCommand::UpdateAiModel(value) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.analysis_openai_model = value;
                state.analysis_error_message = None;
            }
        }
    }

    fn execute_compare(&self) {
        let (left_root, right_root, state_ref) = {
            let mut state = self.state.lock().expect("state mutex poisoned");
            if state.running {
                return;
            }
            state.running = true;
            state.error_message = None;
            state.status_text = "Comparing...".to_string();
            state.selected_row = None;
            state.selected_relative_path = None;
            state.clear_diff_panel();
            state.analysis_available = false;
            state.clear_analysis_panel();
            state.analysis_hint = Some("Select one changed text file to analyze.".to_string());
            (
                state.left_root.clone(),
                state.right_root.clone(),
                Arc::clone(&self.state),
            )
        };

        thread::spawn(move || {
            // Give UI one short frame to render loading state before heavy work.
            thread::sleep(BACKGROUND_START_DELAY);

            let result = bridge::build_compare_request(&left_root, &right_root)
                .and_then(run_compare)
                .map(bridge::map_compare_report);

            let mut state = state_ref.lock().expect("state mutex poisoned");
            state.running = false;
            match result {
                Ok(vm) => {
                    let count = vm.entry_rows.len();
                    state.summary_text = vm.summary_text;
                    state.entry_rows = vm.entry_rows;
                    state.warning_lines = vm.warnings;
                    state.truncated = vm.truncated;
                    state.error_message = None;
                    state.status_text = format!("Compare finished: {} entries", count);
                }
                Err(message) => {
                    state.summary_text.clear();
                    state.entry_rows.clear();
                    state.warning_lines.clear();
                    state.truncated = false;
                    state.error_message = Some(message);
                    state.status_text = "Compare failed".to_string();
                }
            }
        });
    }

    fn execute_load_selected_diff(&self) {
        let (left_root, right_root, selected_row, state_ref) = {
            let mut state = self.state.lock().expect("state mutex poisoned");
            if state.diff_loading {
                return;
            }
            state.diff_loading = true;
            state.diff_error_message = None;
            state.selected_diff = None;
            state.diff_warning = None;
            state.diff_truncated = false;
            state.analysis_available = false;
            state.clear_analysis_panel();
            state.analysis_hint = Some("Detailed diff is loading...".to_string());
            (
                state.left_root.clone(),
                state.right_root.clone(),
                state
                    .selected_row
                    .and_then(|idx| state.entry_rows.get(idx).cloned()),
                Arc::clone(&self.state),
            )
        };

        thread::spawn(move || {
            // Give UI one short frame to render loading state before heavy work.
            thread::sleep(BACKGROUND_START_DELAY);

            let result = selected_row
                .ok_or_else(|| "select one compare row before loading detailed diff".to_string())
                .and_then(|row| {
                    let relative_path = row.relative_path.clone();
                    bridge::build_text_diff_request(&left_root, &right_root, &row)
                        .and_then(run_text_diff)
                        .map(|diff_result| {
                            (
                                row,
                                bridge::map_text_diff_result(&relative_path, diff_result),
                            )
                        })
                });

            let mut state = state_ref.lock().expect("state mutex poisoned");
            state.diff_loading = false;
            match result {
                Ok((row, diff_vm)) => {
                    state.selected_relative_path = Some(diff_vm.relative_path.clone());
                    state.diff_warning = diff_vm.warning.clone();
                    state.diff_truncated = diff_vm.truncated;
                    state.diff_error_message = None;
                    state.selected_diff = Some(diff_vm);
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
                    state.status_text = "Detailed diff loaded".to_string();
                }
                Err(message) => {
                    state.diff_error_message = Some(message);
                    state.selected_diff = None;
                    state.diff_warning = None;
                    state.diff_truncated = false;
                    state.analysis_available = false;
                    state.clear_analysis_panel();
                    state.analysis_hint =
                        Some("Detailed diff is unavailable; AI analysis is disabled.".to_string());
                    state.status_text = "Detailed diff unavailable".to_string();
                }
            }
        });
    }

    fn execute_load_ai_analysis(&self) {
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
                return;
            };
            if !row.can_load_analysis {
                state.analysis_error_message =
                    Some(row.analysis_blocked_reason.clone().unwrap_or_else(|| {
                        "selected row does not support AI analysis".to_string()
                    }));
                return;
            }
            if state.diff_loading {
                state.analysis_error_message =
                    Some("wait until detailed diff loading completes".to_string());
                return;
            }
            if state.selected_diff.is_none() {
                state.analysis_error_message =
                    Some("load detailed diff before running AI analysis".to_string());
                return;
            }
            if state.analysis_remote_mode() && !state.analysis_remote_config_ready() {
                state.analysis_error_message = Some(
                    "remote provider configuration is incomplete (endpoint/api key/model required)"
                        .to_string(),
                );
                return;
            }

            state.analysis_loading = true;
            state.analysis_error_message = None;
            state.analysis_result = None;
            state.analysis_hint = Some(format!(
                "Running AI analysis with {} provider...",
                state.analysis_provider_mode_text()
            ));
            (
                selected_row,
                state.selected_diff.clone(),
                state.diff_warning.clone(),
                state.diff_truncated,
                state.analysis_ai_config(),
                Arc::clone(&self.state),
            )
        };

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
                }
                Err(message) => {
                    state.analysis_error_message = Some(message);
                    state.analysis_result = None;
                    state.status_text = "AI analysis unavailable".to_string();
                }
            }
        });
    }

    fn reconcile_selected_row_visibility(state: &mut AppState) {
        if let Some(selected_row) = state.selected_row {
            if !state.is_row_visible_in_filter(selected_row) {
                state.selected_row = None;
                state.selected_relative_path = None;
                state.clear_diff_panel();
                state.analysis_available = false;
                state.clear_analysis_panel();
                state.analysis_hint = Some("Select one changed text file to analyze.".to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        assert!(snapshot
            .entry_rows
            .iter()
            .any(|row| row.relative_path.contains("a.txt")));
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
        assert!(snapshot.selected_relative_path.is_none());
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
        assert!(snapshot.selected_relative_path.is_none());
        assert_eq!(snapshot.entry_status_filter, "equal");
        assert_eq!(snapshot.filtered_entry_rows_with_index().len(), 1);
    }

    #[test]
    fn load_selected_diff_for_non_diffable_row_sets_diff_error_only() {
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
        presenter.handle_command(UiCommand::LoadSelectedDiff);
        let snapshot = wait_until(&presenter, |state| !state.diff_loading);

        assert!(snapshot.error_message.is_none());
        assert!(snapshot.diff_error_message.is_some());
        assert!(snapshot.selected_diff.is_none());
    }

    #[test]
    fn rerun_compare_clears_previous_diff_panel_state() {
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
        let snapshot = wait_until(&presenter, |state| !state.running);
        assert!(snapshot.selected_diff.is_none());
        assert!(snapshot.diff_error_message.is_none());
        assert!(!snapshot.diff_loading);
        assert_eq!(snapshot.entry_filter, "doc");
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
        assert!(snapshot
            .analysis_error_message
            .as_deref()
            .unwrap_or_default()
            .contains("incomplete"));
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
        let snapshot = wait_until(&presenter, |state| !state.running);
        assert!(snapshot.analysis_result.is_none());
        assert!(snapshot.analysis_error_message.is_none());
        assert!(!snapshot.analysis_loading);
        assert!(!snapshot.analysis_available);
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
}
