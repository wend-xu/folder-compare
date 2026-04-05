//! Rust-owned tree domain for Results / Navigator tree mode.

use crate::view_models::CompareEntryRowViewModel;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalNavigatorTree {
    nodes: BTreeMap<String, CanonicalNavigatorNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalNavigatorNode {
    key: String,
    relative_path: String,
    display_name: String,
    kind: NavigatorTreeNodeKind,
    source_index: Option<usize>,
    direct_status: Option<String>,
    base_status: String,
    path_depth: u16,
    child_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigatorTreeNodeKind {
    Root,
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilteredNavigatorNode<'a> {
    node: &'a CanonicalNavigatorNode,
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

impl CanonicalNavigatorNode {
    pub(crate) fn is_directory(&self) -> bool {
        matches!(
            self.kind,
            NavigatorTreeNodeKind::Root | NavigatorTreeNodeKind::Directory
        )
    }
}

pub fn build_canonical_navigator_tree(
    entry_rows: &[CompareEntryRowViewModel],
) -> CanonicalNavigatorTree {
    let mut nodes = BTreeMap::new();
    nodes.insert(
        String::new(),
        CanonicalNavigatorNode {
            key: String::new(),
            relative_path: String::new(),
            display_name: "compare root".to_string(),
            kind: NavigatorTreeNodeKind::Root,
            source_index: None,
            direct_status: None,
            base_status: "equal".to_string(),
            path_depth: 0,
            child_keys: Vec::new(),
        },
    );

    for (source_index, row) in entry_rows.iter().enumerate() {
        let normalized_path = normalize_relative_path(&row.relative_path);
        let components = path_components(&normalized_path);
        if components.is_empty() {
            continue;
        }

        let final_kind = node_kind_from_entry_kind(&row.entry_kind);
        let mut parent_key = String::new();
        for (component_index, component) in components.iter().enumerate() {
            let is_last = component_index + 1 == components.len();
            let key = join_path_components(&components[..=component_index]);
            let path_depth = u16::try_from(component_index + 1).unwrap_or(u16::MAX);
            let kind = if is_last {
                final_kind
            } else {
                NavigatorTreeNodeKind::Directory
            };

            nodes
                .entry(key.clone())
                .or_insert_with(|| CanonicalNavigatorNode {
                    key: key.clone(),
                    relative_path: key.clone(),
                    display_name: (*component).to_string(),
                    kind,
                    source_index: None,
                    direct_status: None,
                    base_status: "equal".to_string(),
                    path_depth,
                    child_keys: Vec::new(),
                });

            if !parent_key.is_empty() || key != parent_key {
                let parent = nodes
                    .get_mut(&parent_key)
                    .expect("parent tree node must exist");
                parent.child_keys.push(key.clone());
            }

            if is_last {
                let node = nodes.get_mut(&key).expect("final tree node must exist");
                node.kind = kind;
                node.source_index = Some(source_index);
                node.direct_status = Some(row.status.clone());
            }

            parent_key = key;
        }
    }

    for node in nodes.values_mut() {
        node.child_keys.sort();
        node.child_keys.dedup();
    }

    let root_key = String::new();
    compute_base_status(&root_key, &mut nodes);

    CanonicalNavigatorTree { nodes }
}

pub fn project_navigator_tree_rows(
    tree: &CanonicalNavigatorTree,
    show_hidden_files: bool,
    status_filter: &str,
    expansion_overrides: &BTreeMap<String, bool>,
) -> NavigatorTreeProjection {
    let normalized_filter = normalize_status_filter(status_filter);
    let Some(root) = filter_tree_node(tree, "", show_hidden_files, normalized_filter.as_str())
    else {
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
    tree: &'a CanonicalNavigatorTree,
    key: &str,
    show_hidden_files: bool,
    status_filter: &str,
) -> Option<FilteredNavigatorNode<'a>> {
    let node = tree.nodes.get(key)?;
    if !matches!(node.kind, NavigatorTreeNodeKind::Root)
        && !show_hidden_files
        && is_hidden_relative_path(&node.relative_path)
    {
        return None;
    }

    let children = node
        .child_keys
        .iter()
        .filter_map(|child_key| filter_tree_node(tree, child_key, show_hidden_files, status_filter))
        .collect::<Vec<_>>();

    if matches!(node.kind, NavigatorTreeNodeKind::Root) {
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

    let direct_status = node.direct_status.as_deref();
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
    if matches!(node.node.kind, NavigatorTreeNodeKind::File) {
        if let Some(source_index) = node.node.source_index {
            selectable_source_indices.insert(source_index);
        }
    }

    let is_expandable = !node.children.is_empty();
    let is_expanded = is_expandable
        && expansion_state(
            node.node.key.as_str(),
            node.node.path_depth,
            expansion_overrides,
        );
    if push_visible_row {
        out.push(NavigatorTreeRowProjection {
            key: node.node.key.clone(),
            relative_path: node.node.relative_path.clone(),
            source_index: node.node.source_index,
            depth: node.node.path_depth.saturating_sub(1),
            is_directory: node.node.is_directory(),
            is_expandable,
            is_expanded,
            display_status: node.display_status.clone(),
            display_name: node.node.display_name.clone(),
            tooltip_text: if node.node.relative_path.is_empty() {
                node.node.display_name.clone()
            } else {
                node.node.relative_path.clone()
            },
            is_selectable: matches!(node.node.kind, NavigatorTreeNodeKind::File)
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
    entry_rows: &[CompareEntryRowViewModel],
    key: &str,
) -> Option<(String, u16)> {
    let normalized_key = normalize_relative_path(key);
    if normalized_key.is_empty() {
        return None;
    }

    let prefix = format!("{normalized_key}/");
    entry_rows
        .iter()
        .map(|row| normalize_relative_path(&row.relative_path))
        .any(|relative_path| relative_path.starts_with(prefix.as_str()))
        .then(|| {
            let path_depth =
                u16::try_from(path_components(normalized_key.as_str()).len()).unwrap_or(u16::MAX);
            (normalized_key, path_depth)
        })
}

pub(crate) fn navigator_tree_reveal_targets(
    entry_rows: &[CompareEntryRowViewModel],
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
        if let Some(target) = navigator_tree_toggle_target(entry_rows, key.as_str()) {
            targets.push(target);
        }
    }
    targets
}

fn compute_base_status(key: &str, nodes: &mut BTreeMap<String, CanonicalNavigatorNode>) -> String {
    let child_keys = nodes
        .get(key)
        .expect("tree node must exist")
        .child_keys
        .clone();
    let child_statuses = child_keys
        .iter()
        .map(|child_key| compute_base_status(child_key, nodes))
        .collect::<Vec<_>>();

    let fallback = nodes
        .get(key)
        .and_then(|node| node.direct_status.clone())
        .unwrap_or_else(|| "equal".to_string());
    let aggregated = aggregate_statuses(
        child_statuses.iter().map(|status| status.as_str()),
        fallback.as_str(),
    );
    if let Some(node) = nodes.get_mut(key) {
        node.base_status = aggregated.clone();
    }
    aggregated
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

fn node_kind_from_entry_kind(entry_kind: &str) -> NavigatorTreeNodeKind {
    match entry_kind.trim().to_ascii_lowercase().as_str() {
        "directory" => NavigatorTreeNodeKind::Directory,
        "symlink" => NavigatorTreeNodeKind::Symlink,
        "other" => NavigatorTreeNodeKind::Other,
        _ => NavigatorTreeNodeKind::File,
    }
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
    fn canonical_tree_builds_missing_ancestors_without_duplicates() {
        let rows = vec![
            row("src/app/main.rs", "different", "file", true),
            row("src/app/lib.rs", "equal", "file", true),
            row("assets/logo.png", "different", "file", false),
        ];

        let tree = build_canonical_navigator_tree(&rows);

        let src = tree.nodes.get("src").expect("src directory should exist");
        assert!(src.is_directory());
        assert_eq!(src.path_depth, 1);
        assert_eq!(src.child_keys, vec!["src/app".to_string()]);

        let app = tree
            .nodes
            .get("src/app")
            .expect("app directory should exist");
        assert!(app.is_directory());
        assert_eq!(app.child_keys.len(), 2);
        assert!(app.child_keys.contains(&"src/app/lib.rs".to_string()));
        assert!(app.child_keys.contains(&"src/app/main.rs".to_string()));

        let main = tree
            .nodes
            .get("src/app/main.rs")
            .expect("main file should exist");
        assert!(!main.is_directory());
        assert_eq!(main.source_index, Some(0));
    }

    #[test]
    fn projection_defaults_top_level_directories_to_expanded() {
        let rows = vec![
            row("src/main.rs", "different", "file", true),
            row("src/util.rs", "equal", "file", true),
            row("Cargo.toml", "equal", "file", true),
        ];

        let tree = build_canonical_navigator_tree(&rows);
        let projection =
            project_navigator_tree_rows(&tree, true, "all", &BTreeMap::<String, bool>::new());

        assert_eq!(projection.rows[0].key, "Cargo.toml");
        assert_eq!(projection.rows[1].key, "src");
        assert!(projection.rows[1].is_expandable);
        assert!(projection.rows[1].is_expanded);
        assert!(projection.rows.iter().any(|row| row.key == "src/main.rs"));
    }

    #[test]
    fn status_filter_prunes_tree_and_recomputes_directory_status() {
        let rows = vec![
            row("src", "equal", "directory", false),
            row("src/a.txt", "equal", "file", true),
            row("src/b.txt", "different", "file", true),
        ];

        let tree = build_canonical_navigator_tree(&rows);
        let projection = project_navigator_tree_rows(&tree, true, "different", &BTreeMap::new());

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
        let rows = vec![
            row(".env", "different", "file", true),
            row("src/.cache/data.json", "different", "file", true),
            row("src/main.rs", "different", "file", true),
        ];

        let tree = build_canonical_navigator_tree(&rows);
        let projection = project_navigator_tree_rows(&tree, false, "all", &BTreeMap::new());

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
    fn expansion_override_can_collapse_top_level_directory_without_changing_membership() {
        let rows = vec![
            row("src/main.rs", "different", "file", true),
            row("src/lib.rs", "equal", "file", true),
        ];
        let tree = build_canonical_navigator_tree(&rows);
        let projection = project_navigator_tree_rows(
            &tree,
            true,
            "all",
            &BTreeMap::from([(String::from("src"), false)]),
        );

        assert_eq!(projection.rows.len(), 1);
        assert_eq!(projection.rows[0].key, "src");
        assert!(!projection.rows[0].is_expanded);
        assert_eq!(
            projection.selectable_source_indices,
            BTreeSet::from([0usize, 1usize])
        );
    }

    #[test]
    fn toggle_target_accepts_missing_ancestor_directory_with_descendants() {
        let rows = vec![
            row("src/app/main.rs", "different", "file", true),
            row("src/app/lib.rs", "equal", "file", true),
        ];

        assert_eq!(
            navigator_tree_toggle_target(&rows, "src/app"),
            Some((String::from("src/app"), 2))
        );
        assert_eq!(navigator_tree_toggle_target(&rows, "src/app/main.rs"), None);
    }

    #[test]
    fn reveal_targets_include_expandable_ancestors_only() {
        let rows = vec![
            row("src/app/main.rs", "different", "file", true),
            row("src/app/lib.rs", "equal", "file", true),
        ];

        assert_eq!(
            navigator_tree_reveal_targets(&rows, "src/app/main.rs"),
            vec![("src".to_string(), 1), ("src/app".to_string(), 2)]
        );
        assert!(navigator_tree_reveal_targets(&rows, "src").is_empty());
    }
}
