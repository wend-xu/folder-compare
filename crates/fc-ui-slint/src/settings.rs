//! App settings persistence for fc-ui-slint.

use anyhow::Context;
use fc_ai::AiProviderKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

const SETTINGS_FILE_NAME: &str = "settings.toml";
const LEGACY_PROVIDER_SETTINGS_FILE_NAME: &str = "provider_settings.toml";
const SETTINGS_VERSION: u32 = 2;

#[cfg(test)]
static TEST_SETTINGS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
#[cfg(test)]
static TEST_SETTINGS_DIR_OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

/// Persisted provider configuration payload.
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

/// Persisted behavior preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorSettings {
    /// Whether dot-prefixed files/folders remain visible in Results / Navigator.
    pub show_hidden_files: bool,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            show_hidden_files: true,
        }
    }
}

/// Persisted application settings payload.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppPreferences {
    /// Provider configuration for AI analysis.
    pub provider: ProviderSettings,
    /// Non-provider UI behavior preferences.
    pub behavior: BehaviorSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ProviderSettingsToml {
    provider_kind: AiProviderKind,
    openai_endpoint: String,
    openai_api_key: String,
    openai_model: String,
    timeout_secs: u64,
}

impl Default for ProviderSettingsToml {
    fn default() -> Self {
        Self::from(&ProviderSettings::default())
    }
}

impl From<&ProviderSettings> for ProviderSettingsToml {
    fn from(value: &ProviderSettings) -> Self {
        Self {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BehaviorSettingsToml {
    show_hidden_files: bool,
}

impl Default for BehaviorSettingsToml {
    fn default() -> Self {
        Self::from(&BehaviorSettings::default())
    }
}

impl From<&BehaviorSettings> for BehaviorSettingsToml {
    fn from(value: &BehaviorSettings) -> Self {
        Self {
            show_hidden_files: value.show_hidden_files,
        }
    }
}

impl From<BehaviorSettingsToml> for BehaviorSettings {
    fn from(value: BehaviorSettingsToml) -> Self {
        Self {
            show_hidden_files: value.show_hidden_files,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct AppPreferencesToml {
    version: u32,
    #[serde(default)]
    provider: ProviderSettingsToml,
    #[serde(default)]
    behavior: BehaviorSettingsToml,
}

impl From<&AppPreferences> for AppPreferencesToml {
    fn from(value: &AppPreferences) -> Self {
        Self {
            version: SETTINGS_VERSION,
            provider: ProviderSettingsToml::from(&value.provider),
            behavior: BehaviorSettingsToml::from(&value.behavior),
        }
    }
}

impl From<AppPreferencesToml> for AppPreferences {
    fn from(value: AppPreferencesToml) -> Self {
        Self {
            provider: value.provider.into(),
            behavior: value.behavior.into(),
        }
    }
}

/// Returns the current settings file path.
pub fn settings_file_path() -> PathBuf {
    settings_dir().join(SETTINGS_FILE_NAME)
}

/// Loads application settings from disk, falling back to the legacy provider-only file.
pub fn load_app_preferences() -> anyhow::Result<Option<AppPreferences>> {
    let path = settings_file_path();
    if path.exists() {
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("read settings from {}", path.display()))?;
        let parsed: AppPreferencesToml = toml::from_str(&raw)
            .with_context(|| format!("parse settings from {}", path.display()))?;
        return Ok(Some(parsed.into()));
    }

    let legacy_path = legacy_provider_settings_file_path();
    if !legacy_path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&legacy_path).with_context(|| {
        format!(
            "read legacy provider settings from {}",
            legacy_path.display()
        )
    })?;
    let parsed: ProviderSettingsToml = toml::from_str(&raw).with_context(|| {
        format!(
            "parse legacy provider settings from {}",
            legacy_path.display()
        )
    })?;
    Ok(Some(AppPreferences {
        provider: parsed.into(),
        behavior: BehaviorSettings::default(),
    }))
}

/// Saves application settings to disk and returns the written path.
pub fn save_app_preferences(settings: &AppPreferences) -> anyhow::Result<PathBuf> {
    let path = settings_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create settings dir {}", parent.display()))?;
    }

    let payload: AppPreferencesToml = settings.into();
    let serialized = toml::to_string_pretty(&payload).context("serialize settings to toml")?;
    std::fs::write(&path, serialized)
        .with_context(|| format!("write settings to {}", path.display()))?;
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

fn settings_dir() -> PathBuf {
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

fn legacy_provider_settings_file_path() -> PathBuf {
    settings_dir().join(LEGACY_PROVIDER_SETTINGS_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_none_when_no_settings_files_exist() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let _settings_guard = TestSettingsDirGuard::new(dir.path());

        let loaded = load_app_preferences().expect("loading should succeed");
        assert!(loaded.is_none());
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let _settings_guard = TestSettingsDirGuard::new(dir.path());

        let settings = AppPreferences {
            provider: ProviderSettings {
                provider_kind: AiProviderKind::OpenAiCompatible,
                openai_endpoint: "https://api.example.com/v1".to_string(),
                openai_api_key: "sk-test".to_string(),
                openai_model: "gpt-4o-mini".to_string(),
                timeout_secs: 45,
            },
            behavior: BehaviorSettings {
                show_hidden_files: false,
            },
        };
        let path = save_app_preferences(&settings).expect("save should succeed");
        assert!(path.exists());

        let loaded = load_app_preferences()
            .expect("load should succeed")
            .expect("settings should exist");
        assert_eq!(loaded, settings);
    }

    #[test]
    fn load_falls_back_to_legacy_provider_settings_file() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let _settings_guard = TestSettingsDirGuard::new(dir.path());

        let legacy_settings = ProviderSettings {
            provider_kind: AiProviderKind::OpenAiCompatible,
            openai_endpoint: "https://api.example.com/v1".to_string(),
            openai_api_key: "sk-legacy".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            timeout_secs: 55,
        };
        let legacy_payload = toml::to_string_pretty(&ProviderSettingsToml::from(&legacy_settings))
            .expect("legacy payload should serialize");
        std::fs::write(legacy_provider_settings_file_path(), legacy_payload)
            .expect("legacy settings should be written");

        let loaded = load_app_preferences()
            .expect("load should succeed")
            .expect("settings should exist");
        assert_eq!(loaded.provider, legacy_settings);
        assert_eq!(loaded.behavior, BehaviorSettings::default());
    }
}
