//! Path normalization placeholder.

use crate::domain::error::{CompareError, InvalidInputKind, PathSide};
use std::env;
use std::path::{Component, Path, PathBuf};

/// Normalizes a root path for compare pipeline.
pub(crate) fn normalize_root_path(path: &Path, side: PathSide) -> Result<PathBuf, CompareError> {
    if path.as_os_str().is_empty() {
        return Err(CompareError::InvalidRootPath {
            side,
            path: path.to_path_buf(),
        });
    }

    absolutize_path(path)
}

/// Normalizes a text diff file path for compare pipeline.
pub(crate) fn normalize_file_path(path: &Path, side: PathSide) -> Result<PathBuf, CompareError> {
    if path.as_os_str().is_empty() {
        return Err(CompareError::InvalidInput {
            kind: InvalidInputKind::EmptyFilePath { side },
        });
    }

    absolutize_path(path)
}

/// Converts an absolute child path into a stable compare key relative to root.
pub(crate) fn relative_path_key(root: &Path, absolute_path: &Path) -> Result<String, CompareError> {
    let relative =
        absolute_path
            .strip_prefix(root)
            .map_err(|err| CompareError::PathNormalizationFailed {
                path: absolute_path.to_path_buf(),
                reason: format!("failed to strip root prefix `{}`: {err}", root.display()),
            })?;

    if relative.as_os_str().is_empty() {
        return Err(CompareError::PathNormalizationFailed {
            path: absolute_path.to_path_buf(),
            reason: "root path itself cannot be used as a compare key".to_string(),
        });
    }

    let mut parts: Vec<String> = Vec::new();
    for component in relative.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => {
                parts.push(part.to_string_lossy().to_string());
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CompareError::PathNormalizationFailed {
                    path: absolute_path.to_path_buf(),
                    reason: "relative path contains unsupported components".to_string(),
                });
            }
        }
    }

    if parts.is_empty() {
        return Err(CompareError::PathNormalizationFailed {
            path: absolute_path.to_path_buf(),
            reason: "relative path resolved to empty key".to_string(),
        });
    }

    Ok(parts.join("/"))
}

fn absolutize_path(path: &Path) -> Result<PathBuf, CompareError> {
    let normalized = lexical_normalize(path);
    if normalized.is_absolute() {
        return Ok(normalized);
    }

    let cwd = env::current_dir().map_err(|err| CompareError::PathNormalizationFailed {
        path: path.to_path_buf(),
        reason: format!("failed to read current directory: {err}"),
    })?;

    Ok(lexical_normalize(&cwd.join(normalized)))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::RootDir
            | Component::Prefix(_)
            | Component::ParentDir
            | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn relative_path_key_uses_forward_slash() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let root = temp.path();
        let nested = root.join("a").join("b");
        fs::create_dir_all(&nested).expect("nested dir should be created");
        let file = nested.join("c.txt");
        fs::write(&file, "abc").expect("file should be written");

        let key = relative_path_key(root, &file).expect("key should be generated");
        assert_eq!(key, "a/b/c.txt");
    }

    #[test]
    fn relative_path_key_rejects_root_path() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let root = temp.path();
        let err = relative_path_key(root, root).expect_err("root must not be a compare key");
        assert!(matches!(err, CompareError::PathNormalizationFailed { .. }));
    }
}
