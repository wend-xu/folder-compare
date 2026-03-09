//! Presenter layer placeholders.

use crate::commands::UiCommand;
use crate::state::AppState;
use crate::view_models::MainViewModel;
use std::sync::{Arc, Mutex};

/// Presenter that maps state into view models.
#[derive(Clone)]
pub struct Presenter {
    state: Arc<Mutex<AppState>>,
}

impl Presenter {
    /// Creates a presenter from state.
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self { state }
    }

    /// Produces the current main view model.
    pub fn view_model(&self) -> MainViewModel {
        let state = self.state.lock().expect("state mutex poisoned");
        MainViewModel {
            title: "Folder Compare".to_string(),
            subtitle: if state.status_text.is_empty() {
                "Phase 1 skeleton".to_string()
            } else {
                state.status_text.clone()
            },
        }
    }

    /// Handles a command with placeholder behavior.
    pub fn handle_command(&self, command: UiCommand) {
        let mut state = self.state.lock().expect("state mutex poisoned");
        match command {
            UiCommand::Initialize => state.status_text = "Ready".to_string(),
            UiCommand::RunComparePlaceholder => {
                state.status_text = "Compare action is not implemented in Phase 1".to_string();
            }
        }
    }
}
