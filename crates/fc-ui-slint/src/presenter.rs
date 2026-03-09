//! Presenter layer for MVP compare orchestration.

use crate::bridge;
use crate::commands::run_compare;
use crate::commands::UiCommand;
use crate::state::AppState;
use std::sync::{Arc, Mutex};

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
            UiCommand::SelectRow(index) => {
                let mut state = self.state.lock().expect("state mutex poisoned");
                state.selected_row = usize::try_from(index).ok();
            }
        }
    }

    fn execute_compare(&self) {
        let (left_root, right_root) = {
            let mut state = self.state.lock().expect("state mutex poisoned");
            state.running = true;
            state.error_message = None;
            state.status_text = "Comparing...".to_string();
            (state.left_root.clone(), state.right_root.clone())
        };

        let result = bridge::build_compare_request(&left_root, &right_root)
            .and_then(run_compare)
            .map(bridge::map_compare_report);

        let mut state = self.state.lock().expect("state mutex poisoned");
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn run_compare_with_invalid_input_sets_error() {
        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::UpdateLeftRoot("".to_string()));
        presenter.handle_command(UiCommand::UpdateRightRoot("".to_string()));
        presenter.handle_command(UiCommand::RunCompare);

        let snapshot = presenter.state_snapshot();
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

        let snapshot = presenter.state_snapshot();
        assert!(!snapshot.running);
        assert!(snapshot.error_message.is_none());
        assert!(!snapshot.summary_text.is_empty());
        assert!(!snapshot.entry_rows.is_empty());
        assert!(snapshot.status_text.contains("Compare finished"));
        assert!(snapshot
            .entry_display_lines()
            .iter()
            .any(|line| line.contains("a.txt")));
    }

    #[test]
    fn select_row_updates_state() {
        let presenter = Presenter::new(Arc::new(Mutex::new(AppState::default())));
        presenter.handle_command(UiCommand::SelectRow(3));
        assert_eq!(presenter.state_snapshot().selected_row, Some(3));
        presenter.handle_command(UiCommand::SelectRow(-1));
        assert_eq!(presenter.state_snapshot().selected_row, None);
    }
}
