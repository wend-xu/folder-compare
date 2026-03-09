//! Diff input preparation and truncation helpers.

/// Prepared diff excerpt used by analyzer orchestration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedDiffInput {
    /// Final excerpt passed into prompt generation.
    pub prepared_excerpt: String,
    /// Whether the input was truncated.
    pub was_truncated: bool,
    /// Optional truncation note for upstream response metadata.
    pub truncation_note: Option<String>,
}

/// Prepares diff excerpt under a character budget.
pub(crate) fn prepare_diff_excerpt(input: &str, max_chars: usize) -> PreparedDiffInput {
    let input_chars = input.chars().count();
    if input_chars <= max_chars {
        return PreparedDiffInput {
            prepared_excerpt: input.to_string(),
            was_truncated: false,
            truncation_note: None,
        };
    }

    let prepared_excerpt: String = input.chars().take(max_chars).collect();
    PreparedDiffInput {
        prepared_excerpt,
        was_truncated: true,
        truncation_note: Some(format!(
            "diff excerpt truncated from {} chars to {} chars",
            input_chars, max_chars
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_diff_excerpt_no_truncation() {
        let prepared = prepare_diff_excerpt("abc", 8);
        assert_eq!(prepared.prepared_excerpt, "abc");
        assert!(!prepared.was_truncated);
        assert!(prepared.truncation_note.is_none());
    }

    #[test]
    fn prepare_diff_excerpt_with_truncation() {
        let prepared = prepare_diff_excerpt("abcdef", 3);
        assert_eq!(prepared.prepared_excerpt, "abc");
        assert!(prepared.was_truncated);
        assert_eq!(
            prepared.truncation_note.as_deref(),
            Some("diff excerpt truncated from 6 chars to 3 chars")
        );
    }
}
