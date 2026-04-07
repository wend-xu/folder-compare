//! Structured compare-data foundation for workspace-level projections.

use crate::view_models::CompareEntryRowViewModel;
use fc_core::{CompareEntry, EntryDetail, EntryKind, EntryStatus, TextDetailDeferredReason};
use std::collections::BTreeMap;

const COMPARE_ROOT_DISPLAY_NAME: &str = "compare root";

/// Rust-owned compare focus anchor for future Compare View targets.
///
/// `None` means the compare root itself.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareFocusPath {
    relative_path: Option<String>,
}

impl CompareFocusPath {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn relative(relative_path: impl Into<String>) -> Self {
        let normalized = normalize_relative_path(relative_path.into().as_str());
        if normalized.is_empty() {
            Self::root()
        } else {
            Self {
                relative_path: Some(normalized),
            }
        }
    }

    pub fn as_relative_path(&self) -> Option<&str> {
        self.relative_path.as_deref()
    }

    pub fn raw_text(&self) -> String {
        self.relative_path.clone().unwrap_or_default()
    }

    pub fn is_root(&self) -> bool {
        self.relative_path.is_none()
    }
}

/// Side presence normalized for workspace projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompareSidePresence {
    pub left: bool,
    pub right: bool,
}

impl CompareSidePresence {
    fn merge(self, other: Self) -> Self {
        Self {
            left: self.left || other.left,
            right: self.right || other.right,
        }
    }
}

/// Base compare status normalized for workspace projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareBaseStatus {
    LeftOnly,
    RightOnly,
    Equal,
    Different,
    Pending,
    Skipped,
}

impl CompareBaseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LeftOnly => "left-only",
            Self::RightOnly => "right-only",
            Self::Equal => "equal",
            Self::Different => "different",
            Self::Pending => "pending",
            Self::Skipped => "skipped",
        }
    }

    fn aggregate(
        statuses: impl IntoIterator<Item = CompareBaseStatus>,
        fallback: CompareBaseStatus,
    ) -> Self {
        let mut iter = statuses.into_iter();
        let Some(first) = iter.next() else {
            return fallback;
        };
        if iter.all(|status| status == first) {
            first
        } else {
            Self::Different
        }
    }

    fn side_presence(self) -> CompareSidePresence {
        match self {
            Self::LeftOnly => CompareSidePresence {
                left: true,
                right: false,
            },
            Self::RightOnly => CompareSidePresence {
                left: false,
                right: true,
            },
            Self::Equal | Self::Different | Self::Pending | Self::Skipped => CompareSidePresence {
                left: true,
                right: true,
            },
        }
    }
}

/// Compare node kind normalized for workspace/data projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareNodeKind {
    Root,
    File,
    Directory,
    Symlink,
    Other,
}

impl CompareNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::File => "file",
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::Other => "other",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Root | Self::Directory => "Directory",
            Self::File => "File",
            Self::Symlink => "Symlink",
            Self::Other => "Special entry",
        }
    }

    pub fn is_directory_target(self) -> bool {
        matches!(self, Self::Root | Self::Directory)
    }
}

/// Normalized text-detail deferral reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareTextDeferredReason {
    LargeDirectoryMode,
    FileTooLarge,
}

impl CompareTextDeferredReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::LargeDirectoryMode => "large-directory mode",
            Self::FileTooLarge => "file too large",
        }
    }
}

/// Structured compare detail normalized for presenter/state projections.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CompareFoundationDetail {
    #[default]
    None,
    Message {
        message: String,
    },
    TypeMismatch {
        left: CompareNodeKind,
        right: CompareNodeKind,
    },
    FileComparison {
        left_size: u64,
        right_size: u64,
        content_checked: bool,
    },
    ContentComparisonDeferred,
    TextDetailDeferred {
        reason: CompareTextDeferredReason,
        left_size: u64,
        right_size: u64,
        max_text_file_size_bytes: u64,
        content_checked: bool,
    },
    TextDiffSummary {
        hunk_count: usize,
        added_lines: usize,
        removed_lines: usize,
        context_lines: usize,
    },
}

