//! Deterministic file-level comparison service.

use crate::domain::error::CompareError;
use crate::infra::fs;
use std::io::Read;
use std::path::Path;

const BUFFER_SIZE: usize = 8 * 1024;

/// Result of file-level comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileComparisonResult {
    /// Left file size in bytes.
    pub left_size: u64,
    /// Right file size in bytes.
    pub right_size: u64,
    /// Whether file contents are equal.
    pub is_equal: bool,
    /// Whether byte-level content comparison ran.
    pub content_checked: bool,
}

/// Compares two files with `size + bytes` strategy.
pub(crate) fn compare_files(
    left_path: &Path,
    right_path: &Path,
) -> Result<FileComparisonResult, CompareError> {
    let left_meta = fs::metadata(left_path)?;
    let right_meta = fs::metadata(right_path)?;
    let left_size = left_meta.len();
    let right_size = right_meta.len();

    if left_size != right_size {
        return Ok(FileComparisonResult {
            left_size,
            right_size,
            is_equal: false,
            content_checked: false,
        });
    }

    let mut left_file = fs::open_file(left_path)?;
    let mut right_file = fs::open_file(right_path)?;
    let mut left_buf = [0_u8; BUFFER_SIZE];
    let mut right_buf = [0_u8; BUFFER_SIZE];

    loop {
        let left_n = left_file
            .read(&mut left_buf)
            .map_err(|source| fs::map_read_file_error(left_path.to_path_buf(), source))?;
        let right_n = right_file
            .read(&mut right_buf)
            .map_err(|source| fs::map_read_file_error(right_path.to_path_buf(), source))?;

        if left_n != right_n {
            return Ok(FileComparisonResult {
                left_size,
                right_size,
                is_equal: false,
                content_checked: true,
            });
        }

        if left_n == 0 {
            break;
        }

        if left_buf[..left_n] != right_buf[..right_n] {
            return Ok(FileComparisonResult {
                left_size,
                right_size,
                is_equal: false,
                content_checked: true,
            });
        }
    }

    Ok(FileComparisonResult {
        left_size,
        right_size,
        is_equal: true,
        content_checked: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn compare_files_equal_content() {
        let left_dir = tempfile::tempdir().expect("left tempdir should be created");
        let right_dir = tempfile::tempdir().expect("right tempdir should be created");
        let left = left_dir.path().join("a.bin");
        let right = right_dir.path().join("a.bin");
        fs::write(&left, b"abcdef").expect("left file should be written");
        fs::write(&right, b"abcdef").expect("right file should be written");

        let result = compare_files(&left, &right).expect("compare should succeed");
        assert!(result.is_equal);
        assert!(result.content_checked);
    }

    #[test]
    fn compare_files_different_size() {
        let left_dir = tempfile::tempdir().expect("left tempdir should be created");
        let right_dir = tempfile::tempdir().expect("right tempdir should be created");
        let left = left_dir.path().join("a.bin");
        let right = right_dir.path().join("a.bin");
        fs::write(&left, b"abc").expect("left file should be written");
        fs::write(&right, b"abcdef").expect("right file should be written");

        let result = compare_files(&left, &right).expect("compare should succeed");
        assert!(!result.is_equal);
        assert!(!result.content_checked);
    }

    #[test]
    fn compare_files_missing_path_returns_io_boundary() {
        let left_dir = tempfile::tempdir().expect("left tempdir should be created");
        let left = left_dir.path().join("a.bin");
        let right = left_dir.path().join("missing.bin");
        fs::write(&left, b"abc").expect("left file should be written");

        let err = compare_files(&left, &right).expect_err("missing file should fail");
        assert!(matches!(err, CompareError::IoBoundary { .. }));
    }
}
