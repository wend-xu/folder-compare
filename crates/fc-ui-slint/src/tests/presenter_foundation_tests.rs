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
        auto_locate_current_file_on_compare_return: true,
        lock_compare_horizontal_scrolling_by_default: true,
    });

    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.selected_row, None);
    assert_eq!(snapshot.selected_relative_path.as_deref(), Some(".env"));
    assert!(!snapshot.show_hidden_files);
}

#[test]
fn save_settings_compare_scroll_lock_default_does_not_modify_current_tab_lock() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    assert!(state.set_lock_compare_horizontal_scrolling_by_default(false));
    state.ensure_compare_tree_session();
    assert!(state.toggle_compare_view_horizontal_scroll_locked());
    assert!(state.compare_view_horizontal_scroll_locked());

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SaveAppSettings {
        provider_kind: fc_ai::AiProviderKind::Mock,
        endpoint: String::new(),
        api_key: String::new(),
        model: "gpt-4o-mini".to_string(),
        timeout_secs_text: "30".to_string(),
        show_hidden_files: true,
        default_results_view: NavigatorViewMode::Tree,
        auto_locate_current_file_on_compare_return: true,
        lock_compare_horizontal_scrolling_by_default: false,
    });

    let snapshot = presenter.state_snapshot();
    assert!(!snapshot.lock_compare_horizontal_scrolling_by_default());
    assert!(snapshot.compare_view_horizontal_scroll_locked());
    assert_eq!(
        snapshot
            .compare_tree_session
            .as_ref()
            .map(|session| session.horizontal_scroll_locked),
        Some(true)
    );
}

#[test]
fn selecting_sidebar_file_row_with_compare_session_requests_standard_file_view_confirmation() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    let selected_index = state
        .row_index_for_relative_path("src/bin/main.rs")
        .expect("file row should exist");
    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SelectRow(selected_index as i32));
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(snapshot.active_session_id.as_deref(), Some("compare-tree"));
    assert!(snapshot.workspace_session_confirmation_open());
    assert_eq!(
        snapshot.workspace_session_confirmation_title_text(),
        "Open standard File View and close current Compare session?"
    );
}

#[test]
fn confirming_sidebar_file_row_transition_closes_compare_session_and_loads_standard_file_view() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    let selected_index = state
        .row_index_for_relative_path("src/bin/main.rs")
        .expect("file row should exist");
    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::SelectRow(selected_index as i32));
    presenter.handle_command(UiCommand::ConfirmWorkspaceSessionAction);
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::FileView);
    assert!(!snapshot.has_compare_tree_session());
    assert!(!snapshot.workspace_sessions_visible());
    assert_eq!(snapshot.selected_row, Some(selected_index));
    assert_eq!(
        snapshot.selected_relative_path.as_deref(),
        Some("src/bin/main.rs")
    );
    assert!(snapshot.diff_loading || snapshot.selected_diff.is_some());
}

#[test]
fn opening_compare_view_switches_workspace_mode_without_clearing_file_selection() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    let selected_index = state
        .row_index_for_relative_path("src/main.rs")
        .expect("file row should exist");
    state.selected_row = Some(selected_index);
    state.selected_relative_path = Some("src/main.rs".to_string());

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenCompareView("src".to_string()));
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert!(snapshot.has_compare_tree_session());
    assert_eq!(snapshot.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(
        snapshot.compare_row_focus_path.as_deref(),
        Some("src/main.rs")
    );
    assert_eq!(snapshot.selected_row, Some(selected_index));
    assert_eq!(
        snapshot.selected_relative_path.as_deref(),
        Some("src/main.rs")
    );
}

#[test]
fn opening_compare_view_at_compare_root_is_allowed() {
    let state = state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenCompareView(String::new()));
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert!(snapshot.has_compare_tree_session());
    assert_eq!(snapshot.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(snapshot.compare_focus_path_raw_text(), "");
    assert_eq!(
        snapshot.compare_view_breadcrumb_labels(),
        vec!["Compare Root"]
    );
}

#[test]
fn opening_file_view_from_compare_preserves_compare_return_context() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert_eq!(state.compare_row_focus_path.as_deref(), Some("src/main.rs"));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenFileViewFromCompare(
        "src/main.rs".to_string(),
    ));
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::FileView);
    assert!(snapshot.can_return_to_compare_view);
    assert_eq!(
        snapshot.active_session_id.as_deref(),
        Some("file:src/main.rs")
    );
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(
        snapshot.compare_row_focus_path.as_deref(),
        Some("src/main.rs")
    );
    assert_eq!(
        snapshot.selected_relative_path.as_deref(),
        Some("src/main.rs")
    );
}

#[test]
fn selecting_compare_tree_session_restores_compare_mode() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert_eq!(state.compare_row_focus_path.as_deref(), Some("src/main.rs"));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenFileViewFromCompare(
        "src/main.rs".to_string(),
    ));
    presenter.handle_command(UiCommand::SelectWorkspaceSession(
        "compare-tree".to_string(),
    ));
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(
        snapshot.compare_row_focus_path.as_deref(),
        Some("src/main.rs")
    );
}

#[test]
fn selecting_compare_tree_session_auto_locates_current_compare_file() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);
    state.ensure_compare_tree_session();
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert!(state.set_compare_row_focus_path(Some("src/main.rs")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenFileViewFromCompare("src/lib.rs".to_string()));
    presenter.handle_command(UiCommand::SelectWorkspaceSession(
        "compare-tree".to_string(),
    ));

    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(
        snapshot.compare_row_focus_path.as_deref(),
        Some("src/lib.rs")
    );
    assert_eq!(
        snapshot.compare_view_scroll_target_relative_path.as_deref(),
        Some("src/lib.rs")
    );
}