impl CompareFoundationDetail {
    pub fn kind_token(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Message { .. } => "message",
            Self::TypeMismatch { .. } => "type-mismatch",
            Self::FileComparison { .. } => "file-comparison",
            Self::ContentComparisonDeferred => "content-comparison-deferred",
            Self::TextDetailDeferred { .. } => "text-detail-deferred",
            Self::TextDiffSummary { .. } => "text-diff",
        }
    }

    pub fn legacy_text(&self, kind: CompareNodeKind) -> String {
        match self {
            Self::None => format!("kind={}", kind.as_str()),
            Self::Message { message } => message.clone(),
            Self::TypeMismatch { left, right } => {
                format!(
                    "type mismatch: left={} right={}",
                    left.as_str(),
                    right.as_str()
                )
            }
            Self::FileComparison {
                left_size,
                right_size,
                content_checked,
            } => format!(
                "file compare: left={}B right={}B content_checked={}",
                left_size, right_size, content_checked
            ),
            Self::ContentComparisonDeferred => "content comparison deferred".to_string(),
            Self::TextDetailDeferred {
                reason,
                left_size,
                right_size,
                max_text_file_size_bytes,
                content_checked,
            } => format!(
                "text detail deferred ({}): left={}B right={}B limit={}B content_checked={}",
                reason.label(),
                left_size,
                right_size,
                max_text_file_size_bytes,
                content_checked
            ),
            Self::TextDiffSummary {
                hunk_count,
                added_lines,
                removed_lines,
                context_lines,
            } => format!(
                "text summary: hunks={} +{} -{} ctx={}",
                hunk_count, added_lines, removed_lines, context_lines
            ),
        }
    }
}

/// Structured reason why detailed diff cannot load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareDiffUnavailableReason {
    NonFileEntry,
    SkippedEntry,
    TypeMismatch,
    NonTextFileComparison,
    DetailMessage(String),
}

impl CompareDiffUnavailableReason {
    pub fn to_text(&self) -> String {
        match self {
            Self::NonFileEntry => {
                "detailed text diff is only available for file entries".to_string()
            }
            Self::SkippedEntry => {
                "entry was skipped during compare and cannot load detailed diff".to_string()
            }
            Self::TypeMismatch => {
                "type mismatch entries cannot load detailed text diff".to_string()
            }
            Self::NonTextFileComparison => {
                "entry was compared as non-text/binary candidate, detailed text diff unavailable"
                    .to_string()
            }
            Self::DetailMessage(message) => {
                format!("entry detail indicates detailed text diff is unavailable: {message}")
            }
        }
    }
}

/// Structured reason why AI analysis cannot load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareAnalysisUnavailableReason {
    DiffUnavailable(CompareDiffUnavailableReason),
    RequiresChangedFile,
    #[allow(dead_code)]
    Custom(String),
}

impl CompareAnalysisUnavailableReason {
    pub fn to_text(&self) -> String {
        match self {
            Self::DiffUnavailable(reason) => reason.to_text(),
            Self::RequiresChangedFile => {
                "AI analysis is only available for changed file entries".to_string()
            }
            Self::Custom(message) => message.clone(),
        }
    }
}

/// Detailed diff availability normalized away from widget-row strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareDiffAvailability {
    Available,
    Unavailable(CompareDiffUnavailableReason),
}

/// AI analysis availability normalized away from widget-row strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareAnalysisAvailability {
    Available,
    Unavailable(CompareAnalysisUnavailableReason),
}

/// Normalized compare capabilities for file-view projections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareEntryCapabilities {
    pub diff: CompareDiffAvailability,
    pub analysis: CompareAnalysisAvailability,
}

impl CompareEntryCapabilities {
    fn default_for_kind(kind: CompareNodeKind) -> Self {
        if kind == CompareNodeKind::File {
            Self {
                diff: CompareDiffAvailability::Available,
                analysis: CompareAnalysisAvailability::Unavailable(
                    CompareAnalysisUnavailableReason::RequiresChangedFile,
                ),
            }
        } else {
            let reason = CompareDiffUnavailableReason::NonFileEntry;
            Self {
                diff: CompareDiffAvailability::Unavailable(reason.clone()),
                analysis: CompareAnalysisAvailability::Unavailable(
                    CompareAnalysisUnavailableReason::DiffUnavailable(reason),
                ),
            }
        }
    }

    pub fn can_load_diff(&self) -> bool {
        matches!(self.diff, CompareDiffAvailability::Available)
    }

