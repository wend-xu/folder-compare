use super::*;
use crate::compare_foundation::CompareFocusPath;
use crate::tests::fixtures::{
    directory_entry, file_comparison_entry, file_entry, state_from_entries, text_diff_entry,
};
use fc_core::{CompareEntry, EntryDetail, EntryKind, EntryStatus};

fn sample_entries() -> Vec<CompareEntry> {
    vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        file_comparison_entry("assets/logo.png", EntryStatus::Different, 10, 12, true),
    ]
}

#[test]
fn empty_filter_returns_all_rows() {
    let state = state_from_entries(sample_entries());
    let filtered = state.filtered_entry_rows_with_index();
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].0, 0);
    assert_eq!(filtered[1].0, 1);
}

#[test]
fn non_empty_filter_matches_path_or_name_only() {
    let mut state = state_from_entries(sample_entries());
    state.entry_filter = "logo".to_string();
    let filtered = state.filtered_entry_rows_with_index();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, 1);

    let mut state = state_from_entries(sample_entries());
    state.entry_filter = "main.rs".to_string();
    let filtered = state.filtered_entry_rows_with_index();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, 0);

    let mut state = state_from_entries(sample_entries());
    state.entry_filter = "text summary".to_string();
    let filtered = state.filtered_entry_rows_with_index();
    assert!(filtered.is_empty());
}

#[test]
fn status_filter_reduces_visible_rows() {
    let state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        file_comparison_entry("assets/logo.png", EntryStatus::Different, 10, 12, true),
        file_entry("docs/guide.md", EntryStatus::Equal),
    ]);
    let mut state = state;
    state.entry_status_filter = "equal".to_string();

    let filtered = state.filtered_entry_rows_with_index();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.status, "equal");
}

#[test]
fn results_collection_text_tracks_search_and_scope() {
    let mut state = state_from_entries(sample_entries());
    assert_eq!(
        state.results_collection_text(),
        "Showing 2 / 2 · All results"
    );

    state.entry_filter = "logo".to_string();
    state.set_entry_status_filter("different");
    let text = state.results_collection_text();
    assert_eq!(text, "Showing 1 / 2 · Search: \"logo\" · Diff");
}

#[test]
fn hidden_files_preference_filters_dot_prefixed_entries() {
    let entries = vec![
        file_entry(".gitignore", EntryStatus::Different),
        file_entry("src/main.rs", EntryStatus::Different),
        file_entry("assets/.cache/logo.png", EntryStatus::Different),
    ];

    let hidden = state_from_entries(entries.clone());
    let mut hidden = hidden;
    hidden.show_hidden_files = false;
    let visible = hidden.filtered_entry_rows_with_index();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].1.relative_path, "src/main.rs");

    let mut shown = state_from_entries(entries);
    shown.show_hidden_files = true;
    assert_eq!(shown.filtered_entry_rows_with_index().len(), 3);
}

#[test]
fn workspace_mode_and_compare_focus_are_independent_from_file_selection() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        file_entry("src/lib.rs", EntryStatus::Equal),
    ]);

    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(state.workspace_mode_text(), "compare-view");

    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src/bin/main.rs")));
    assert_eq!(state.compare_focus_path_raw_text(), "src/bin");

    state.selected_row = state.row_index_for_relative_path("src/bin/main.rs");
    state.selected_relative_path = Some("src/bin/main.rs".to_string());
    assert!(state.focus_compare_parent());
    assert_eq!(state.compare_focus_path_raw_text(), "src");

    assert!(state.reset_compare_focus_path());
    assert_eq!(state.compare_focus_path_raw_text(), "");
    assert_eq!(
        state.selected_relative_path.as_deref(),
        Some("src/bin/main.rs")
    );
}

