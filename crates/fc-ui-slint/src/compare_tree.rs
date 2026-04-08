//! Rust-owned visible tree projection for Compare View.

use crate::compare_foundation::{CompareFocusPath, CompareFoundation};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareTreeVisibleRow {
    pub relative_path: String,
    pub depth: u16,
    pub is_directory: bool,
    pub is_expandable: bool,
    pub is_expanded: bool,
}

pub fn project_compare_tree_rows(
    foundation: &CompareFoundation,
    focus: &CompareFocusPath,
    show_hidden_files: bool,
    expansion_overrides: &BTreeMap<String, bool>,
) -> Vec<CompareTreeVisibleRow> {
    let focus = foundation.clamp_compare_focus_path(focus);
    let anchor_key = focus.as_relative_path().unwrap_or_default();
    let Some(anchor_node) = foundation.node(anchor_key) else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    for child_key in &anchor_node.child_relative_paths {
        let Some(child_node) = foundation.node(child_key.as_str()) else {
            continue;
        };
        flatten_compare_tree(
            foundation,
            child_node.relative_path.as_str(),
            anchor_node.path_depth,
            show_hidden_files,
            expansion_overrides,
            &mut rows,
        );
    }
    rows
}

fn flatten_compare_tree(
    foundation: &CompareFoundation,
    key: &str,
    anchor_depth: u16,
    show_hidden_files: bool,
    expansion_overrides: &BTreeMap<String, bool>,
    out: &mut Vec<CompareTreeVisibleRow>,
) {
    let Some(node) = foundation.node(key) else {
        return;
    };
    if !show_hidden_files && is_hidden_relative_path(&node.relative_path) {
        return;
    }

    let is_directory = node.is_directory_target();
    let is_expandable = is_directory && !node.child_relative_paths.is_empty();
    let is_expanded =
        is_expandable && compare_tree_expansion_state(key, node.path_depth, expansion_overrides);
    out.push(CompareTreeVisibleRow {
        relative_path: node.relative_path.clone(),
        depth: node
            .path_depth
            .saturating_sub(anchor_depth.saturating_add(1)),
        is_directory,
        is_expandable,
        is_expanded,
    });

    if !is_expanded {
        return;
    }

    for child_key in &node.child_relative_paths {
        flatten_compare_tree(
            foundation,
            child_key.as_str(),
            anchor_depth,
            show_hidden_files,
            expansion_overrides,
            out,
        );
    }
}

pub(crate) fn compare_tree_toggle_target(
    foundation: &CompareFoundation,
    key: &str,
) -> Option<(String, u16)> {
    let normalized_key = key.trim();
    if normalized_key.is_empty() {
        return None;
    }

    let node = foundation.node(normalized_key)?;
    if !node.is_directory_target() || node.child_relative_paths.is_empty() {
        return None;
    }

    Some((normalized_key.to_string(), node.path_depth))
}

pub(crate) fn compare_tree_reveal_targets(
    foundation: &CompareFoundation,
    focus: &CompareFocusPath,
    relative_path: &str,
) -> Vec<(String, u16)> {
    let normalized_path = relative_path.trim();
    if normalized_path.is_empty() {
        return Vec::new();
    }

    let focus = foundation.clamp_compare_focus_path(focus);
    let anchor_key = focus.as_relative_path().unwrap_or_default();
    if !compare_tree_contains_path(foundation, anchor_key, normalized_path) {
        return Vec::new();
    }

    let mut targets = Vec::new();
    let mut current = foundation
        .node(normalized_path)
        .and_then(|node| node.parent_relative_path.clone());
    while let Some(key) = current {
        if key == anchor_key {
            break;
        }
        if let Some(target) = compare_tree_toggle_target(foundation, key.as_str()) {
            targets.push(target);
        }
        current = foundation
            .node(key.as_str())
            .and_then(|node| node.parent_relative_path.clone());
    }
    targets.reverse();
    targets
}

fn compare_tree_contains_path(
    foundation: &CompareFoundation,
    anchor_key: &str,
    relative_path: &str,
) -> bool {
    if anchor_key.is_empty() {
        return foundation.node(relative_path).is_some();
    }

    let mut current = Some(relative_path.to_string());
    while let Some(key) = current {
        if key == anchor_key {
            return true;
        }
        current = foundation
            .node(key.as_str())
            .and_then(|node| node.parent_relative_path.clone());
    }
    false
}

pub(crate) fn compare_tree_expansion_state(
    key: &str,
    path_depth: u16,
    expansion_overrides: &BTreeMap<String, bool>,
) -> bool {
    expansion_overrides
        .get(key)
        .copied()
        .unwrap_or(path_depth <= 1)
}

fn is_hidden_relative_path(relative_path: &str) -> bool {
    relative_path
        .trim_matches(|ch| ch == '/' || ch == '\\')
        .split(['/', '\\'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .any(|part| part.starts_with('.'))
}