    pub fn diff_blocked_reason_text(&self) -> Option<String> {
        match &self.diff {
            CompareDiffAvailability::Available => None,
            CompareDiffAvailability::Unavailable(reason) => Some(reason.to_text()),
        }
    }

    pub fn can_load_analysis(&self) -> bool {
        matches!(self.analysis, CompareAnalysisAvailability::Available)
    }

    pub fn analysis_blocked_reason_text(&self) -> Option<String> {
        match &self.analysis {
            CompareAnalysisAvailability::Available => None,
            CompareAnalysisAvailability::Unavailable(reason) => Some(reason.to_text()),
        }
    }
}

/// One normalized compare node keyed by relative path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareFoundationNode {
    pub relative_path: String,
    pub display_name: String,
    pub parent_relative_path: Option<String>,
    pub kind: CompareNodeKind,
    pub path_depth: u16,
    pub source_index: Option<usize>,
    pub side_presence: CompareSidePresence,
    pub base_status: CompareBaseStatus,
    pub detail: CompareFoundationDetail,
    pub capabilities: CompareEntryCapabilities,
    pub child_relative_paths: Vec<String>,
}

impl CompareFoundationNode {
    pub fn is_directory_target(&self) -> bool {
        self.kind.is_directory_target()
    }

    pub fn has_compare_entry(&self) -> bool {
        self.source_index.is_some()
    }
}

/// Stable child projection for future Compare View immediate-children surfaces.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareImmediateChild {
    pub relative_path: String,
    pub display_name: String,
    pub kind: CompareNodeKind,
    pub side_presence: CompareSidePresence,
    pub base_status: CompareBaseStatus,
    pub has_children: bool,
    pub source_index: Option<usize>,
}

/// Canonical compare-data foundation owned by `fc-ui-slint`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareFoundation {
    nodes: BTreeMap<String, CompareFoundationNode>,
    ordered_source_paths: Vec<String>,
}

impl Default for CompareFoundation {
    fn default() -> Self {
        Self::empty()
    }
}