#[test]
fn reveal_current_file_in_compare_tree_ignores_disabled_auto_locate_setting() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);
    state.ensure_compare_tree_session();
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert!(state.set_compare_row_focus_path(Some("src/main.rs")));
    assert!(state.set_auto_locate_current_file_on_compare_return(false));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenFileViewFromCompare("src/lib.rs".to_string()));
    presenter.handle_command(UiCommand::RevealCurrentFileInCompareTree);

    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(
        snapshot.compare_row_focus_path.as_deref(),
        Some("src/lib.rs")
    );
}

#[test]
fn toggling_sidebar_visibility_does_not_disturb_compare_return_context() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenFileViewFromCompare(
        "src/main.rs".to_string(),
    ));
    presenter.handle_command(UiCommand::ToggleSidebarVisibility);
    let snapshot = presenter.state_snapshot();
    assert!(!snapshot.sidebar_visible());
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::FileView);
    assert!(snapshot.can_return_to_compare_view);
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
}

#[test]
fn closing_active_compare_file_session_auto_locates_on_return() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);
    state.ensure_compare_tree_session();
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert!(state.set_compare_row_focus_path(Some("src/main.rs")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenFileViewFromCompare("src/lib.rs".to_string()));
    presenter.handle_command(UiCommand::CloseWorkspaceSession(
        "file:src/lib.rs".to_string(),
    ));

    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(
        snapshot.compare_row_focus_path.as_deref(),
        Some("src/lib.rs")
    );
}

#[test]
fn compare_view_up_one_level_focuses_previous_child_directory() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        file_entry("src/lib.rs", EntryStatus::Equal),
    ]);
    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src/bin")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::CompareViewUpOneLevel);
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(snapshot.compare_row_focus_path.as_deref(), Some("src/bin"));
}

#[test]
fn breadcrumb_navigation_reanchors_without_resetting_compare_session() {
    let state = state_from_entries(vec![
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        file_entry("src/lib.rs", EntryStatus::Equal),
    ]);

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenCompareView("src/bin".to_string()));
    presenter.handle_command(UiCommand::OpenFileViewFromCompare(
        "src/bin/main.rs".to_string(),
    ));
    presenter.handle_command(UiCommand::NavigateCompareView("src".to_string()));

    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(snapshot.compare_row_focus_path.as_deref(), Some("src/bin"));
    assert_eq!(
        snapshot.workspace_session_ids(),
        vec!["compare-tree", "file:src/bin/main.rs"]
    );
}

#[test]
fn toggling_compare_tree_node_expands_directory_without_reanchoring_compare_view() {
    let mut state = state_from_entries(vec![
        directory_entry("src/bin", EntryStatus::Different),
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        text_diff_entry("src/root.rs", EntryStatus::Different),
    ]);
    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::ToggleCompareTreeNode("src/bin".to_string()));
    let snapshot = presenter.state_snapshot();
    assert_eq!(snapshot.workspace_mode, WorkspaceMode::CompareView);
    assert_eq!(snapshot.compare_focus_path_raw_text(), "src");
    assert_eq!(snapshot.compare_row_focus_path.as_deref(), Some("src/bin"));
    assert_eq!(
        snapshot.compare_view_expansion_overrides.get("src/bin"),
        Some(&true)
    );
    assert!(
        snapshot
            .compare_view_row_projections()
            .iter()
            .any(|row| row.relative_path == "src/bin/main.rs")
    );
}

#[test]
fn closing_compare_tree_session_with_file_tabs_requires_confirmation() {
    let state = state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenCompareView("src".to_string()));
    presenter.handle_command(UiCommand::OpenFileViewFromCompare(
        "src/main.rs".to_string(),
    ));

    presenter.handle_command(UiCommand::CloseWorkspaceSession("compare-tree".to_string()));
    let pending_snapshot = presenter.state_snapshot();
    assert!(pending_snapshot.workspace_session_confirmation_open());
    assert!(pending_snapshot.workspace_sessions_visible());

    presenter.handle_command(UiCommand::ConfirmWorkspaceSessionAction);
    let final_snapshot = presenter.state_snapshot();
    assert!(!final_snapshot.workspace_sessions_visible());
    assert!(!final_snapshot.workspace_session_confirmation_open());
}

#[test]
fn opening_compare_view_with_existing_file_tabs_requests_reset_confirmation() {
    let state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("docs/guide.md", EntryStatus::Different),
    ]);
    let presenter = presenter_from_state(state);
    presenter.handle_command(UiCommand::OpenCompareView("src".to_string()));
    presenter.handle_command(UiCommand::OpenFileViewFromCompare(
        "src/main.rs".to_string(),
    ));
    presenter.handle_command(UiCommand::OpenFileViewFromCompare(
        "docs/guide.md".to_string(),
    ));

    presenter.handle_command(UiCommand::OpenCompareView("docs".to_string()));
    let pending_snapshot = presenter.state_snapshot();
    assert!(pending_snapshot.workspace_session_confirmation_open());
    assert_eq!(
        pending_snapshot.workspace_session_confirmation_title_text(),
        "Reset Compare session?"
    );

    presenter.handle_command(UiCommand::ConfirmWorkspaceSessionAction);
    let final_snapshot = presenter.state_snapshot();
    assert!(final_snapshot.has_compare_tree_session());
    assert_eq!(final_snapshot.workspace_session_ids(), vec!["compare-tree"]);
    assert_eq!(final_snapshot.compare_focus_path_raw_text(), "docs");
    assert_eq!(final_snapshot.workspace_mode, WorkspaceMode::CompareView);
}