#[test]
fn compare_session_reset_accepts_compare_root_target() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        file_entry("src/lib.rs", EntryStatus::Equal),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.request_compare_session_reset("", None));
    assert_eq!(state.workspace_mode_text(), "compare-view");
    assert_eq!(state.compare_focus_path_raw_text(), "");
    assert_eq!(state.compare_view_breadcrumb_labels(), vec!["Compare Root"]);
    assert_eq!(state.compare_view_breadcrumb_paths(), vec![""]);
}

#[test]
fn compare_view_breadcrumbs_follow_compare_focus_path() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        file_entry("src/lib.rs", EntryStatus::Equal),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src/bin")));

    assert_eq!(
        state.compare_view_breadcrumb_labels(),
        vec!["Compare Root", "src", "bin"]
    );
    assert_eq!(
        state.compare_view_breadcrumb_paths(),
        vec!["", "src", "src/bin"]
    );
}

#[test]
fn compare_tree_horizontal_scroll_lock_defaults_to_locked_and_toggles() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);

    state.ensure_compare_tree_session();
    assert!(state.compare_view_horizontal_scroll_locked());
    assert!(state.toggle_compare_view_horizontal_scroll_locked());
    assert!(!state.compare_view_horizontal_scroll_locked());
    assert!(state.set_compare_view_horizontal_scroll_locked(true));
    assert!(state.compare_view_horizontal_scroll_locked());
}

#[test]
fn compare_horizontal_scroll_default_applies_to_new_compare_sessions() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    assert!(state.set_lock_compare_horizontal_scrolling_by_default(false));

    state.ensure_compare_tree_session();
    assert!(!state.compare_view_horizontal_scroll_locked());
    assert_eq!(
        state
            .compare_tree_session
            .as_ref()
            .map(|session| session.horizontal_scroll_locked),
        Some(false)
    );
}

#[test]
fn compare_scroll_lock_is_independent_per_session_tab() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);
    assert!(state.set_lock_compare_horizontal_scrolling_by_default(false));

    state.ensure_compare_tree_session();
    assert!(!state.compare_view_horizontal_scroll_locked());
    assert!(state.toggle_compare_view_horizontal_scroll_locked());
    assert_eq!(
        state
            .compare_tree_session
            .as_ref()
            .map(|session| session.horizontal_scroll_locked),
        Some(true)
    );

    assert!(state.open_or_activate_file_session("src/main.rs").is_some());
    let first_file_session_id = state
        .file_sessions
        .iter()
        .find(|session| session.relative_path == "src/main.rs")
        .map(|session| session.session_id.clone())
        .expect("first file session should exist");
    assert!(!state.compare_view_horizontal_scroll_locked());
    assert_eq!(
        state
            .file_sessions
            .iter()
            .find(|session| session.relative_path == "src/main.rs")
            .map(|session| session.horizontal_scroll_locked),
        Some(false)
    );
    assert_eq!(
        state
            .compare_tree_session
            .as_ref()
            .map(|session| session.horizontal_scroll_locked),
        Some(true)
    );

    assert!(state.toggle_compare_view_horizontal_scroll_locked());
    assert_eq!(
        state
            .file_sessions
            .iter()
            .find(|session| session.relative_path == "src/main.rs")
            .map(|session| session.horizontal_scroll_locked),
        Some(true)
    );

    assert!(state.activate_workspace_session("compare-tree"));
    assert!(state.compare_view_horizontal_scroll_locked());

    assert!(state.open_or_activate_file_session("src/lib.rs").is_some());
    let second_file_session_id = state
        .file_sessions
        .iter()
        .find(|session| session.relative_path == "src/lib.rs")
        .map(|session| session.session_id.clone())
        .expect("second file session should exist");
    assert!(!state.compare_view_horizontal_scroll_locked());
    assert_eq!(
        state
            .file_sessions
            .iter()
            .find(|session| session.relative_path == "src/lib.rs")
            .map(|session| session.horizontal_scroll_locked),
        Some(false)
    );

    assert!(state.activate_workspace_session(first_file_session_id.as_str()));
    assert!(state.compare_view_horizontal_scroll_locked());
    assert!(state.activate_workspace_session(second_file_session_id.as_str()));
    assert!(!state.compare_view_horizontal_scroll_locked());
}

