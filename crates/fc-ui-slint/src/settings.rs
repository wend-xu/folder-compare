//! Provider settings persistence for fc-ui-slint.

use anyhow::Context;
use fc_ai::AiProviderKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

const SETTINGS_FILE_NAME: &str = "provider_settings.toml";
const SETTINGS_VERSION: u32 = 1;

#[cfg(test)]
static TEST_SETTINGS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
#[cfg(test)]
static TEST_SETTINGS_DIR_OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

/// Persisted provider settings payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSettings {
    /// Selected provider mode.
    pub provider_kind: AiProviderKind,
    /// OpenAI-compatible endpoint text.
    pub openai_endpoint: String,
    /// OpenAI-compatible API key text.
    pub openai_api_key: String,
    /// OpenAI-compatible model text.
    pub openai_model: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for ProviderSettings {
    fn default() -> Self {
        Self {
            provider_kind: AiProviderKind::Mock,
            openai_endpoint: String::new(),
            openai_api_key: String::new(),
            openai_model: "gpt-4o-mini".to_string(),
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ProviderSettingsToml {
    version: u32,
    provider_kind: AiProviderKind,
    openai_endpoint: String,
    openai_api_key: String,
    openai_model: String,
    timeout_secs: u64,
}

impl From<&ProviderSettings> for ProviderSettingsToml {
    fn from(value: &ProviderSettings) -> Self {
        Self {
            version: SETTINGS_VERSION,
            provider_kind: value.provider_kind,
            openai_endpoint: value.openai_endpoint.clone(),
            openai_api_key: value.openai_api_key.clone(),
            openai_model: value.openai_model.clone(),
            timeout_secs: value.timeout_secs.max(1),
        }
    }
}

impl From<ProviderSettingsToml> for ProviderSettings {
    fn from(value: ProviderSettingsToml) -> Self {
        Self {
            provider_kind: value.provider_kind,
            openai_endpoint: value.openai_endpoint,
            openai_api_key: value.openai_api_key,
            openai_model: if value.openai_model.trim().is_empty() {
                "gpt-4o-mini".to_string()
            } else {
                value.openai_model
            },
            timeout_secs: value.timeout_secs.max(1),
        }
    }
}

/// Returns the provider settings file path.
pub fn provider_settings_file_path() -> PathBuf {
    provider_settings_dir().join(SETTINGS_FILE_NAME)
}

/// Loads provider settings from disk.
pub fn load_provider_settings() -> anyhow::Result<Option<ProviderSettings>> {
    let path = provider_settings_file_path();
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read provider settings from {}", path.display()))?;
    let parsed: ProviderSettingsToml = toml::from_str(&raw)
        .with_context(|| format!("parse provider settings from {}", path.display()))?;
    Ok(Some(parsed.into()))
}

/// Saves provider settings to disk and returns the written path.
pub fn save_provider_settings(settings: &ProviderSettings) -> anyhow::Result<PathBuf> {
    let path = provider_settings_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create provider settings dir {}", parent.display()))?;
    }

    let payload: ProviderSettingsToml = settings.into();
    let serialized =
        toml::to_string_pretty(&payload).context("serialize provider settings to toml")?;
    std::fs::write(&path, serialized)
        .with_context(|| format!("write provider settings to {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
pub(crate) struct TestSettingsDirGuard {
    _lock: MutexGuard<'static, ()>,
}

#[cfg(test)]
impl TestSettingsDirGuard {
    pub(crate) fn new(path: &std::path::Path) -> Self {
        let lock = TEST_SETTINGS_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("test settings lock should not be poisoned");
        *TEST_SETTINGS_DIR_OVERRIDE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .expect("test settings dir override should not be poisoned") = Some(path.to_path_buf());
        Self { _lock: lock }
    }
}

#[cfg(test)]
impl Drop for TestSettingsDirGuard {
    fn drop(&mut self) {
        *TEST_SETTINGS_DIR_OVERRIDE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .expect("test settings dir override should not be poisoned") = None;
    }
}

fn provider_settings_dir() -> PathBuf {
    #[cfg(test)]
    if let Some(dir) = TEST_SETTINGS_DIR_OVERRIDE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("test settings dir override should not be poisoned")
        .clone()
    {
        return dir;
    }

    if let Some(dir) = std::env::var_os("FOLDER_COMPARE_CONFIG_DIR") {
        return PathBuf::from(dir);
    }

    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("folder-compare");
        }
    }

    if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("folder-compare");
        }
    }

    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("folder-compare");
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("folder-compare");
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".folder-compare")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_none_when_file_does_not_exist() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let _settings_guard = TestSettingsDirGuard::new(dir.path());

        let loaded = load_provider_settings().expect("loading should succeed");
        assert!(loaded.is_none());
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let _settings_guard = TestSettingsDirGuard::new(dir.path());

        let settings = ProviderSettings {
            provider_kind: AiProviderKind::OpenAiCompatible,
            openai_endpoint: "https://api.example.com/v1".to_string(),
            openai_api_key: "sk-test".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            timeout_secs: 45,
        };
        let path = save_provider_settings(&settings).expect("save should succeed");
        assert!(path.exists());

        let loaded = load_provider_settings()
            .expect("load should succeed")
            .expect("settings should exist");
        assert_eq!(loaded, settings);
    }
}
