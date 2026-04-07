//! Rust-owned tree projection for Results / Navigator tree mode.

use crate::compare_foundation::{CompareFoundation, CompareFoundationNode, CompareNodeKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilteredNavigatorNode<'a> {
    node: &'a CompareFoundationNode,
    display_status: String,
    children: Vec<FilteredNavigatorNode<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigatorTreeProjection {
    pub rows: Vec<NavigatorTreeRowProjection>,
    pub selectable_source_indices: BTreeSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigatorTreeRowProjection {
    pub key: String,
    pub relative_path: String,
    pub source_index: Option<usize>,
    pub depth: u16,
    pub is_directory: bool,
    pub is_expandable: bool,
    pub is_expanded: bool,
    pub display_status: String,
    pub display_name: String,
    pub tooltip_text: String,
    pub is_selectable: bool,
}

pub fn project_navigator_tree_rows(
    foundation: &CompareFoundation,
    show_hidden_files: bool,
    status_filter: &str,
    expansion_overrides: &BTreeMap<String, bool>,
) -> NavigatorTreeProjection {
    let normalized_filter = normalize_status_filter(status_filter);
    let Some(root) = filter_tree_node(
        foundation,
        "",
        show_hidden_files,
        normalized_filter.as_str(),
    ) else {
        return NavigatorTreeProjection {
            rows: Vec::new(),
            selectable_source_indices: BTreeSet::new(),
        };
    };

    let mut selectable_source_indices = BTreeSet::new();
    let mut rows = Vec::new();
    for child in &root.children {
        flatten_filtered_tree(
            child,
            expansion_overrides,
            true,
            &mut rows,
            &mut selectable_source_indices,
        );
    }

    NavigatorTreeProjection {
        rows,
        selectable_source_indices,
    }
}

fn filter_tree_node<'a>(
    foundation: &'a CompareFoundation,
    key: &str,
    show_hidden_files: bool,
    status_filter: &str,
) -> Option<FilteredNavigatorNode<'a>> {
    let node = foundation.node(key)?;
    if node.kind != CompareNodeKind::Root
        && !show_hidden_files
        && is_hidden_relative_path(&node.relative_path)
    {
        return None;
    }

    let children = node
        .child_relative_paths
        .iter()
        .filter_map(|child_key| {
            filter_tree_node(
                foundation,
                child_key.as_str(),
                show_hidden_files,
                status_filter,
            )
        })
        .collect::<Vec<_>>();

    if node.kind == CompareNodeKind::Root {
        return if children.is_empty() {
            None
        } else {
            Some(FilteredNavigatorNode {
                node,
                display_status: aggregate_statuses(
                    children.iter().map(|child| child.display_status.as_str()),
                    node.base_status.as_str(),
                ),
                children,
            })
        };
    }

    let direct_status = node.has_compare_entry().then(|| node.base_status.as_str());
    let keep_without_children = direct_status
        .map(|status| status_filter_matches(status, status_filter))
        .unwrap_or(false);
    if !children.is_empty() || keep_without_children {
        let display_status = if children.is_empty() {
            direct_status
                .unwrap_or(node.base_status.as_str())
                .to_string()
        } else {
            aggregate_statuses(
                children.iter().map(|child| child.display_status.as_str()),
                node.base_status.as_str(),
            )
        };
        return Some(FilteredNavigatorNode {
            node,
            display_status,
            children,
        });
    }

    None
}

fn flatten_filtered_tree(
    node: &FilteredNavigatorNode<'_>,
    expansion_overrides: &BTreeMap<String, bool>,
    push_visible_row: bool,
    out: &mut Vec<NavigatorTreeRowProjection>,
    selectable_source_indices: &mut BTreeSet<usize>,
) {
    if node.node.kind == CompareNodeKind::File {
        if let Some(source_index) = node.node.source_index {
            selectable_source_indices.insert(source_index);
        }
    }

    let is_expandable = !node.children.is_empty();
    let is_expanded = is_expandable
        && expansion_state(
            node.node.relative_path.as_str(),
            node.node.path_depth,
            expansion_overrides,
        );
    if push_visible_row {
        out.push(NavigatorTreeRowProjection {
            key: node.node.relative_path.clone(),
            relative_path: node.node.relative_path.clone(),
            source_index: node.node.source_index,
            depth: node.node.path_depth.saturating_sub(1),
            is_directory: node.node.kind.is_directory_target(),
            is_expandable,
            is_expanded,
            display_status: node.display_status.clone(),
            display_name: node.node.display_name.clone(),
            tooltip_text: if node.node.relative_path.is_empty() {
                node.node.display_name.clone()
            } else {
                node.node.relative_path.clone()
            },
            is_selectable: node.node.kind == CompareNodeKind::File
                && node.node.source_index.is_some(),
        });
    }

    let push_children = push_visible_row && is_expanded;
    for child in &node.children {
        flatten_filtered_tree(
            child,
            expansion_overrides,
            push_children,
            out,
            selectable_source_indices,
        );
    }
}