#[test]
fn returning_to_compare_tree_auto_locates_current_file_when_enabled() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert!(state.set_compare_row_focus_path(Some("src/main.rs")));
    assert!(state.open_or_activate_file_session("src/lib.rs").is_some());

    let before_scroll_revision = state.compare_view_scroll_request_revision;
    assert!(state.return_to_compare_tree(false));

    assert_eq!(state.workspace_mode_text(), "compare-view");
    assert_eq!(state.compare_focus_path_raw_text(), "src");
    assert_eq!(state.compare_row_focus_path.as_deref(), Some("src/lib.rs"));
    assert_eq!(
        state.compare_view_scroll_target_relative_path.as_deref(),
        Some("src/lib.rs")
    );
    assert!(state.compare_view_scroll_request_revision > before_scroll_revision);
}

#[test]
fn returning_to_compare_tree_respects_auto_locate_setting() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert!(state.set_compare_row_focus_path(Some("src/main.rs")));
    assert!(state.open_or_activate_file_session("src/lib.rs").is_some());
    assert!(state.set_auto_locate_current_file_on_compare_return(false));

    let before_scroll_revision = state.compare_view_scroll_request_revision;
    assert!(state.return_to_compare_tree(false));

    assert_eq!(state.compare_row_focus_path.as_deref(), Some("src/main.rs"));
    assert_eq!(
        state.compare_view_scroll_request_revision,
        before_scroll_revision
    );
}

#[test]
fn hidden_compare_file_cannot_be_located_without_disturbing_existing_focus() {
    let mut state = state_from_entries(vec![
        text_diff_entry(".env", EntryStatus::Different),
        text_diff_entry("src/main.rs", EntryStatus::Different),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.set_compare_row_focus_path(Some("src/main.rs")));
    assert!(state.open_or_activate_file_session(".env").is_some());
    state.set_show_hidden_files(false);

    let before_scroll_revision = state.compare_view_scroll_request_revision;
    assert!(state.return_to_compare_tree(true));

    assert_eq!(state.workspace_mode_text(), "compare-view");
    assert_eq!(state.compare_row_focus_path.as_deref(), Some("src/main.rs"));
    assert_eq!(
        state.compare_view_scroll_request_revision,
        before_scroll_revision
    );
    assert!(!state.can_locate_relative_path_in_compare_view(".env"));
}

#[test]
fn closing_active_compare_file_tab_auto_locates_when_returning_to_tree() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert!(state.set_compare_row_focus_path(Some("src/main.rs")));
    assert!(state.open_or_activate_file_session("src/lib.rs").is_some());

    assert!(state.close_workspace_session("file:src/lib.rs"));

    assert_eq!(state.workspace_mode_text(), "compare-view");
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(state.compare_row_focus_path.as_deref(), Some("src/lib.rs"));
}

#[test]
fn compare_view_quick_locate_match_detection_respects_current_anchor() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        file_entry("assets/logo.png", EntryStatus::Equal),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.set_compare_view_quick_locate_query("main"));
    assert!(state.compare_view_quick_locate_has_match());

    assert!(state.set_compare_focus_path(CompareFocusPath::relative("assets")));
    assert!(!state.compare_view_quick_locate_has_match());
}

#[test]
fn sidebar_visibility_is_top_level_shell_state() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);

    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));
    assert!(state.sidebar_visible());

    assert!(state.toggle_sidebar_visible());
    assert!(!state.sidebar_visible());
    assert_eq!(state.workspace_mode_text(), "compare-view");
    assert_eq!(state.compare_focus_path_raw_text(), "src");

    assert!(state.set_sidebar_visible(true));
    assert!(state.sidebar_visible());
    assert_eq!(state.workspace_mode_text(), "compare-view");
    assert_eq!(state.compare_focus_path_raw_text(), "src");
}

