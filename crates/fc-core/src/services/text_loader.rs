//! Text candidate detection and decoding helpers.

use crate::domain::error::{CompareError, TextPathUnavailableReason};
use crate::domain::options::TextDetectionStrategy;
use crate::infra::fs;
use std::path::Path;

const PROBE_BYTES: usize = 8 * 1024;

/// Loaded text document.
#[derive(Debug, Clone)]
pub(crate) struct LoadedText {
    /// Decoded content.
    pub content: String,
}

/// Text-load outcome for one file.
#[derive(Debug, Clone)]
pub(crate) enum TextLoadOutcome {
    /// File was decoded as text.
    Loaded(LoadedText),
    /// File is not considered a text candidate.
    NotTextCandidate,
    /// File looked like text but decoding failed.
    DecodeFailed,
}

/// Attempts to load a file as text according to detection strategy.
pub(crate) fn load_text_if_candidate(
    path: &Path,
    strategy: TextDetectionStrategy,
) -> Result<TextLoadOutcome, CompareError> {
    let bytes = fs::read_file(path)?;
    if !is_text_candidate(path, &bytes, strategy) {
        return Ok(TextLoadOutcome::NotTextCandidate);
    }

    match decode_text_bytes(&bytes) {
        Some(content) => Ok(TextLoadOutcome::Loaded(LoadedText { content })),
        None => Ok(TextLoadOutcome::DecodeFailed),
    }
}

/// Loads text for detailed diff API and returns structured boundary errors.
pub(crate) fn load_text_for_diff(
    path: &Path,
    strategy: TextDetectionStrategy,
) -> Result<LoadedText, CompareError> {
    match load_text_if_candidate(path, strategy)? {
        TextLoadOutcome::Loaded(doc) => Ok(doc),
        TextLoadOutcome::NotTextCandidate => Err(CompareError::TextPathUnavailable {
            path: path.to_path_buf(),
            reason: TextPathUnavailableReason::NotTextCandidate,
        }),
        TextLoadOutcome::DecodeFailed => Err(CompareError::TextPathUnavailable {
            path: path.to_path_buf(),
            reason: TextPathUnavailableReason::DecodeFailed,
        }),
    }
}

fn is_text_candidate(path: &Path, bytes: &[u8], strategy: TextDetectionStrategy) -> bool {
    if has_binary_extension(path) {
        return false;
    }

    let ext_hint = has_text_extension(path);
    let sample = &bytes[..bytes.len().min(PROBE_BYTES)];
    let content_hint = looks_like_text(sample);

    match strategy {
        TextDetectionStrategy::ExtensionHeuristic => ext_hint || content_hint,
        TextDetectionStrategy::ContentHeuristic => content_hint,
    }
}

fn has_binary_extension(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_ascii_lowercase());
    let Some(extension) = extension else {
        return false;
    };

    matches!(
        extension.as_str(),
        "bin"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "bmp"
            | "webp"
            | "pdf"
            | "zip"
            | "gz"
            | "tar"
            | "7z"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "class"
            | "jar"
            | "woff"
            | "woff2"
    )
}

fn has_text_extension(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_ascii_lowercase());
    let Some(extension) = extension else {
        return false;
    };

    matches!(
        extension.as_str(),
        "txt"
            | "md"
            | "rs"
            | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "xml"
            | "html"
            | "css"
            | "js"
            | "ts"
            | "py"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "sh"
            | "ini"
            | "conf"
            | "log"
    )
}

fn looks_like_text(sample: &[u8]) -> bool {
    if sample.is_empty() {
        return true;
    }
    if sample.starts_with(&[0xEF, 0xBB, 0xBF])
        || sample.starts_with(&[0xFF, 0xFE])
        || sample.starts_with(&[0xFE, 0xFF])
    {
        return true;
    }
    if sample.contains(&0) {
        return false;
    }

    let mut suspicious = 0usize;
    for byte in sample {
        let is_allowed_control = matches!(*byte, b'\n' | b'\r' | b'\t' | 0x0C);
        if *byte < 0x20 && !is_allowed_control {
            suspicious += 1;
        }
    }

    suspicious * 100 <= sample.len() * 5
}

fn decode_text_bytes(bytes: &[u8]) -> Option<String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(bytes[3..].to_vec()).ok();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return decode_utf16(&bytes[2..], true);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode_utf16(&bytes[2..], false);
    }

    String::from_utf8(bytes.to_vec()).ok()
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Option<String> {
    let chunks = bytes.chunks_exact(2);
    if !chunks.remainder().is_empty() {
        return None;
    }

    let mut units = Vec::with_capacity(bytes.len() / 2);
    for chunk in chunks {
        let unit = if little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        };
        units.push(unit);
    }

    let decoded: Result<String, _> = char::decode_utf16(units)
        .map(|r| r.map_err(|_| ()))
        .collect();
    decoded.ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn extension_hint_marks_text_candidate() {
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let path = dir.path().join("a.txt");
        fs::write(&path, [0xFF, 0x00, 0xAA]).expect("file should be written");

        let loaded = load_text_if_candidate(&path, TextDetectionStrategy::ExtensionHeuristic)
            .expect("loader should not fail");
        assert!(matches!(loaded, TextLoadOutcome::DecodeFailed));
    }

    #[test]
    fn binary_sample_is_not_text_candidate() {
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let path = dir.path().join("a.bin");
        fs::write(&path, [0x00, 0xFF, 0x13, 0x88]).expect("file should be written");

        let loaded = load_text_if_candidate(&path, TextDetectionStrategy::ContentHeuristic)
            .expect("loader should not fail");
        assert!(matches!(loaded, TextLoadOutcome::NotTextCandidate));
    }

    #[test]
    fn utf16le_bom_decodes() {
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let path = dir.path().join("a.txt");
        // BOM + \"Hi\" in UTF-16 LE
        fs::write(&path, [0xFF, 0xFE, b'H', 0x00, b'i', 0x00]).expect("file should be written");

        let loaded = load_text_if_candidate(&path, TextDetectionStrategy::ExtensionHeuristic)
            .expect("loader should not fail");
        let TextLoadOutcome::Loaded(doc) = loaded else {
            panic!("expected text to decode");
        };
        assert_eq!(doc.content, "Hi");
    }
}