impl CompareFoundation {
    pub fn empty() -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert(
            String::new(),
            CompareFoundationNode {
                relative_path: String::new(),
                display_name: COMPARE_ROOT_DISPLAY_NAME.to_string(),
                parent_relative_path: None,
                kind: CompareNodeKind::Root,
                path_depth: 0,
                source_index: None,
                side_presence: CompareSidePresence::default(),
                base_status: CompareBaseStatus::Equal,
                detail: CompareFoundationDetail::None,
                capabilities: CompareEntryCapabilities::default_for_kind(CompareNodeKind::Root),
                child_relative_paths: Vec::new(),
            },
        );
        Self {
            nodes,
            ordered_source_paths: Vec::new(),
        }
    }

    pub fn from_compare_entries(entries: &[CompareEntry]) -> Self {
        let mut foundation = Self::empty();
        foundation.ordered_source_paths = vec![String::new(); entries.len()];

        for (source_index, entry) in entries.iter().enumerate() {
            foundation.insert_compare_entry(source_index, entry);
        }

        for node in foundation.nodes.values_mut() {
            node.child_relative_paths.sort();
            node.child_relative_paths.dedup();
        }

        foundation.finalize_node(String::new().as_str());
        foundation
    }

    pub fn node(&self, relative_path: &str) -> Option<&CompareFoundationNode> {
        let normalized = normalize_relative_path(relative_path);
        self.nodes.get(normalized.as_str())
    }

    pub fn source_node(&self, source_index: usize) -> Option<&CompareFoundationNode> {
        self.ordered_source_paths
            .get(source_index)
            .and_then(|path| self.nodes.get(path.as_str()))
    }

    pub fn source_nodes(&self) -> impl Iterator<Item = &CompareFoundationNode> {
        self.ordered_source_paths
            .iter()
            .filter(|path| !path.is_empty())
            .filter_map(|path| self.nodes.get(path.as_str()))
    }

    #[allow(dead_code)]
    pub fn source_entry_count(&self) -> usize {
        self.ordered_source_paths
            .iter()
            .filter(|path| !path.is_empty())
            .count()
    }

    pub fn source_index_for_relative_path(&self, relative_path: &str) -> Option<usize> {
        self.node(relative_path).and_then(|node| node.source_index)
    }

    pub fn clamp_compare_focus_path(&self, focus: &CompareFocusPath) -> CompareFocusPath {
        if focus.is_root() {
            return CompareFocusPath::root();
        }

        let mut current = focus
            .as_relative_path()
            .map(CompareFocusPath::relative)
            .map(|value| value.raw_text());
        while let Some(candidate) = current {
            if self
                .node(candidate.as_str())
                .is_some_and(|node| node.is_directory_target())
            {
                return CompareFocusPath::relative(candidate);
            }
            current = parent_relative_path(candidate.as_str());
        }
        CompareFocusPath::root()
    }

    #[allow(dead_code)]
    pub fn parent_compare_focus_path(&self, focus: &CompareFocusPath) -> CompareFocusPath {
        let Some(path) = focus.as_relative_path() else {
            return CompareFocusPath::root();
        };
        let Some(parent) = parent_relative_path(path) else {
            return CompareFocusPath::root();
        };
        self.clamp_compare_focus_path(&CompareFocusPath::relative(parent))
    }

    #[allow(dead_code)]
    pub fn immediate_children(&self, focus: &CompareFocusPath) -> Vec<CompareImmediateChild> {
        let focus = self.clamp_compare_focus_path(focus);
        let key = focus.as_relative_path().unwrap_or_default();
        let Some(node) = self.nodes.get(key) else {
            return Vec::new();
        };

        node.child_relative_paths
            .iter()
            .filter_map(|child_path| self.nodes.get(child_path.as_str()))
            .map(|child| CompareImmediateChild {
                relative_path: child.relative_path.clone(),
                display_name: child.display_name.clone(),
                kind: child.kind,
                side_presence: child.side_presence,
                base_status: child.base_status,
                has_children: !child.child_relative_paths.is_empty(),
                source_index: child.source_index,
            })
            .collect()
    }

    pub fn project_legacy_entry_rows(&self) -> Vec<CompareEntryRowViewModel> {
        self.source_nodes()
            .map(project_legacy_entry_row)
            .collect::<Vec<_>>()
    }

    fn insert_compare_entry(&mut self, source_index: usize, entry: &CompareEntry) {
        let normalized_path = normalize_relative_path(entry.relative_path.as_str());
        if normalized_path.is_empty() {
            return;
        }

        if let Some(slot) = self.ordered_source_paths.get_mut(source_index) {
            *slot = normalized_path.clone();
        }

        let components = path_components(normalized_path.as_str());
        if components.is_empty() {
            return;
        }

        let mut parent_key = String::new();
        for (component_index, component) in components.iter().enumerate() {
            let is_last = component_index + 1 == components.len();
            let key = join_path_components(&components[..=component_index]);
            let default_kind = if is_last {
                compare_kind_from_core(entry.kind)
            } else {
                CompareNodeKind::Directory
            };

            self.nodes
                .entry(key.clone())
                .or_insert_with(|| CompareFoundationNode {
                    relative_path: key.clone(),
                    display_name: (*component).to_string(),
                    parent_relative_path: (!parent_key.is_empty()).then(|| parent_key.clone()),
                    kind: default_kind,
                    path_depth: u16::try_from(component_index + 1).unwrap_or(u16::MAX),
                    source_index: None,
                    side_presence: CompareSidePresence::default(),
                    base_status: CompareBaseStatus::Equal,
                    detail: CompareFoundationDetail::None,
                    capabilities: CompareEntryCapabilities::default_for_kind(default_kind),
                    child_relative_paths: Vec::new(),
                });

            if let Some(parent) = self.nodes.get_mut(parent_key.as_str()) {
                parent.child_relative_paths.push(key.clone());
            }

            if is_last {
                let status = compare_status_from_core(entry.status);
                let detail = compare_detail_from_core(&entry.detail);
                let capabilities = compare_capabilities_from_core(entry);
                let node = self
                    .nodes
                    .get_mut(key.as_str())
                    .expect("compare foundation node must exist");
                node.kind = default_kind;
                node.source_index = Some(source_index);
                node.side_presence = status.side_presence();
                node.base_status = status;
                node.detail = detail;
                node.capabilities = capabilities;
            }

            parent_key = key;
        }
    }

    fn finalize_node(&mut self, key: &str) -> (CompareSidePresence, CompareBaseStatus) {
        let child_keys = self
            .nodes
            .get(key)
            .expect("compare foundation node must exist")
            .child_relative_paths
            .clone();
        let child_state = child_keys
            .iter()
            .map(|child_key| self.finalize_node(child_key.as_str()))
            .collect::<Vec<_>>();
        let aggregated_presence = child_state
            .iter()
            .fold(CompareSidePresence::default(), |acc, (presence, _)| {
                acc.merge(*presence)
            });
        let aggregated_status = CompareBaseStatus::aggregate(
            child_state.iter().map(|(_, status)| *status),
            CompareBaseStatus::Equal,
        );

        let Some(node) = self.nodes.get_mut(key) else {
            return (aggregated_presence, aggregated_status);
        };
        if !node.has_compare_entry() || node.kind == CompareNodeKind::Root {
            node.side_presence = aggregated_presence;
            node.base_status = aggregated_status;
        }
        (node.side_presence, node.base_status)
    }
}