#[test]
fn workspace_sessions_keep_compare_tree_left_and_unique() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);

    state.ensure_compare_tree_session();
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert!(state.open_or_activate_file_session("src/main.rs").is_some());
    assert!(state.open_or_activate_file_session("src/lib.rs").is_some());
    assert!(state.open_or_activate_file_session("src/main.rs").is_some());

    assert_eq!(
        state.workspace_session_kinds(),
        vec!["compare-tree", "file", "file"]
    );
    assert_eq!(
        state.workspace_session_labels(),
        vec!["Compare Tree", "main.rs", "lib.rs"]
    );
    assert_eq!(state.workspace_session_ids()[0], "compare-tree");
}

#[test]
fn compare_tree_session_close_requires_confirmation_when_file_tabs_exist() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);

    state.ensure_compare_tree_session();
    assert!(state.open_or_activate_file_session("src/main.rs").is_some());

    assert!(state.close_workspace_session("compare-tree"));
    assert!(state.workspace_session_confirmation_open());
    assert!(state.workspace_sessions_visible());
    assert_eq!(
        state.workspace_session_confirmation_title_text(),
        "Close Compare session?"
    );

    assert_eq!(
        state.confirm_workspace_session_action(),
        WorkspaceSessionConfirmationEffect::None
    );
    assert!(!state.workspace_sessions_visible());
    assert_eq!(state.compare_tree_file_tab_count(), 0);
    assert_eq!(state.selected_relative_path.as_deref(), None);
}

#[test]
fn sidebar_file_open_confirmation_closes_compare_session_and_restores_standard_file_view() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("src/lib.rs", EntryStatus::Different),
    ]);
    let main_index = state
        .row_index_for_relative_path("src/main.rs")
        .expect("main row should exist");

    state.ensure_compare_tree_session();
    assert!(state.open_or_activate_file_session("src/lib.rs").is_some());

    assert!(
        state.request_standard_file_view_after_compare_session_close(
            "src/main.rs",
            Some(main_index),
        )
    );
    assert!(state.workspace_session_confirmation_open());
    assert_eq!(
        state.workspace_session_confirmation_title_text(),
        "Open standard File View and close current Compare session?"
    );

    assert_eq!(
        state.confirm_workspace_session_action(),
        WorkspaceSessionConfirmationEffect::LoadSelectedDiff
    );
    assert!(!state.has_compare_tree_session());
    assert!(!state.workspace_sessions_visible());
    assert_eq!(state.workspace_mode_text(), "file-view");
    assert_eq!(state.selected_row, Some(main_index));
    assert_eq!(state.selected_relative_path.as_deref(), Some("src/main.rs"));
}

#[test]
fn compare_session_reset_confirmation_clears_file_tabs_and_reanchors_compare_tree() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        text_diff_entry("docs/guide.md", EntryStatus::Different),
    ]);

    state.ensure_compare_tree_session();
    assert!(state.open_or_activate_file_session("src/main.rs").is_some());
    assert!(
        state
            .open_or_activate_file_session("docs/guide.md")
            .is_some()
    );

    assert!(state.request_compare_session_reset("docs", None));
    assert!(state.workspace_session_confirmation_open());
    assert_eq!(
        state.workspace_session_confirmation_title_text(),
        "Reset Compare session?"
    );

    assert_eq!(
        state.confirm_workspace_session_action(),
        WorkspaceSessionConfirmationEffect::None
    );
    assert!(state.has_compare_tree_session());
    assert_eq!(state.workspace_session_ids(), vec!["compare-tree"]);
    assert_eq!(state.compare_tree_file_tab_count(), 0);
    assert_eq!(state.active_session_id.as_deref(), Some("compare-tree"));
    assert_eq!(state.workspace_mode_text(), "compare-view");
    assert_eq!(state.compare_focus_path_raw_text(), "docs");
}

