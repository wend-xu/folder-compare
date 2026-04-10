//! View model definitions for Slint binding.

use crate::compare_foundation::CompareFoundation;

/// One compare entry row rendered by the MVP list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareEntryRowViewModel {
    /// Relative path from compare roots.
    pub relative_path: String,
    /// Human-readable entry status.
    pub status: String,
    /// Human-readable detail summary.
    pub detail: String,
    /// Entry kind from compare result (`file`, `directory`, ...).
    pub entry_kind: String,
    /// Detail payload kind (`text-diff`, `type-mismatch`, ...).
    pub detail_kind: String,
    /// Whether this row can trigger detailed text diff.
    pub can_load_diff: bool,
    /// Why this row cannot trigger detailed diff (if any).
    pub diff_blocked_reason: Option<String>,
    /// Whether this row can trigger AI analysis.
    pub can_load_analysis: bool,
    /// Why this row cannot trigger AI analysis (if any).
    pub analysis_blocked_reason: Option<String>,
}

/// Compare result view model projected from `fc-core::CompareReport`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareResultViewModel {
    /// Summary text rendered in the header area.
    pub summary_text: String,
    /// Structured compare-data foundation for workspace projections.
    pub compare_foundation: CompareFoundation,
    /// Flat result rows for list rendering.
    pub entry_rows: Vec<CompareEntryRowViewModel>,
    /// Warning lines to show in warning area.
    pub warnings: Vec<String>,
    /// Indicates core report is truncated.
    pub truncated: bool,
}

/// One inline segment inside one compare-file content line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareFileTextSegmentViewModel {
    /// Visible text content for this segment.
    pub text: String,
    /// Semantic tone token used for restrained inline emphasis.
    pub tone: String,
}

/// One side-by-side compare row rendered inside Compare File View.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareFileRowViewModel {
    /// Row kind token (`context`, `modified`, `left-only`, `right-only`).
    pub row_kind: String,
    /// Restrained relation label rendered in the middle lane.
    pub relation_label: String,
    /// Relation tone token for lane/background styling.
    pub relation_tone: String,
    /// Left/base line number when present.
    pub left_line_no: Option<usize>,
    /// Right/target line number when present.
    pub right_line_no: Option<usize>,
    /// Full left/base line text.
    pub left_text: String,
    /// Full right/target line text.
    pub right_text: String,
    /// Structured left/base inline segments.
    pub left_segments: Vec<CompareFileTextSegmentViewModel>,
    /// Structured right/target inline segments.
    pub right_segments: Vec<CompareFileTextSegmentViewModel>,
    /// Whether the left/base side is only padding for alignment.
    pub left_padding: bool,
    /// Whether the right/target side is only padding for alignment.
    pub right_padding: bool,
    /// Whether this row remains a meaningful compare row for focus/copy affordances.
    pub focusable: bool,
}

/// Dedicated compare-file payload for compare-originated file tabs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompareFilePanelViewModel {
    /// Relative path currently rendered inside Compare File View.
    pub relative_path: String,
    /// Compact compare summary for helper/header strips.
    pub summary_text: String,
    /// Ordered visible compare rows.
    pub rows: Vec<CompareFileRowViewModel>,
    /// Optional warning emitted while building the compare-file payload.
    pub warning: Option<String>,
    /// Whether the underlying compare payload was truncated.
    pub truncated: bool,
}

/// Detailed diff panel payload for one selected row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffPanelViewModel {
    /// Relative path currently shown in the panel.
    pub relative_path: String,
    /// Human-readable summary for this detailed diff.
    pub summary_text: String,
    /// Detailed hunk list for line-level rendering.
    pub hunks: Vec<DiffHunkViewModel>,
    /// Optional warning emitted by `diff_text_file`.
    pub warning: Option<String>,
    /// Whether detailed diff output was truncated.
    pub truncated: bool,
}

/// One hunk in detailed diff panel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffHunkViewModel {
    /// Starting line number in old text (1-based).
    pub old_start: usize,
    /// Number of old lines covered by this hunk.
    pub old_len: usize,
    /// Starting line number in new text (1-based).
    pub new_start: usize,
    /// Number of new lines covered by this hunk.
    pub new_len: usize,
    /// Ordered lines within this hunk.
    pub lines: Vec<DiffLineViewModel>,
}

impl DiffHunkViewModel {
    /// Returns one unified-style hunk header.
    pub fn header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_len, self.new_start, self.new_len
        )
    }
}

/// One detailed diff line in panel output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffLineViewModel {
    /// Old-side line number if present.
    pub old_line_no: Option<usize>,
    /// New-side line number if present.
    pub new_line_no: Option<usize>,
    /// Kind label (`Context`, `Added`, `Removed`).
    pub kind: String,
    /// Line content.
    pub content: String,
}

impl DiffLineViewModel {
    /// Returns normalized lowercase kind token.
    pub fn kind_tag(&self) -> &'static str {
        match self.kind.as_str() {
            "Added" => "added",
            "Removed" => "removed",
            _ => "context",
        }
    }

    /// Returns one unified diff marker.
    pub fn marker(&self) -> &'static str {
        match self.kind_tag() {
            "added" => "+",
            "removed" => "-",
            _ => " ",
        }
    }
}

/// AI analysis panel payload for one selected diff.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AnalysisResultViewModel {
    /// Short headline for UI display.
    pub title: String,
    /// Risk level text (`low`, `medium`, `high`).
    pub risk_level: String,
    /// Human-readable reasoning text.
    pub rationale: String,
    /// Key bullet points.
    pub key_points: Vec<String>,
    /// Follow-up review suggestions.
    pub review_suggestions: Vec<String>,
}

impl AnalysisResultViewModel {
    /// Returns key points as one multiline bullet block.
    pub fn key_points_text(&self) -> String {
        bullet_lines(&self.key_points)
    }

    /// Returns review suggestions as one multiline bullet block.
    pub fn review_suggestions_text(&self) -> String {
        bullet_lines(&self.review_suggestions)
    }
}

fn bullet_lines(items: &[String]) -> String {
    items
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| format!("• {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}
