use super::*;
use crate::tests::fixtures::{compare_fixture, directory_entry, file_entry, text_diff_entry};
use fc_core::EntryStatus;

#[test]
fn projection_defaults_top_level_directories_to_expanded() {
    let foundation = compare_fixture(vec![
        text_diff_entry("src/main.rs", EntryStatus::Different),
        file_entry("src/util.rs", EntryStatus::Equal),
        file_entry("Cargo.toml", EntryStatus::Equal),
    ])
    .foundation;

    let projection =
        project_navigator_tree_rows(&foundation, true, "all", &BTreeMap::<String, bool>::new());

    assert_eq!(projection.rows[0].key, "Cargo.toml");
    assert_eq!(projection.rows[1].key, "src");
    assert!(projection.rows[1].is_expandable);
    assert!(projection.rows[1].is_expanded);
    assert!(projection.rows.iter().any(|row| row.key == "src/main.rs"));
}

#[test]
fn status_filter_prunes_tree_and_recomputes_directory_status() {
    let foundation = compare_fixture(vec![
        directory_entry("src", EntryStatus::Equal),
        file_entry("src/a.txt", EntryStatus::Equal),
        text_diff_entry("src/b.txt", EntryStatus::Different),
    ])
    .foundation;

    let projection = project_navigator_tree_rows(&foundation, true, "different", &BTreeMap::new());

    assert_eq!(projection.rows.len(), 2);
    assert_eq!(projection.rows[0].key, "src");
    assert_eq!(projection.rows[0].display_status, "different");
    assert_eq!(projection.rows[1].key, "src/b.txt");
    assert_eq!(projection.rows[1].display_status, "different");
    assert!(projection.selectable_source_indices.contains(&2));
    assert!(!projection.selectable_source_indices.contains(&1));
}

#[test]
fn hidden_files_filter_excludes_hidden_subtree_rows() {
    let foundation = compare_fixture(vec![
        file_entry(".env", EntryStatus::Different),
        file_entry("src/.cache/data.json", EntryStatus::Different),
        text_diff_entry("src/main.rs", EntryStatus::Different),
    ])
    .foundation;

    let projection = project_navigator_tree_rows(&foundation, false, "all", &BTreeMap::new());

    assert!(
        projection
            .rows
            .iter()
            .all(|row| !row.key.starts_with(".env"))
    );
    assert!(
        projection
            .rows
            .iter()
            .all(|row| !row.key.starts_with("src/.cache"))
    );
    assert!(projection.rows.iter().any(|row| row.key == "src/main.rs"));
    assert_eq!(projection.selectable_source_indices.len(), 1);
}

#[test]
fn toggle_target_accepts_expandable_directory_only() {
    let foundation = compare_fixture(vec![
        text_diff_entry("src/app/main.rs", EntryStatus::Different),
        file_entry("src/app/lib.rs", EntryStatus::Equal),
    ])
    .foundation;

    assert_eq!(
        navigator_tree_toggle_target(&foundation, "src/app"),
        Some((String::from("src/app"), 2))
    );
    assert_eq!(
        navigator_tree_toggle_target(&foundation, "src/app/main.rs"),
        None
    );
}

#[test]
fn reveal_targets_include_expandable_ancestors_only() {
    let foundation = compare_fixture(vec![
        text_diff_entry("src/app/main.rs", EntryStatus::Different),
        file_entry("src/app/lib.rs", EntryStatus::Equal),
    ])
    .foundation;

    assert_eq!(
        navigator_tree_reveal_targets(&foundation, "src/app/main.rs"),
        vec![("src".to_string(), 1), ("src/app".to_string(), 2)]
    );
    assert!(navigator_tree_reveal_targets(&foundation, "src").is_empty());
}