#[test]
fn closing_file_session_is_immediate_and_keeps_compare_tree_tab() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);

    state.ensure_compare_tree_session();
    assert!(state.open_or_activate_file_session("src/main.rs").is_some());
    let file_session_id = state
        .workspace_session_ids()
        .into_iter()
        .find(|session_id| session_id != "compare-tree")
        .expect("file session should exist");

    assert!(state.close_workspace_session(file_session_id.as_str()));
    assert_eq!(state.workspace_session_ids(), vec!["compare-tree"]);
    assert_eq!(state.workspace_mode_text(), "compare-view");
}

#[test]
fn results_collection_text_mentions_hidden_entries_filtered_by_settings() {
    let mut state = state_from_entries(vec![
        file_entry(".env", EntryStatus::Different),
        file_entry("src/main.rs", EntryStatus::Different),
    ]);
    state.entry_status_filter = "different".to_string();
    state.show_hidden_files = false;

    assert_eq!(
        state.results_collection_text(),
        "Showing 1 / 2 · 1 hidden by Settings · Diff"
    );
}

#[test]
fn tree_projection_keeps_directories_non_selectable_and_files_selectable() {
    let state = state_from_entries(vec![
        CompareEntry::new("src", fc_core::EntryKind::Directory, EntryStatus::Equal),
        text_diff_entry("src/main.rs", EntryStatus::Different),
    ]);

    let rows = state.navigator_tree_row_projections();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].is_directory);
    assert!(!rows[0].is_selectable);
    assert_eq!(rows[1].key, "src/main.rs");
    assert!(rows[1].is_selectable);
}

#[test]
fn tree_membership_ignores_collapsed_ancestor() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    let selected_index = state
        .row_index_for_relative_path("src/main.rs")
        .expect("selected file should exist");
    state.selected_row = Some(selected_index);
    state.selected_relative_path = Some("src/main.rs".to_string());

    assert!(state.is_row_member_in_active_results(selected_index));
    assert!(state.toggle_navigator_tree_node("src"));
    let rows = state.navigator_tree_row_projections();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, "src");
    assert!(!rows[0].is_expanded);
    assert!(state.is_row_member_in_active_results(selected_index));
}

#[test]
fn reveal_navigator_tree_path_expands_nested_ancestors() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    state.navigator_tree_expansion_overrides = BTreeMap::from([("src/bin".to_string(), false)]);

    assert!(state.reveal_navigator_tree_path("src/bin/main.rs"));
    assert_eq!(
        state.navigator_tree_expansion_overrides.get("src/bin"),
        Some(&true)
    );
    assert!(
        state
            .navigator_tree_row_projections()
            .iter()
            .any(|row| row.key == "src/bin/main.rs")
    );
}

#[test]
fn flat_scroll_request_requires_visible_source_row() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        file_entry("docs/readme.md", EntryStatus::Equal),
    ]);
    state.entry_status_filter = "different".to_string();

    let visible_index = state
        .row_index_for_relative_path("src/main.rs")
        .expect("visible row should exist");
    let hidden_index = state
        .row_index_for_relative_path("docs/readme.md")
        .expect("hidden row should exist");

    assert_eq!(
        state.navigator_flat_visual_row_index_for_source_index(visible_index),
        Some(0)
    );
    assert!(state.request_navigator_flat_scroll_to_source_index(visible_index));
    assert_eq!(state.navigator_flat_scroll_request_revision, 1);
    assert_eq!(
        state.navigator_flat_scroll_target_source_index,
        Some(visible_index)
    );

    assert_eq!(
        state.navigator_flat_visual_row_index_for_source_index(hidden_index),
        None
    );
    assert!(!state.request_navigator_flat_scroll_to_source_index(hidden_index));
    assert_eq!(state.navigator_flat_scroll_request_revision, 1);
    assert_eq!(
        state.navigator_flat_scroll_target_source_index,
        Some(visible_index)
    );
}

