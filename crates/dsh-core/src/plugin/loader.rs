use super::skill_parser::parse_skill_file;
use super::types::{PluginInfo, PluginManifest};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct PluginLoader;

impl PluginLoader {
    /// Discover all plugins in candidate search paths
    pub fn discover_plugins(workspace: &Path) -> Vec<PluginInfo> {
        let mut search_paths = Vec::new();

        // 1. Workspace paths
        search_paths.push(workspace.join(".dsh").join("plugins"));
        search_paths.push(workspace.join(".agents").join("skills"));
        search_paths.push(workspace.join(".codex").join("skills"));
        search_paths.push(workspace.join("skills"));

        // 2. User home paths
        if let Some(home) = directories::UserDirs::new().map(|u| u.home_dir().to_path_buf()) {
            search_paths.push(home.join(".dsh").join("plugins"));
            search_paths.push(home.join(".agents").join("skills"));
            search_paths.push(home.join(".codex").join("skills"));
        }

        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            search_paths.push(PathBuf::from(appdata).join("deepseek").join("plugins"));
        }

        let mut plugins = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for base_dir in search_paths {
            if !base_dir.exists() || !base_dir.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&base_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(plugin) = Self::load_single_plugin_dir(&path) {
                            if seen_ids.insert(plugin.id.clone()) {
                                info!("Discovered plugin: {} ({})", plugin.name, plugin.id);
                                plugins.push(plugin);
                            }
                        }
                    }
                }
            }
        }

        plugins
    }

    pub fn load_single_plugin_dir(dir: &Path) -> Option<PluginInfo> {
        let dir_name = dir.file_name()?.to_str()?.to_string();

        // 1. Look for plugin.json
        let plugin_json_path = dir.join("plugin.json");
        let manifest: Option<PluginManifest> = if plugin_json_path.exists() {
            fs::read_to_string(&plugin_json_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
        } else {
            None
        };

        // 2. Look for SKILL.md
        let skill_md_path = dir.join("SKILL.md");
        let mut skills = Vec::new();
        if skill_md_path.exists() {
            if let Ok(skill) = parse_skill_file(&skill_md_path) {
                skills.push(skill);
            }
        }

        // Also scan subdirectories for nested SKILL.md
        if let Ok(sub_entries) = fs::read_dir(dir) {
            for sub in sub_entries.flatten() {
                let sub_path = sub.path();
                if sub_path.is_dir() {
                    let nested_skill = sub_path.join("SKILL.md");
                    if nested_skill.exists() {
                        if let Ok(skill) = parse_skill_file(&nested_skill) {
                            skills.push(skill);
                        }
                    }
                }
            }
        }

        if manifest.is_none() && skills.is_empty() {
            return None;
        }

        let (name, version, description, tools) = match manifest {
            Some(m) => (m.name, m.version, m.description, m.tools),
            None => {
                let skill_desc = skills
                    .first()
                    .map(|s| s.description.clone())
                    .unwrap_or_else(|| "Community skill extension".to_string());
                (
                    dir_name.clone(),
                    "1.0.0".to_string(),
                    skill_desc,
                    Vec::new(),
                )
            }
        };

        Some(PluginInfo {
            id: format!("plugin-{}", dir_name),
            name,
            version,
            description,
            path: dir.to_path_buf(),
            skills,
            tools,
            enabled: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_plugin_dir_with_skill() {
        let temp_dir =
            std::env::temp_dir().join(format!("dsh_test_plugin_{}", uuid::Uuid::new_v4()));
        let plugin_dir = temp_dir.join("archify");
        let _ = fs::create_dir_all(&plugin_dir);

        let skill_content = r#"---
name: archify
description: Visual diagram generator
---
# Instructions
Generate architecture diagrams in SVG format.
"#;
        fs::write(plugin_dir.join("SKILL.md"), skill_content).unwrap();

        let plugin = PluginLoader::load_single_plugin_dir(&plugin_dir).unwrap();
        assert_eq!(plugin.name, "archify");
        assert_eq!(plugin.skills.len(), 1);
        assert_eq!(plugin.skills[0].name, "archify");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
