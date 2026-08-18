use dsh_common::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum McpTransport {
    Stdio,
    Sse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub description: String,
    pub transport: McpTransport,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
}

pub struct McpRegistry;

impl McpRegistry {
    pub fn get_default_presets() -> Vec<McpServerConfig> {
        vec![
            McpServerConfig {
                name: "filesystem".to_string(),
                description: "Local filesystem access for reading, writing, and listing files"
                    .to_string(),
                transport: McpTransport::Stdio,
                command: Some("npx".to_string()),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    ".".to_string(),
                ],
                url: None,
                env: HashMap::new(),
                enabled: true,
            },
            McpServerConfig {
                name: "git".to_string(),
                description: "Inspect, commit, and diff Git repositories".to_string(),
                transport: McpTransport::Stdio,
                command: Some("uvx".to_string()),
                args: vec!["mcp-server-git".to_string()],
                url: None,
                env: HashMap::new(),
                enabled: true,
            },
            McpServerConfig {
                name: "fetch".to_string(),
                description: "Fetch web page content and convert HTML to markdown".to_string(),
                transport: McpTransport::Stdio,
                command: Some("uvx".to_string()),
                args: vec!["mcp-server-fetch".to_string()],
                url: None,
                env: HashMap::new(),
                enabled: true,
            },
            McpServerConfig {
                name: "github".to_string(),
                description: "GitHub API integration for PRs, issues, and repos".to_string(),
                transport: McpTransport::Stdio,
                command: Some("npx".to_string()),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-github".to_string(),
                ],
                url: None,
                env: HashMap::new(),
                enabled: false,
            },
        ]
    }

    pub fn load_servers(storage_dir: &Path) -> Vec<McpServerConfig> {
        let file = storage_dir.join("mcp_servers.json");
        if file.exists() {
            if let Ok(content) = fs::read_to_string(&file) {
                if let Ok(servers) = serde_json::from_str::<Vec<McpServerConfig>>(&content) {
                    return servers;
                }
            }
        }
        Self::get_default_presets()
    }

    pub fn save_servers(storage_dir: &Path, servers: &[McpServerConfig]) -> Result<()> {
        if !storage_dir.exists() {
            fs::create_dir_all(storage_dir)?;
        }
        let file = storage_dir.join("mcp_servers.json");
        let json = serde_json::to_string_pretty(servers)?;
        fs::write(&file, json)?;
        info!("Saved MCP server registry to {}", file.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_mcp_presets_and_persistence() {
        let temp_dir = env::temp_dir().join(format!("dsh_mcp_{}", uuid::Uuid::new_v4()));
        let presets = McpRegistry::get_default_presets();
        assert!(!presets.is_empty());

        McpRegistry::save_servers(&temp_dir, &presets).unwrap();

        let loaded = McpRegistry::load_servers(&temp_dir);
        assert_eq!(loaded.len(), presets.len());
        assert_eq!(loaded[0].name, "filesystem");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