#[test]
fn tree_scroll_request_requires_visible_revealed_source_row() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    state.navigator_tree_expansion_overrides = BTreeMap::from([("src/bin".to_string(), false)]);
    let source_index = state
        .row_index_for_relative_path("src/bin/main.rs")
        .expect("source row should exist");

    assert_eq!(
        state.navigator_tree_visual_row_index_for_source_index(source_index),
        None
    );
    assert!(!state.request_navigator_tree_scroll_to_source_index(source_index));
    assert_eq!(state.navigator_tree_scroll_request_revision, 0);

    assert!(state.reveal_navigator_tree_path("src/bin/main.rs"));
    assert_eq!(
        state.navigator_tree_visual_row_index_for_source_index(source_index),
        Some(2)
    );
    assert!(state.request_navigator_tree_scroll_to_source_index(source_index));
    assert_eq!(state.navigator_tree_scroll_request_revision, 1);
    assert_eq!(
        state.navigator_tree_scroll_target_source_index,
        Some(source_index)
    );
}

#[test]
fn compare_view_rows_project_visible_tree_status_and_icons() {
    let mut state = state_from_entries(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        file_entry("src/lib.rs", EntryStatus::Equal),
        CompareEntry::new("src/only_left.txt", EntryKind::File, EntryStatus::LeftOnly),
        CompareEntry::new(
            "src/only_right.txt",
            EntryKind::File,
            EntryStatus::RightOnly,
        ),
        CompareEntry::new("src/mismatch", EntryKind::File, EntryStatus::Different).with_detail(
            EntryDetail::TypeMismatch {
                left: EntryKind::File,
                right: EntryKind::Directory,
            },
        ),
    ]);
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    let rows = state.compare_view_row_projections();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].relative_path, "src/lib.rs");
    assert_eq!(rows[0].depth, 0);
    assert_eq!(rows[0].status_label, "Equal");
    assert_eq!(rows[0].left_icon, "TXT");
    assert_eq!(rows[0].right_icon, "TXT");

    let deleted = rows
        .iter()
        .find(|row| row.relative_path == "src/only_left.txt")
        .expect("deleted row should exist");
    assert_eq!(deleted.status_label, "Left");
    assert_eq!(deleted.status_tone, "left");
    assert_eq!(deleted.right_name, "");

    let added = rows
        .iter()
        .find(|row| row.relative_path == "src/only_right.txt")
        .expect("added row should exist");
    assert_eq!(added.status_label, "Right");
    assert_eq!(added.status_tone, "right");
    assert_eq!(added.left_name, "");

    let mismatch = rows
        .iter()
        .find(|row| row.relative_path == "src/mismatch")
        .expect("mismatch row should exist");
    assert_eq!(mismatch.status_label, "Mismatch");
    assert_eq!(mismatch.left_icon, "TXT");
    assert_eq!(mismatch.right_icon, "DIR");
    assert!(!mismatch.is_expandable);
}

#[test]
fn compare_view_tree_toggle_reveals_nested_rows_in_place() {
    let mut state = state_from_entries(vec![
        directory_entry("src/bin", EntryStatus::Different),
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        file_entry("src/bin/lib.rs", EntryStatus::Equal),
        text_diff_entry("src/root.rs", EntryStatus::Different),
    ]);
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    let rows = state.compare_view_row_projections();
    assert_eq!(
        rows.iter()
            .map(|row| row.relative_path.as_str())
            .collect::<Vec<_>>(),
        vec!["src/bin", "src/root.rs"]
    );
    assert!(rows[0].is_expandable);
    assert!(!rows[0].is_expanded);

    assert!(state.toggle_compare_view_node("src/bin"));
    let expanded_rows = state.compare_view_row_projections();
    assert_eq!(
        expanded_rows
            .iter()
            .map(|row| row.relative_path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "src/bin",
            "src/bin/lib.rs",
            "src/bin/main.rs",
            "src/root.rs"
        ]
    );
    assert_eq!(expanded_rows[1].depth, 1);
    assert_eq!(expanded_rows[2].depth, 1);
}

