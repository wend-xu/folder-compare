//! Directory scanning service.

use crate::domain::entry::EntryKind;
use crate::domain::error::{CompareError, PathSide};
use crate::infra::fs;
use crate::infra::path_norm;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One scanned file-system entry.
#[derive(Debug, Clone)]
pub(crate) struct ScannedEntry {
    /// Absolute path from file system traversal.
    pub absolute_path: PathBuf,
    /// Stable relative path key used for alignment.
    pub relative_path: String,
    /// Entry kind.
    pub kind: EntryKind,
    /// Optional entry size in bytes.
    pub size_bytes: Option<u64>,
}

/// Scan result for one compare root.
#[derive(Debug, Clone)]
pub(crate) struct ScannedTree {
    /// Side that produced this tree.
    pub side: PathSide,
    /// Normalized absolute root path.
    pub root: PathBuf,
    /// Indexed entries by relative path.
    pub entries: BTreeMap<String, ScannedEntry>,
}

/// Recursively scans one compare root.
pub(crate) fn scan_tree(
    root: &Path,
    side: PathSide,
    follow_symlinks: bool,
) -> Result<ScannedTree, CompareError> {
    if !root.exists() {
        return Err(CompareError::RootPathNotFound {
            side,
            path: root.to_path_buf(),
        });
    }

    let root_metadata = fs::metadata(root)?;
    if !root_metadata.is_dir() {
        return Err(CompareError::RootPathNotDirectory {
            side,
            path: root.to_path_buf(),
        });
    }

    let mut entries: BTreeMap<String, ScannedEntry> = BTreeMap::new();
    walk_dir(root, root, follow_symlinks, &mut entries)?;

    Ok(ScannedTree {
        side,
        root: root.to_path_buf(),
        entries,
    })
}

fn walk_dir(
    root: &Path,
    current: &Path,
    follow_symlinks: bool,
    output: &mut BTreeMap<String, ScannedEntry>,
) -> Result<(), CompareError> {
    let dir_iter = fs::read_dir(current)?;
    for entry_result in dir_iter {
        let entry = entry_result
            .map_err(|source| fs::map_read_dir_entry_error(current.to_path_buf(), source))?;
        let absolute_path = entry.path();

        let metadata = if follow_symlinks {
            fs::metadata(&absolute_path)?
        } else {
            fs::symlink_metadata(&absolute_path)?
        };
        let file_type = metadata.file_type();
        let kind = entry_kind_from_file_type(file_type);

        let relative_path = path_norm::relative_path_key(root, &absolute_path)?;
        let size_bytes = if matches!(kind, EntryKind::File) {
            Some(metadata.len())
        } else {
            None
        };

        let scanned = ScannedEntry {
            absolute_path: absolute_path.clone(),
            relative_path: relative_path.clone(),
            kind,
            size_bytes,
        };

        if output.insert(relative_path.clone(), scanned).is_some() {
            return Err(CompareError::PathNormalizationFailed {
                path: absolute_path,
                reason: format!("duplicate relative path key generated: `{relative_path}`"),
            });
        }

        if file_type.is_dir() {
            walk_dir(root, &absolute_path, follow_symlinks, output)?;
        }
    }

    Ok(())
}

fn entry_kind_from_file_type(file_type: std::fs::FileType) -> EntryKind {
    if file_type.is_file() {
        EntryKind::File
    } else if file_type.is_dir() {
        EntryKind::Directory
    } else if file_type.is_symlink() {
        EntryKind::Symlink
    } else {
        EntryKind::Other
    }
}
