use crate::llm::types::ToolDefinition;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PluginManifest {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub main: Option<String>,
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    #[serde(default)]
    pub skills: Vec<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub prompt_injection: String,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: PathBuf,
    pub skills: Vec<SkillDefinition>,
    pub tools: Vec<ToolDefinition>,
    pub enabled: bool,
}
