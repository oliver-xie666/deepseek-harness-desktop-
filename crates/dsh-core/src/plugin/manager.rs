use super::loader::PluginLoader;
use super::types::PluginInfo;
use crate::tools::ToolRegistry;
use std::path::Path;

#[derive(Clone, Default)]
pub struct PluginManager {
    plugins: Vec<PluginInfo>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self, workspace: &Path) {
        let discovered = PluginLoader::discover_plugins(workspace);
        self.plugins = discovered;
    }

    pub fn add_plugin(&mut self, plugin: PluginInfo) {
        self.plugins.retain(|p| p.id != plugin.id);
        self.plugins.push(plugin);
    }

    pub fn list_plugins(&self) -> &[PluginInfo] {
        &self.plugins
    }

    pub fn set_plugin_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(p) = self.plugins.iter_mut().find(|p| p.id == id) {
            p.enabled = enabled;
        }
    }

    pub fn build_skills_prompt(&self) -> String {
        let mut prompt = String::new();
        for plugin in &self.plugins {
            if !plugin.enabled {
                continue;
            }
            for skill in &plugin.skills {
                prompt.push_str(&format!("### Skill: {}\n", skill.name));
                prompt.push_str(&format!("Description: {}\n", skill.description));
                prompt.push_str(&skill.prompt_injection);
                prompt.push_str("\n\n");
            }
        }
        prompt
    }

    pub fn register_tools_to(&self, registry: &mut ToolRegistry) {
        for plugin in &self.plugins {
            if !plugin.enabled {
                continue;
            }
            for tool in &plugin.tools {
                registry.register_custom_tool(tool.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::types::SkillDefinition;
    use std::path::PathBuf;

    #[test]
    fn test_plugin_manager_skills_prompt() {
        let mut manager = PluginManager::new();
        let plugin = PluginInfo {
            id: "plugin-test".into(),
            name: "test-plugin".into(),
            version: "1.0.0".into(),
            description: "Test description".into(),
            path: PathBuf::from("/test"),
            skills: vec![SkillDefinition {
                name: "test-skill".into(),
                description: "Skill for testing".into(),
                prompt_injection: "Follow test rules".into(),
                source_path: PathBuf::from("/test/SKILL.md"),
            }],
            tools: vec![],
            enabled: true,
        };

        manager.add_plugin(plugin);
        let prompt = manager.build_skills_prompt();
        assert!(prompt.contains("Skill: test-skill"));
        assert!(prompt.contains("Follow test rules"));

        manager.set_plugin_enabled("plugin-test", false);
        let prompt2 = manager.build_skills_prompt();
        assert!(prompt2.is_empty());
    }
}
