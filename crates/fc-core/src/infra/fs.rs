//! File-system helpers for scanner and compare pipeline.

use crate::domain::error::{CompareError, IoOperation};
use std::fs::{self, Metadata, ReadDir};
use std::path::{Path, PathBuf};

/// Reads metadata and maps I/O errors to `CompareError`.
pub(crate) fn metadata(path: &Path) -> Result<Metadata, CompareError> {
    fs::metadata(path).map_err(|source| CompareError::IoBoundary {
        operation: IoOperation::Stat,
        path: path.to_path_buf(),
        source,
    })
}

/// Reads symlink metadata and maps I/O errors to `CompareError`.
pub(crate) fn symlink_metadata(path: &Path) -> Result<Metadata, CompareError> {
    fs::symlink_metadata(path).map_err(|source| CompareError::IoBoundary {
        operation: IoOperation::Stat,
        path: path.to_path_buf(),
        source,
    })
}

/// Lists directory entries and maps I/O errors to `CompareError`.
pub(crate) fn read_dir(path: &Path) -> Result<ReadDir, CompareError> {
    fs::read_dir(path).map_err(|source| CompareError::IoBoundary {
        operation: IoOperation::ReadDir,
        path: path.to_path_buf(),
        source,
    })
}

/// Maps a read-dir entry iteration error to `CompareError`.
pub(crate) fn map_read_dir_entry_error(path: PathBuf, source: std::io::Error) -> CompareError {
    CompareError::IoBoundary {
        operation: IoOperation::ReadDir,
        path,
        source,
    }
}
