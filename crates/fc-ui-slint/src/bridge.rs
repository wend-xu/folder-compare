//! Bridge between UI event handlers and presenter logic.

use crate::commands::UiCommand;
use crate::presenter::Presenter;

/// Thin bridge for wiring command dispatch.
#[derive(Clone)]
pub struct UiBridge {
    presenter: Presenter,
}

impl UiBridge {
    /// Creates a new bridge.
    pub fn new(presenter: Presenter) -> Self {
        Self { presenter }
    }

    /// Dispatches a command to the presenter.
    pub fn dispatch(&self, command: UiCommand) {
        self.presenter.handle_command(command);
    }
}