#[test]
fn reveal_compare_view_path_expands_nested_ancestors() {
    let mut state = state_from_entries(vec![
        directory_entry("src/bin", EntryStatus::Different),
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
    ]);
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    assert!(state.reveal_compare_view_path("src/bin/main.rs"));
    assert_eq!(
        state.compare_view_expansion_overrides.get("src/bin"),
        Some(&true)
    );
    assert!(
        state
            .compare_view_row_projections()
            .iter()
            .any(|row| row.relative_path == "src/bin/main.rs")
    );
}

#[test]
fn compare_view_row_action_uses_tree_toggle_for_directories() {
    let mut state = state_from_entries(vec![
        directory_entry("src/bin", EntryStatus::Different),
        text_diff_entry("src/bin/main.rs", EntryStatus::Different),
        text_diff_entry("src/root.rs", EntryStatus::Different),
    ]);
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    assert_eq!(
        state.compare_view_row_action("src/bin"),
        Some(CompareViewRowAction::ToggleDirectory)
    );
    assert_eq!(
        state.compare_view_row_action("src/root.rs"),
        Some(CompareViewRowAction::OpenFileView)
    );
}

#[test]
fn compare_view_hidden_files_preference_filters_hidden_subtrees() {
    let mut state = state_from_entries(vec![
        directory_entry("src/.cache", EntryStatus::Different),
        text_diff_entry("src/.cache/secret.txt", EntryStatus::Different),
        text_diff_entry("src/visible.txt", EntryStatus::Different),
    ]);
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src")));

    state.set_show_hidden_files(false);
    assert_eq!(
        state
            .compare_view_row_projections()
            .iter()
            .map(|row| row.relative_path.as_str())
            .collect::<Vec<_>>(),
        vec!["src/visible.txt"]
    );

    state.set_show_hidden_files(true);
    assert_eq!(
        state
            .compare_view_row_projections()
            .iter()
            .map(|row| row.relative_path.as_str())
            .collect::<Vec<_>>(),
        vec!["src/.cache", "src/visible.txt"]
    );
}

#[test]
fn hiding_hidden_files_reanchors_compare_focus_to_visible_parent() {
    let mut state = state_from_entries(vec![
        directory_entry("src/.cache", EntryStatus::Different),
        text_diff_entry("src/.cache/secret.txt", EntryStatus::Different),
        text_diff_entry("src/visible.txt", EntryStatus::Different),
    ]);
    assert!(state.set_compare_focus_path(CompareFocusPath::relative("src/.cache")));
    assert_eq!(state.compare_focus_path_raw_text(), "src/.cache");
    assert_eq!(
        state.compare_row_focus_path.as_deref(),
        Some("src/.cache/secret.txt")
    );

    state.set_show_hidden_files(false);
    assert_eq!(state.compare_focus_path_raw_text(), "src");
    assert_eq!(
        state.compare_row_focus_path.as_deref(),
        Some("src/visible.txt")
    );
}

#[test]
fn file_view_header_context_uses_compare_status_and_compare_path() {
    let mut state =
        state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);
    let selected_index = state
        .row_index_for_relative_path("src/main.rs")
        .expect("selected file should exist");
    state.selected_row = Some(selected_index);
    state.selected_relative_path = Some("src/main.rs".to_string());

    assert_eq!(state.file_view_title_text(), "main.rs");
    assert_eq!(state.file_view_compare_status_label(), "Modified");
    assert_eq!(state.file_view_compare_status_tone(), "different");
    assert_eq!(
        state.file_view_path_context_text(),
        "Compare Path · src/main.rs"
    );
}

