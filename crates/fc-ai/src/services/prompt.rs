//! Provider-neutral prompt payload construction.

use crate::domain::types::{AnalysisTask, AnalyzeDiffRequest, PromptPayload};

/// Builds provider-neutral prompt payload from analysis request.
pub(crate) fn build_prompt_payload(req: &AnalyzeDiffRequest) -> PromptPayload {
    let task_line = match req.task {
        AnalysisTask::Summary => "Task: summarize the code change clearly.",
        AnalysisTask::RiskReview => "Task: assess risk and call out potential regressions.",
        AnalysisTask::ReviewComments => "Task: provide review comments and actionable suggestions.",
    };
    let path_line = req.relative_path.as_deref().unwrap_or("(unknown path)");
    let language_line = req.language_hint.as_deref().unwrap_or("(unknown language)");
    let summary_line = match req.summary.as_ref() {
        Some(summary) => format!(
            "Summary Stats: hunks={}, added={}, removed={}, context={}",
            summary.hunk_count, summary.added_lines, summary.removed_lines, summary.context_lines
        ),
        None => "Summary Stats: (none)".to_string(),
    };
    let truncation_line = req.truncation_note.as_deref().unwrap_or("(none)");
    let output_contract = "Return strict JSON object only with keys: risk_level (low|medium|high), title, rationale, key_points (string[]), review_suggestions (string[]).";

    let user_prompt = format!(
        "{task_line}\nTarget Path: {path_line}\nLanguage Hint: {language_line}\n{summary_line}\nTruncation Note: {truncation_line}\n{output_contract}\nDiff Excerpt:\n{}",
        req.diff_excerpt
    );

    PromptPayload {
        system_instruction: "You are a deterministic code-review assistant. Keep output concise, factual, and actionable.".to_string(),
        user_prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{AiConfig, AnalysisTask, AnalyzeDiffRequest};

    #[test]
    fn payload_contains_task_and_path_context() {
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::Summary,
            relative_path: Some("src/lib.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "-old\n+new".to_string(),
            summary: None,
            truncation_note: None,
            config: AiConfig::default(),
        };

        let payload = build_prompt_payload(&req);
        assert!(payload
            .user_prompt
            .contains("Task: summarize the code change clearly."));
        assert!(payload.user_prompt.contains("Target Path: src/lib.rs"));
        assert!(payload.user_prompt.contains("Language Hint: rust"));
        assert!(payload
            .user_prompt
            .contains("Return strict JSON object only"));
        assert!(payload.user_prompt.contains("Diff Excerpt:\n-old\n+new"));
    }

    #[test]
    fn payload_contains_summary_and_truncation_note_when_present() {
        let req = AnalyzeDiffRequest {
            task: AnalysisTask::RiskReview,
            relative_path: None,
            language_hint: None,
            diff_excerpt: "+unsafe { do_work(); }".to_string(),
            summary: Some(fc_core::TextDiffSummary {
                hunk_count: 1,
                added_lines: 5,
                removed_lines: 1,
                context_lines: 2,
            }),
            truncation_note: Some("truncated from 100 to 50".to_string()),
            config: AiConfig::default(),
        };

        let payload = build_prompt_payload(&req);
        assert!(payload
            .user_prompt
            .contains("Summary Stats: hunks=1, added=5, removed=1, context=2"));
        assert!(payload
            .user_prompt
            .contains("Truncation Note: truncated from 100 to 50"));
    }
}