fn project_legacy_entry_row(node: &CompareFoundationNode) -> CompareEntryRowViewModel {
    CompareEntryRowViewModel {
        relative_path: node.relative_path.clone(),
        status: node.base_status.as_str().to_string(),
        detail: node.detail.legacy_text(node.kind),
        entry_kind: node.kind.as_str().to_string(),
        detail_kind: node.detail.kind_token().to_string(),
        can_load_diff: node.capabilities.can_load_diff(),
        diff_blocked_reason: node.capabilities.diff_blocked_reason_text(),
        can_load_analysis: node.capabilities.can_load_analysis(),
        analysis_blocked_reason: node.capabilities.analysis_blocked_reason_text(),
    }
}

fn compare_capabilities_from_core(entry: &CompareEntry) -> CompareEntryCapabilities {
    let diff = if entry.kind != EntryKind::File {
        CompareDiffAvailability::Unavailable(CompareDiffUnavailableReason::NonFileEntry)
    } else {
        match entry.status {
            EntryStatus::LeftOnly | EntryStatus::RightOnly | EntryStatus::Equal => {
                CompareDiffAvailability::Available
            }
            EntryStatus::Skipped => {
                CompareDiffAvailability::Unavailable(CompareDiffUnavailableReason::SkippedEntry)
            }
            EntryStatus::Different | EntryStatus::Pending => match &entry.detail {
                EntryDetail::TypeMismatch { .. } => {
                    CompareDiffAvailability::Unavailable(CompareDiffUnavailableReason::TypeMismatch)
                }
                EntryDetail::FileComparison { .. } => CompareDiffAvailability::Unavailable(
                    CompareDiffUnavailableReason::NonTextFileComparison,
                ),
                EntryDetail::Message(message) => CompareDiffAvailability::Unavailable(
                    CompareDiffUnavailableReason::DetailMessage(message.clone()),
                ),
                EntryDetail::None
                | EntryDetail::ContentComparisonDeferred
                | EntryDetail::TextDetailDeferred { .. }
                | EntryDetail::TextDiff(_) => CompareDiffAvailability::Available,
            },
        }
    };

    let analysis = match &diff {
        CompareDiffAvailability::Unavailable(reason) => CompareAnalysisAvailability::Unavailable(
            CompareAnalysisUnavailableReason::DiffUnavailable(reason.clone()),
        ),
        CompareDiffAvailability::Available => {
            if entry.status == EntryStatus::Different {
                CompareAnalysisAvailability::Available
            } else {
                CompareAnalysisAvailability::Unavailable(
                    CompareAnalysisUnavailableReason::RequiresChangedFile,
                )
            }
        }
    };

    CompareEntryCapabilities { diff, analysis }
}

fn compare_kind_from_core(kind: EntryKind) -> CompareNodeKind {
    match kind {
        EntryKind::File => CompareNodeKind::File,
        EntryKind::Directory => CompareNodeKind::Directory,
        EntryKind::Symlink => CompareNodeKind::Symlink,
        EntryKind::Other => CompareNodeKind::Other,
    }
}

fn compare_status_from_core(status: EntryStatus) -> CompareBaseStatus {
    match status {
        EntryStatus::LeftOnly => CompareBaseStatus::LeftOnly,
        EntryStatus::RightOnly => CompareBaseStatus::RightOnly,
        EntryStatus::Equal => CompareBaseStatus::Equal,
        EntryStatus::Different => CompareBaseStatus::Different,
        EntryStatus::Pending => CompareBaseStatus::Pending,
        EntryStatus::Skipped => CompareBaseStatus::Skipped,
    }
}

