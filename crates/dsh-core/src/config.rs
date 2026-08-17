use dsh_common::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProviderType {
    DeepSeek,
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
            model_name: "deepseek-chat".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            reasoning_effort: "high".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub font_size: u32,
    pub show_diff_panel: bool,
    pub show_sidebar: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 14,
            show_diff_panel: true,
            show_sidebar: true,
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
