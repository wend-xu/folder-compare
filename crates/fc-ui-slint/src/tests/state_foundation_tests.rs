use super::*;
use crate::compare_foundation::CompareFocusPath;
use crate::tests::fixtures::{
    file_comparison_entry, file_entry, state_from_entries, text_diff_entry,
};
use fc_core::{CompareEntry, EntryStatus};

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

    state.set_workspace_mode(WorkspaceMode::CompareView);
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
