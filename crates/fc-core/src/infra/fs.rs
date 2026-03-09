//! File-system helpers for scanner and compare pipeline.

use crate::domain::error::{CompareError, IoOperation};
use std::fs::{self, File, Metadata, ReadDir};
use std::io::Read;
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

/// Opens a file for content comparison and maps I/O errors.
pub(crate) fn open_file(path: &Path) -> Result<File, CompareError> {
    File::open(path).map_err(|source| CompareError::IoBoundary {
        operation: IoOperation::ReadFile,
        path: path.to_path_buf(),
        source,
    })
}

/// Maps a file-read error to `CompareError`.
pub(crate) fn map_read_file_error(path: PathBuf, source: std::io::Error) -> CompareError {
    CompareError::IoBoundary {
        operation: IoOperation::ReadFile,
        path,
        source,
    }
}

/// Reads the full file into memory and maps I/O errors.
pub(crate) fn read_file(path: &Path) -> Result<Vec<u8>, CompareError> {
    let mut file = open_file(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|source| map_read_file_error(path.to_path_buf(), source))?;
    Ok(buffer)
}
