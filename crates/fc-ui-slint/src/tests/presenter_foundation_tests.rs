use super::*;
use crate::compare_foundation::CompareFocusPath;
use crate::state::WorkspaceMode;
use crate::tests::fixtures::{directory_entry, file_entry, state_from_entries, text_diff_entry};
use fc_core::EntryStatus;
use std::sync::{Arc, Mutex};

fn presenter_from_state(state: AppState) -> Presenter {
    Presenter::new(Arc::new(Mutex::new(state)))
}

#[test]
fn switching_to_tree_marks_directory_selection_stale() {
    let mut state = state_from_entries(vec![
        directory_entry("src", EntryStatus::Equal),
        text_diff_entry("src/main.rs", EntryStatus::Different),
    ]);
    state.selected_row = state.row_index_for_relative_path("src");
    state.selected_relative_path = Some("src".to_string());
    state.navigator_runtime_view_mode = NavigatorViewMode::Flat;

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SetNavigatorViewModeTree);
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.selected_row, None);
    assert_eq!(snapshot.selected_relative_path.as_deref(), Some("src"));
    assert_eq!(
        snapshot.diff_shell_state(),
        crate::state::DiffShellState::StaleSelection
    );
}

#[test]
fn switching_to_tree_reveals_selected_file_and_keeps_file_view_open() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    let selected_index = state
        .row_index_for_relative_path("src/bin/main.rs")
        .expect("nested file row should exist");
    state.selected_row = Some(selected_index);
    state.selected_relative_path = Some("src/bin/main.rs".to_string());
    state.selected_diff = Some(crate::view_models::DiffPanelViewModel::default());
    state.navigator_runtime_view_mode = NavigatorViewMode::Flat;
    state.navigator_tree_expansion_overrides =
        std::collections::BTreeMap::from([("src/bin".to_string(), false)]);

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SetNavigatorViewModeTree);
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.selected_row, Some(selected_index));
    assert_eq!(
        snapshot.selected_relative_path.as_deref(),
        Some("src/bin/main.rs")
    );
    assert!(snapshot.selected_diff.is_some());
    assert_eq!(
        snapshot.navigator_runtime_view_mode,
        NavigatorViewMode::Tree
    );
    assert!(
        snapshot
            .navigator_tree_row_projections()
            .iter()
            .any(|row| row.key == "src/bin/main.rs")
    );
    assert_eq!(
        snapshot.navigator_tree_expansion_overrides.get("src/bin"),
        Some(&true)
    );
    assert_eq!(snapshot.navigator_tree_scroll_request_revision, 1);
    assert_eq!(
        snapshot.navigator_tree_scroll_target_source_index,
        Some(selected_index)
    );
}

#[test]
fn switching_to_flat_requests_scroll_for_visible_selected_row() {
    let mut state = state_from_entries(vec![
        file_entry("src/lib.rs", EntryStatus::Equal),
        text_diff_entry("src/main.rs", EntryStatus::Different),
    ]);
    let selected_index = state
        .row_index_for_relative_path("src/main.rs")
        .expect("main row should exist");
    state.selected_row = Some(selected_index);
    state.selected_relative_path = Some("src/main.rs".to_string());
    state.navigator_runtime_view_mode = NavigatorViewMode::Tree;

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SetNavigatorViewModeFlat);
    let snapshot = presenter.state_snapshot();
    assert_eq!(
        snapshot.navigator_runtime_view_mode,
        NavigatorViewMode::Flat
    );
    assert_eq!(snapshot.selected_row, Some(selected_index));
    assert_eq!(snapshot.navigator_flat_scroll_request_revision, 1);
    assert_eq!(
        snapshot.navigator_flat_scroll_target_source_index,
        Some(selected_index)
    );
}

#[test]
fn save_settings_hiding_selected_row_marks_it_stale() {
    let mut state = state_from_entries(vec![
        file_entry(".env", EntryStatus::Different),
        text_diff_entry("src/main.rs", EntryStatus::Different),
    ]);
    state.selected_row = state.row_index_for_relative_path(".env");
    state.selected_relative_path = Some(".env".to_string());
    state.show_hidden_files = true;

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SaveAppSettings {
        provider_kind: fc_ai::AiProviderKind::Mock,
        endpoint: String::new(),
        api_key: String::new(),
        model: "gpt-4o-mini".to_string(),
        timeout_secs_text: "30".to_string(),
        show_hidden_files: false,
        default_results_view: NavigatorViewMode::Tree,
    });

    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.selected_row, None);
    assert_eq!(snapshot.selected_relative_path.as_deref(), Some(".env"));
    assert!(!snapshot.show_hidden_files);
}

#[test]
fn selecting_file_row_switches_workspace_mode_to_file_view_without_resetting_compare_focus() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    let selected_index = state
        .row_index_for_relative_path("src/bin/main.rs")
        .expect("file row should exist");
    state.set_workspace_mode(WorkspaceMode::CompareView);
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SelectRow(selected_index as i32));
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::FileView);
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(
        snapshot.selected_relative_path.as_deref(),
        Some("src/bin/main.rs")
    );
    assert_eq!(snapshot.selected_row, Some(selected_index));
}