#[test]
fn prune_expansion_overrides_removes_invalid_paths_and_default_states() {
    let mut state = state_from_entries(vec![text_diff_entry(
        "src/bin/main.rs",
        EntryStatus::Different,
    )]);
    state.navigator_tree_expansion_overrides = BTreeMap::from([
        ("src/bin".to_string(), true),
        ("src".to_string(), true),
        ("old".to_string(), false),
    ]);

    assert!(state.prune_navigator_tree_expansion_overrides());
    assert_eq!(
        state.navigator_tree_expansion_overrides,
        BTreeMap::from([("src/bin".to_string(), true)])
    );
}

#[test]
fn navigator_row_projection_promotes_leaf_name_and_parent_context() {
    let state = state_from_entries(vec![file_entry(
        "assets/js/runtime/fernetBrowser.js",
        EntryStatus::RightOnly,
    )]);

    let projected = state.navigator_row_projections();
    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].display_name, "fernetBrowser.js");
    assert_eq!(projected[0].parent_path, "assets/js/runtime");
    assert_eq!(projected[0].secondary_text, "Text-only preview");
    assert_eq!(
        projected[0].tooltip_text,
        "fernetBrowser.js\nassets/js/runtime"
    );
}

#[test]
fn navigator_row_projection_keeps_full_parent_path_for_tooltip_completion() {
    let full_parent_path = "workspace/frontend/src/components/navigation/sidebar/results";
    let state = state_from_entries(vec![file_entry(
        &format!("{full_parent_path}/fernetBrowser.js"),
        EntryStatus::RightOnly,
    )]);

    let projected = state.navigator_row_projections();
    assert_eq!(projected.len(), 1);
    assert!(projected[0].parent_path.contains('…'));
    assert_eq!(
        projected[0].tooltip_text,
        format!("fernetBrowser.js\n{full_parent_path}")
    );
}

#[test]
fn navigator_row_projection_marks_image_preview_as_unavailable() {
    let state = state_from_entries(vec![file_entry("assets/logo.jpg", EntryStatus::LeftOnly)]);

    let projected = state.navigator_row_projections();
    assert_eq!(projected[0].secondary_text, "Image · no text preview");
}

#[test]
fn navigator_row_projection_marks_file_compare_diff_as_unavailable() {
    let state = state_from_entries(vec![file_comparison_entry(
        "assets/logo.png",
        EntryStatus::Different,
        1,
        2,
        true,
    )]);

    let projected = state.navigator_row_projections();
    assert_eq!(
        projected[0].secondary_text,
        "Image · no text diff · 1B / 2B"
    );
}

#[test]
fn navigator_row_projection_tracks_name_and_path_hits() {
    let row_state = state_from_entries(vec![text_diff_entry(
        "assets/js/runtime/fernetBrowser.js",
        EntryStatus::Different,
    )]);

    let mut state = row_state.clone();
    state.entry_filter = "fernet".to_string();
    let projected = state.navigator_row_projections();
    assert!(projected[0].display_name_matches_filter);
    assert!(!projected[0].parent_path_matches_filter);

    let mut state = row_state.clone();
    state.entry_filter = "js/runtime".to_string();
    let projected = state.navigator_row_projections();
    assert!(!projected[0].display_name_matches_filter);
    assert!(projected[0].parent_path_matches_filter);

    let mut state = row_state;
    state.entry_filter = "runtime/fernet".to_string();
    let projected = state.navigator_row_projections();
    assert!(projected[0].display_name_matches_filter || projected[0].parent_path_matches_filter);
}

#[test]
fn navigator_row_projection_summarizes_text_diff_for_scanability() {
    let state = state_from_entries(vec![text_diff_entry("src/main.rs", EntryStatus::Different)]);

    let projected = state.navigator_row_projections();
    assert_eq!(
        projected[0].secondary_text,
        "Text diff · 2h · +4 · -1 · 8ctx"
    );
}

#[test]
fn filtering_does_not_mutate_underlying_rows() {
    let mut state = state_from_entries(sample_entries());
    let rows = state.entry_rows.clone();
    state.entry_filter = "logo".to_string();
    let _ = state.filtered_entry_rows_with_index();
    assert_eq!(state.entry_rows, rows);
}