fn compare_detail_from_core(detail: &EntryDetail) -> CompareFoundationDetail {
    match detail {
        EntryDetail::None => CompareFoundationDetail::None,
        EntryDetail::Message(message) => CompareFoundationDetail::Message {
            message: message.clone(),
        },
        EntryDetail::TypeMismatch { left, right } => CompareFoundationDetail::TypeMismatch {
            left: compare_kind_from_core(*left),
            right: compare_kind_from_core(*right),
        },
        EntryDetail::FileComparison {
            left_size,
            right_size,
            content_checked,
        } => CompareFoundationDetail::FileComparison {
            left_size: *left_size,
            right_size: *right_size,
            content_checked: *content_checked,
        },
        EntryDetail::ContentComparisonDeferred => {
            CompareFoundationDetail::ContentComparisonDeferred
        }
        EntryDetail::TextDetailDeferred {
            reason,
            left_size,
            right_size,
            max_text_file_size_bytes,
            content_checked,
        } => CompareFoundationDetail::TextDetailDeferred {
            reason: match reason {
                TextDetailDeferredReason::LargeDirectoryMode => {
                    CompareTextDeferredReason::LargeDirectoryMode
                }
                TextDetailDeferredReason::FileTooLarge => CompareTextDeferredReason::FileTooLarge,
            },
            left_size: *left_size,
            right_size: *right_size,
            max_text_file_size_bytes: *max_text_file_size_bytes,
            content_checked: *content_checked,
        },
        EntryDetail::TextDiff(summary) => CompareFoundationDetail::TextDiffSummary {
            hunk_count: summary.hunk_count,
            added_lines: summary.added_lines,
            removed_lines: summary.removed_lines,
            context_lines: summary.context_lines,
        },
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

fn parent_relative_path(relative_path: &str) -> Option<String> {
    let normalized = normalize_relative_path(relative_path);
    if normalized.is_empty() {
        return None;
    }
    normalized
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
}

#[cfg(test)]
pub(crate) fn foundation_from_legacy_rows(rows: &[CompareEntryRowViewModel]) -> CompareFoundation {
    let mut foundation = CompareFoundation::empty();
    foundation.ordered_source_paths = vec![String::new(); rows.len()];

    for (source_index, row) in rows.iter().enumerate() {
        let normalized_path = normalize_relative_path(&row.relative_path);
        if normalized_path.is_empty() {
            continue;
        }
        if let Some(slot) = foundation.ordered_source_paths.get_mut(source_index) {
            *slot = normalized_path.clone();
        }

        let components = path_components(normalized_path.as_str());
        let mut parent_key = String::new();
        for (component_index, component) in components.iter().enumerate() {
            let is_last = component_index + 1 == components.len();
            let key = join_path_components(&components[..=component_index]);
            let kind = if is_last {
                compare_kind_from_legacy(row.entry_kind.as_str())
            } else {
                CompareNodeKind::Directory
            };

            foundation
                .nodes
                .entry(key.clone())
                .or_insert_with(|| CompareFoundationNode {
                    relative_path: key.clone(),
                    display_name: (*component).to_string(),
                    parent_relative_path: (!parent_key.is_empty()).then(|| parent_key.clone()),
                    kind,
                    path_depth: u16::try_from(component_index + 1).unwrap_or(u16::MAX),
                    source_index: None,
                    side_presence: CompareSidePresence::default(),
                    base_status: CompareBaseStatus::Equal,
                    detail: CompareFoundationDetail::None,
                    capabilities: CompareEntryCapabilities::default_for_kind(kind),
                    child_relative_paths: Vec::new(),
                });

            if let Some(parent) = foundation.nodes.get_mut(parent_key.as_str()) {
                parent.child_relative_paths.push(key.clone());
            }

            if is_last {
                let status = compare_status_from_legacy(row.status.as_str());
                let diff = if row.can_load_diff {
                    CompareDiffAvailability::Available
                } else {
                    CompareDiffAvailability::Unavailable(
                        CompareDiffUnavailableReason::DetailMessage(
                            row.diff_blocked_reason.clone().unwrap_or_else(|| {
                                "selected row does not support detailed text diff".to_string()
                            }),
                        ),
                    )
                };
                let analysis = if row.can_load_analysis {
                    CompareAnalysisAvailability::Available
                } else {
                    CompareAnalysisAvailability::Unavailable(parse_legacy_analysis_reason(
                        row.analysis_blocked_reason
                            .clone()
                            .or_else(|| row.diff_blocked_reason.clone()),
                    ))
                };
                let detail = parse_legacy_detail(row);
                let node = foundation
                    .nodes
                    .get_mut(key.as_str())
                    .expect("legacy compare foundation node must exist");
                node.kind = kind;
                node.source_index = Some(source_index);
                node.side_presence = status.side_presence();
                node.base_status = status;
                node.detail = detail;
                node.capabilities = CompareEntryCapabilities { diff, analysis };
            }

            parent_key = key;
        }
    }

    for node in foundation.nodes.values_mut() {
        node.child_relative_paths.sort();
        node.child_relative_paths.dedup();
    }
    foundation.finalize_node("");
    foundation
}

#[cfg(test)]
fn compare_kind_from_legacy(kind: &str) -> CompareNodeKind {
    match kind.trim().to_ascii_lowercase().as_str() {
        "directory" => CompareNodeKind::Directory,
        "symlink" => CompareNodeKind::Symlink,
        "other" => CompareNodeKind::Other,
        _ => CompareNodeKind::File,
    }
}

#[cfg(test)]
fn compare_status_from_legacy(status: &str) -> CompareBaseStatus {
    match status.trim().to_ascii_lowercase().as_str() {
        "left-only" => CompareBaseStatus::LeftOnly,
        "right-only" => CompareBaseStatus::RightOnly,
        "equal" => CompareBaseStatus::Equal,
        "pending" => CompareBaseStatus::Pending,
        "skipped" => CompareBaseStatus::Skipped,
        _ => CompareBaseStatus::Different,
    }
}

#[cfg(test)]
fn parse_legacy_analysis_reason(reason: Option<String>) -> CompareAnalysisUnavailableReason {
    match reason
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some("AI analysis is only available for changed file entries") => {
            CompareAnalysisUnavailableReason::RequiresChangedFile
        }
        Some(message) => CompareAnalysisUnavailableReason::Custom(message.to_string()),
        None => CompareAnalysisUnavailableReason::RequiresChangedFile,
    }
}

#[cfg(test)]
fn parse_legacy_detail(row: &CompareEntryRowViewModel) -> CompareFoundationDetail {
    match row.detail_kind.as_str() {
        "type-mismatch" => CompareFoundationDetail::TypeMismatch {
            left: extract_prefixed_token(&row.detail, "left=")
                .map(|value| compare_kind_from_legacy(value.as_str()))
                .unwrap_or(CompareNodeKind::Other),
            right: extract_prefixed_token(&row.detail, "right=")
                .map(|value| compare_kind_from_legacy(value.as_str()))
                .unwrap_or(CompareNodeKind::Other),
        },
        "file-comparison" => CompareFoundationDetail::FileComparison {
            left_size: parse_bytes_token(&row.detail, "left=").unwrap_or(0),
            right_size: parse_bytes_token(&row.detail, "right=").unwrap_or(0),
            content_checked: extract_prefixed_token(&row.detail, "content_checked=")
                .is_some_and(|value| value.eq_ignore_ascii_case("true")),
        },
        "content-comparison-deferred" => CompareFoundationDetail::ContentComparisonDeferred,
        "text-detail-deferred" => CompareFoundationDetail::TextDetailDeferred {
            reason: if row.detail.contains("large-directory mode") {
                CompareTextDeferredReason::LargeDirectoryMode
            } else {
                CompareTextDeferredReason::FileTooLarge
            },
            left_size: parse_bytes_token(&row.detail, "left=").unwrap_or(0),
            right_size: parse_bytes_token(&row.detail, "right=").unwrap_or(0),
            max_text_file_size_bytes: parse_bytes_token(&row.detail, "limit=").unwrap_or(0),
            content_checked: extract_prefixed_token(&row.detail, "content_checked=")
                .is_some_and(|value| value.eq_ignore_ascii_case("true")),
        },
        "text-diff" => CompareFoundationDetail::TextDiffSummary {
            hunk_count: extract_prefixed_token(&row.detail, "hunks=")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0),
            added_lines: extract_prefixed_token(&row.detail, "+")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0),
            removed_lines: extract_prefixed_token(&row.detail, "-")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0),
            context_lines: extract_prefixed_token(&row.detail, "ctx=")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0),
        },
        "message" => CompareFoundationDetail::Message {
            message: row.detail.clone(),
        },
        "none" => CompareFoundationDetail::None,
        _ if row.detail.trim().is_empty() => CompareFoundationDetail::None,
        _ => CompareFoundationDetail::Message {
            message: row.detail.clone(),
        },
    }
}

