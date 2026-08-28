use dsh_common::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    DeepSeek,
    OpenAI,
    Anthropic,
    MiniMax,
    Moonshot,
    Qwen,
    Ollama,
    VLLM,
    OpenRouter,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: ProviderType,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub reasoning_effort: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: ProviderType::DeepSeek,
            api_key: String::new(),
            base_url: "https://api.deepseek.com".to_string(),
            model_name: "deepseek-reasoner".to_string(),
            temperature: 0.6,
            max_tokens: 8192,
            reasoning_effort: "high".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CustomPresetConfig {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme: String,
    pub font_size: u32,
    pub show_diff_panel: bool,
    pub show_sidebar: bool,
    pub language: String,
    pub permission_mode: String,
    pub agent_preset: String,
    pub enter_behavior: String,
    pub sidebar_default_open: bool,
    pub sidebar_width_percent: u32,
    pub open_files_in_sidebar: bool,
    pub sidebar_position_compat: bool,
    pub auto_open_jobs: bool,
    pub show_workspace_tree: bool,
    pub show_terminal_logs: bool,
    pub disabled_plugins: Vec<String>,
    pub custom_presets: Vec<CustomPresetConfig>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            font_size: 14,
            show_diff_panel: true,
            show_sidebar: true,
            language: "zh-CN".to_string(),
            permission_mode: "full-access".to_string(),
            agent_preset: "standard".to_string(),
            enter_behavior: "queue".to_string(),
            sidebar_default_open: true,
            sidebar_width_percent: 30,
            open_files_in_sidebar: true,
            sidebar_position_compat: false,
            auto_open_jobs: true,
            show_workspace_tree: true,
            show_terminal_logs: true,
            disabled_plugins: Vec::new(),
            custom_presets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub model: ModelConfig,
    pub ui: UiConfig,
}

impl AppConfig {
    pub fn load_or_default(storage_dir: &Path) -> Self {
        let config_file = storage_dir.join("config.json");
        if config_file.exists() {
            if let Ok(content) = fs::read_to_string(&config_file) {
                if let Ok(config) = serde_json::from_str::<Self>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, storage_dir: &Path) -> Result<()> {
        if !storage_dir.exists() {
            fs::create_dir_all(storage_dir)?;
        }
        let config_file = storage_dir.join("config.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&config_file, json)?;
        info!("Saved configuration to {}", config_file.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = env::temp_dir().join(format!("dsh_cfg_{}", uuid::Uuid::new_v4()));
        let mut config = AppConfig::default();
        config.model.model_name = "deepseek-reasoner".to_string();
        config.model.api_key = "sk-test-key".to_string();

        config.save(&temp_dir).unwrap();

        let loaded = AppConfig::load_or_default(&temp_dir);
        assert_eq!(loaded.model.model_name, "deepseek-reasoner");
        assert_eq!(loaded.model.api_key, "sk-test-key");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