fn expansion_state(
    key: &str,
    path_depth: u16,
    expansion_overrides: &BTreeMap<String, bool>,
) -> bool {
    expansion_overrides
        .get(key)
        .copied()
        .unwrap_or(path_depth <= 1)
}

pub(crate) fn navigator_tree_toggle_target(
    foundation: &CompareFoundation,
    key: &str,
) -> Option<(String, u16)> {
    let normalized_key = normalize_relative_path(key);
    let node = foundation.node(normalized_key.as_str())?;
    if normalized_key.is_empty()
        || !node.kind.is_directory_target()
        || node.child_relative_paths.is_empty()
    {
        return None;
    }
    Some((normalized_key, node.path_depth))
}

pub(crate) fn navigator_tree_reveal_targets(
    foundation: &CompareFoundation,
    relative_path: &str,
) -> Vec<(String, u16)> {
    let normalized_path = normalize_relative_path(relative_path);
    let components = path_components(normalized_path.as_str());
    if components.len() < 2 {
        return Vec::new();
    }

    let mut targets = Vec::new();
    for index in 0..components.len().saturating_sub(1) {
        let key = join_path_components(&components[..=index]);
        if let Some(target) = navigator_tree_toggle_target(foundation, key.as_str()) {
            targets.push(target);
        }
    }
    targets
}

fn aggregate_statuses<'a>(statuses: impl IntoIterator<Item = &'a str>, fallback: &str) -> String {
    let statuses = statuses
        .into_iter()
        .map(str::trim)
        .filter(|status| !status.is_empty())
        .collect::<Vec<_>>();
    let Some(first) = statuses.first().copied() else {
        return fallback.to_string();
    };
    if statuses
        .iter()
        .all(|status| status.eq_ignore_ascii_case(first))
    {
        first.to_string()
    } else {
        "different".to_string()
    }
}

fn normalize_status_filter(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "all" => "all".to_string(),
        "different" => "different".to_string(),
        "equal" => "equal".to_string(),
        "left-only" => "left-only".to_string(),
        "right-only" => "right-only".to_string(),
        _ => "all".to_string(),
    }
}

fn status_filter_matches(status: &str, filter: &str) -> bool {
    filter == "all" || status.eq_ignore_ascii_case(filter)
}

fn normalize_relative_path(relative_path: &str) -> String {
    relative_path
        .trim()
        .trim_matches(|ch| ch == '/' || ch == '\\')
        .replace('\\', "/")
}

fn path_components(relative_path: &str) -> Vec<&str> {
    relative_path
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

fn join_path_components(components: &[&str]) -> String {
    components.join("/")
}

fn is_hidden_relative_path(relative_path: &str) -> bool {
    relative_path
        .trim_matches(|ch| ch == '/' || ch == '\\')
        .split(['/', '\\'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .any(|part| part.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare_foundation::foundation_from_legacy_rows;
    use crate::view_models::CompareEntryRowViewModel;

    fn row(
        relative_path: &str,
        status: &str,
        entry_kind: &str,
        can_load_diff: bool,
    ) -> CompareEntryRowViewModel {
        CompareEntryRowViewModel {
            relative_path: relative_path.to_string(),
            status: status.to_string(),
            entry_kind: entry_kind.to_string(),
            can_load_diff,
            can_load_analysis: can_load_diff,
            ..CompareEntryRowViewModel::default()
        }
    }

    #[test]
    fn projection_defaults_top_level_directories_to_expanded() {
        let foundation = foundation_from_legacy_rows(&[
            row("src/main.rs", "different", "file", true),
            row("src/util.rs", "equal", "file", true),
            row("Cargo.toml", "equal", "file", true),
        ]);

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
        let foundation = foundation_from_legacy_rows(&[
            row("src", "equal", "directory", false),
            row("src/a.txt", "equal", "file", true),
            row("src/b.txt", "different", "file", true),
        ]);

        let projection =
            project_navigator_tree_rows(&foundation, true, "different", &BTreeMap::new());

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
        let foundation = foundation_from_legacy_rows(&[
            row(".env", "different", "file", true),
            row("src/.cache/data.json", "different", "file", true),
            row("src/main.rs", "different", "file", true),
        ]);

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
        let foundation = foundation_from_legacy_rows(&[
            row("src/app/main.rs", "different", "file", true),
            row("src/app/lib.rs", "equal", "file", true),
        ]);

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
        let foundation = foundation_from_legacy_rows(&[
            row("src/app/main.rs", "different", "file", true),
            row("src/app/lib.rs", "equal", "file", true),
        ]);

        assert_eq!(
            navigator_tree_reveal_targets(&foundation, "src/app/main.rs"),
            vec![("src".to_string(), 1), ("src/app".to_string(), 2)]
        );
        assert!(navigator_tree_reveal_targets(&foundation, "src").is_empty());
    }
}
