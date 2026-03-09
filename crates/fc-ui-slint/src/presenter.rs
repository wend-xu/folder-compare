//! Presenter layer for compare, filtering, and detailed diff orchestration.

use crate::bridge;
use crate::commands::UiCommand;
use crate::commands::{run_compare, run_text_diff};
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
                if let Some(selected_row) = state.selected_row {
                    if !state.is_row_visible_in_filter(selected_row) {
                        state.selected_row = None;
                        state.selected_relative_path = None;
                        state.clear_diff_panel();
                    }
                }
            }
            UiCommand::SelectRow(index) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.selected_row = usize::try_from(index)
                    .ok()
                    .filter(|value| *value < state.entry_rows.len());
                state.selected_relative_path = state
                    .selected_row
                    .and_then(|value| state.entry_rows.get(value))
                    .map(|row| row.relative_path.clone());
                state.clear_diff_panel();
            }
            UiCommand::LoadSelectedDiff => self.execute_load_selected_diff(),
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
                            bridge::map_text_diff_result(&relative_path, diff_result)
                        })
                });

            let mut state = state_ref.lock().expect("state mutex poisoned");
            state.diff_loading = false;
            match result {
                Ok(diff_vm) => {
                    state.selected_relative_path = Some(diff_vm.relative_path.clone());
                    state.diff_warning = diff_vm.warning.clone();
                    state.diff_truncated = diff_vm.truncated;
                    state.diff_error_message = None;
                    state.selected_diff = Some(diff_vm);
                    state.status_text = "Detailed diff loaded".to_string();
                }
                Err(message) => {
                    state.diff_error_message = Some(message);
                    state.selected_diff = None;
                    state.diff_warning = None;
                    state.diff_truncated = false;
                    state.status_text = "Detailed diff unavailable".to_string();
                }
            }
        });
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
        presenter.handle_command(UiCommand::SelectRow(0));
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
}
