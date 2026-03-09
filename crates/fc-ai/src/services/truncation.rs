//! Diff truncation placeholder.

/// Truncates text to a maximum length.
pub(crate) fn truncate_text(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}