#[cfg(test)]
fn extract_prefixed_token(text: &str, prefix: &str) -> Option<String> {
    text.split_whitespace()
        .find_map(|part| part.trim_matches('|').strip_prefix(prefix))
        .map(|value| value.trim_matches(|ch| ch == ',' || ch == ';').to_string())
}

#[cfg(test)]
fn parse_bytes_token(text: &str, prefix: &str) -> Option<u64> {
    extract_prefixed_token(text, prefix)
        .map(|value| value.trim_end_matches('B').to_string())
        .and_then(|value| value.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fc_core::TextDiffSummary;

    #[test]
    fn foundation_builds_root_and_immediate_children() {
        let entries = vec![
            CompareEntry::new("src/main.rs", EntryKind::File, EntryStatus::Different).with_detail(
                EntryDetail::TextDiff(TextDiffSummary {
                    hunk_count: 1,
                    added_lines: 2,
                    removed_lines: 1,
                    context_lines: 3,
                }),
            ),
            CompareEntry::new("src/bin/tool.rs", EntryKind::File, EntryStatus::LeftOnly),
            CompareEntry::new("Cargo.toml", EntryKind::File, EntryStatus::Equal),
        ];

        let foundation = CompareFoundation::from_compare_entries(&entries);
        let root_children = foundation.immediate_children(&CompareFocusPath::root());
        assert_eq!(
            root_children
                .iter()
                .map(|child| child.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["Cargo.toml", "src"]
        );

        let src_children = foundation.immediate_children(&CompareFocusPath::relative("src"));
        assert_eq!(
            src_children
                .iter()
                .map(|child| child.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["src/bin", "src/main.rs"]
        );
        assert_eq!(
            foundation
                .node("src")
                .expect("src node should exist")
                .base_status,
            CompareBaseStatus::Different
        );
    }

    #[test]
    fn compare_focus_clamps_to_existing_directory_ancestor() {
        let entries = vec![CompareEntry::new(
            "src/bin/main.rs",
            EntryKind::File,
            EntryStatus::Different,
        )];
        let foundation = CompareFoundation::from_compare_entries(&entries);

        assert_eq!(
            foundation.clamp_compare_focus_path(&CompareFocusPath::relative("src/bin/main.rs")),
            CompareFocusPath::relative("src/bin")
        );
        assert_eq!(
            foundation.clamp_compare_focus_path(&CompareFocusPath::relative("missing/path")),
            CompareFocusPath::root()
        );
    }

    #[test]
    fn legacy_entry_rows_preserve_source_order_and_capabilities() {
        let entries = vec![
            CompareEntry::new("docs/readme.md", EntryKind::File, EntryStatus::Different)
                .with_detail(EntryDetail::TextDiff(TextDiffSummary {
                    hunk_count: 2,
                    added_lines: 5,
                    removed_lines: 3,
                    context_lines: 8,
                })),
            CompareEntry::new("assets/logo.png", EntryKind::File, EntryStatus::Different)
                .with_detail(EntryDetail::FileComparison {
                    left_size: 10,
                    right_size: 12,
                    content_checked: true,
                }),
        ];

        let rows = CompareFoundation::from_compare_entries(&entries).project_legacy_entry_rows();
        assert_eq!(rows[0].relative_path, "docs/readme.md");
        assert_eq!(rows[0].detail_kind, "text-diff");
        assert!(rows[0].can_load_diff);
        assert!(rows[0].can_load_analysis);

        assert_eq!(rows[1].relative_path, "assets/logo.png");
        assert_eq!(rows[1].detail_kind, "file-comparison");
        assert!(!rows[1].can_load_diff);
        assert!(!rows[1].can_load_analysis);
    }
}
