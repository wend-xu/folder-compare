//! Window-local shared context-menu helpers for non-input safe UI surfaces.

use std::rc::Rc;

pub const CONTEXT_MENU_COPY_ACTION_ID: &str = "copy";
pub const CONTEXT_MENU_COPY_SUMMARY_ACTION_ID: &str = "copy-summary";
pub const MAX_CONTEXT_MENU_CUSTOM_ACTIONS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuActionSpec {
    pub label: String,
    pub action_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuCustomActionDescriptor {
    pub label: String,
    pub action_id: String,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct ContextMenuCustomAction {
    pub descriptor: ContextMenuCustomActionDescriptor,
    pub handler: Rc<dyn Fn(ContextMenuInvocation)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuInvocation {
    pub target_token: String,
    pub action_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextMenuTextPayload {
    pub copy_text: String,
    pub summary_text: String,
    pub copy_feedback_label: String,
    pub summary_feedback_label: String,
}

impl ContextMenuTextPayload {
    pub fn copy_enabled(&self) -> bool {
        !self.copy_text.trim().is_empty()
    }

    pub fn summary_enabled(&self) -> bool {
        !self.summary_text.trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuBuildResult {
    pub actions: Vec<ContextMenuActionSpec>,
    pub truncated_custom_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextMenuSyncState {
    pub selected_row: Option<usize>,
    pub running: bool,
    pub diff_loading: bool,
    pub analysis_loading: bool,
}

pub fn build_action_specs(
    payload: &ContextMenuTextPayload,
    custom_actions: &[ContextMenuCustomActionDescriptor],
) -> ContextMenuBuildResult {
    let mut actions = Vec::new();
    actions.push(ContextMenuActionSpec {
        label: "Copy".to_string(),
        action_id: CONTEXT_MENU_COPY_ACTION_ID.to_string(),
        enabled: payload.copy_enabled(),
    });
    actions.push(ContextMenuActionSpec {
        label: "Copy Summary".to_string(),
        action_id: CONTEXT_MENU_COPY_SUMMARY_ACTION_ID.to_string(),
        enabled: payload.summary_enabled(),
    });

    let valid_custom_actions = custom_actions
        .iter()
        .filter(|action| !action.label.trim().is_empty() && !action.action_id.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let truncated_custom_count = valid_custom_actions
        .len()
        .saturating_sub(MAX_CONTEXT_MENU_CUSTOM_ACTIONS);
    actions.extend(
        valid_custom_actions
            .into_iter()
            .take(MAX_CONTEXT_MENU_CUSTOM_ACTIONS)
            .map(|action| ContextMenuActionSpec {
                label: action.label,
                action_id: action.action_id,
                enabled: action.enabled,
            }),
    );

    ContextMenuBuildResult {
        actions,
        truncated_custom_count,
    }
}

pub fn should_close_for_sync_transition(
    previous: ContextMenuSyncState,
    next: ContextMenuSyncState,
) -> bool {
    previous.selected_row != next.selected_row
        || (!previous.running && next.running)
        || (!previous.diff_loading && next.diff_loading)
        || (!previous.analysis_loading && next.analysis_loading)
}

pub fn build_results_row_payload(
    relative_path: &str,
    status_token: &str,
    detail: &str,
    unavailable: bool,
) -> ContextMenuTextPayload {
    let path = normalize_text(relative_path).unwrap_or_else(|| "Unknown file".to_string());
    let status = display_status_label(status_token);
    let detail_text = if unavailable {
        "Detailed diff unavailable".to_string()
    } else {
        normalize_text(detail).unwrap_or_else(|| "No detail".to_string())
    };
    let summary = join_summary_parts(&[
        Some(status.clone()),
        Some(path.clone()),
        Some(sentence_excerpt(&detail_text, 120)),
    ]);

    ContextMenuTextPayload {
        copy_text: join_blocks(&[
            ("File", Some(path)),
            ("Status", Some(status)),
            ("Detail", Some(detail_text)),
        ]),
        summary_text: join_blocks(&[("Result Summary", Some(summary))]),
        copy_feedback_label: "Result".to_string(),
        summary_feedback_label: "Result Summary".to_string(),
    }
}

pub fn build_workspace_header_payload(
    relative_path: &str,
    mode_label: &str,
    status_label: &str,
    summary_text: &str,
    hint_text: &str,
) -> ContextMenuTextPayload {
    let path = normalize_text(relative_path).unwrap_or_else(|| "No file selected".to_string());
    let mode = normalize_text(mode_label).unwrap_or_else(|| "Unknown view".to_string());
    let status = normalize_text(status_label).unwrap_or_else(|| "Unavailable".to_string());
    let summary =
        normalize_text(summary_text).unwrap_or_else(|| "No summary available.".to_string());
    let hint = normalize_text(hint_text);
    let summary_line = join_summary_parts(&[
        Some(path.clone()),
        Some(mode.clone()),
        Some(status.clone()),
        Some(sentence_excerpt(&summary, 120)),
    ]);

    ContextMenuTextPayload {
        copy_text: join_blocks(&[
            ("File", Some(path)),
            ("View", Some(mode)),
            ("Status", Some(status)),
            ("Summary", Some(summary)),
            ("Hint", hint.clone()),
        ]),
        summary_text: join_blocks(&[("File Context Summary", Some(summary_line)), ("Hint", hint)]),
        copy_feedback_label: "File Context".to_string(),
        summary_feedback_label: "File Context Summary".to_string(),
    }
}

pub fn build_analysis_section_payload(
    section_label: &str,
    title: &str,
    body: &str,
    copy_value: &str,
) -> ContextMenuTextPayload {
    let section = normalize_text(section_label).unwrap_or_else(|| "Analysis".to_string());
    let title = normalize_text(title);
    let body = normalize_text(body);
    let fallback_copy = join_blocks(&[(section.as_str(), title.clone().or_else(|| body.clone()))]);
    let copy_text = normalize_text(copy_value).unwrap_or(fallback_copy);
    let summary_source = title
        .clone()
        .or_else(|| body.as_deref().map(|value| sentence_excerpt(value, 140)));
    let summary_label = format!("{section} Summary");
    let summary_text = join_blocks(&[(summary_label.as_str(), summary_source)]);

    ContextMenuTextPayload {
        copy_text,
        summary_text,
        copy_feedback_label: section.clone(),
        summary_feedback_label: format!("{section} Summary"),
    }
}

fn join_summary_parts(parts: &[Option<String>]) -> String {
    parts
        .iter()
        .flatten()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
}

fn join_blocks(blocks: &[(&str, Option<String>)]) -> String {
    blocks
        .iter()
        .filter_map(|(label, value)| {
            value
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(|value| format!("{label}\n{value}"))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn normalize_text(raw: &str) -> Option<String> {
    let text = raw.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn display_status_label(status_token: &str) -> String {
    match status_token.trim() {
        "different" => "Changed".to_string(),
        "equal" => "Equal".to_string(),
        "left-only" => "Left Only".to_string(),
        "right-only" => "Right Only".to_string(),
        "pending" => "Pending".to_string(),
        "skipped" => "Unavailable".to_string(),
        other if !other.trim().is_empty() => other.trim().to_string(),
        _ => "Unavailable".to_string(),
    }
}

fn sentence_excerpt(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut boundary = None;
    for (idx, ch) in trimmed.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            boundary = Some(idx + ch.len_utf8());
            break;
        }
    }

    let excerpt = boundary
        .map(|idx| trimmed[..idx].trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(trimmed);

    if excerpt.chars().count() <= max_chars {
        excerpt.to_string()
    } else {
        abbreviate_middle(excerpt, max_chars, max_chars.saturating_sub(24), 20)
    }
}

fn abbreviate_middle(text: &str, max_chars: usize, head_chars: usize, tail_chars: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars || chars.len() <= head_chars + tail_chars + 1 {
        return text.to_string();
    }
    let head = chars[..head_chars].iter().collect::<String>();
    let tail = chars[chars.len() - tail_chars..].iter().collect::<String>();
    format!("{head}…{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_action_specs_truncates_custom_actions_after_ten() {
        let payload = ContextMenuTextPayload {
            copy_text: "full".to_string(),
            summary_text: "summary".to_string(),
            copy_feedback_label: "Copy".to_string(),
            summary_feedback_label: "Summary".to_string(),
        };
        let custom_actions = (0..12)
            .map(|index| ContextMenuCustomActionDescriptor {
                label: format!("Custom {index}"),
                action_id: format!("custom-{index}"),
                enabled: index % 2 == 0,
            })
            .collect::<Vec<_>>();

        let result = build_action_specs(&payload, &custom_actions);

        assert_eq!(result.actions.len(), 12);
        assert_eq!(result.truncated_custom_count, 2);
        assert_eq!(result.actions[0].action_id, CONTEXT_MENU_COPY_ACTION_ID);
        assert_eq!(
            result.actions[1].action_id,
            CONTEXT_MENU_COPY_SUMMARY_ACTION_ID
        );
        assert_eq!(
            result
                .actions
                .last()
                .map(|action| action.action_id.as_str()),
            Some("custom-9")
        );
    }

    #[test]
    fn results_row_payload_exports_full_and_summary_copy_text() {
        let payload =
            build_results_row_payload("src/main.rs", "different", "hunks=2 +4 -1 ctx=8", false);

        assert!(payload.copy_text.contains("File\nsrc/main.rs"));
        assert!(payload.copy_text.contains("Status\nChanged"));
        assert!(payload.summary_text.contains("Result Summary"));
        assert!(payload.summary_text.contains("src/main.rs"));
    }

    #[test]
    fn sync_transition_closes_on_selected_row_or_busy_start() {
        let previous = ContextMenuSyncState {
            selected_row: Some(1),
            running: false,
            diff_loading: false,
            analysis_loading: false,
        };
        let next_selected_row = ContextMenuSyncState {
            selected_row: Some(2),
            ..previous
        };
        let next_running = ContextMenuSyncState {
            running: true,
            ..previous
        };

        assert!(should_close_for_sync_transition(
            previous,
            next_selected_row
        ));
        assert!(should_close_for_sync_transition(previous, next_running));
        assert!(!should_close_for_sync_transition(previous, previous));
    }
}
