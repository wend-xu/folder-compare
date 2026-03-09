//! View model definitions for Slint binding.

/// Main window view model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainViewModel {
    /// Main title string.
    pub title: String,
    /// Secondary subtitle string.
    pub subtitle: String,
}
