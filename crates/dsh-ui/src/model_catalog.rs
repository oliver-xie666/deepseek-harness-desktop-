#![allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalogItem {
    pub provider: &'static str,
    pub name: &'static str,
    pub is_custom: bool,
}

pub const KNOWN_MODELS: &[ModelCatalogItem] = &[
    ModelCatalogItem {
        provider: "DeepSeek",
        name: "DeepSeek-V4-Flash",
        is_custom: false,
    },
    ModelCatalogItem {
        provider: "DeepSeek",
        name: "DeepSeek-V4-Pro",
        is_custom: false,
    },
    ModelCatalogItem {
        provider: "DeepSeek",
        name: "deepseek-chat",
        is_custom: false,
    },
    ModelCatalogItem {
        provider: "DeepSeek",
        name: "deepseek-reasoner",
        is_custom: false,
    },
    ModelCatalogItem {
        provider: "DeepSeek (modlens vision)",
        name: "DeepSeek-V4-Flash (modlens vision)",
        is_custom: false,
    },
    ModelCatalogItem {
        provider: "DeepSeek (modlens vision)",
        name: "DeepSeek-V4-Pro (modlens vision)",
        is_custom: false,
    },
    ModelCatalogItem {
        provider: "bytecat",
        name: "gpt-5.6-luna",
        is_custom: true,
    },
    ModelCatalogItem {
        provider: "bytecat",
        name: "codex-auto-review",
        is_custom: true,
    },
    ModelCatalogItem {
        provider: "bytecat",
        name: "gpt-5.5",
        is_custom: true,
    },
];

pub fn model_options(current: &str) -> Vec<String> {
    let mut models: Vec<String> = KNOWN_MODELS.iter().map(|m| m.name.to_string()).collect();
    if !current.is_empty() && !models.iter().any(|model| model == current) {
        models.insert(0, current.to_string());
    }
    models
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderGroup {
    pub provider: String,
    pub models: Vec<String>,
}

pub fn provider_groups(current: &str) -> Vec<ProviderGroup> {
    let mut groups: Vec<ProviderGroup> = Vec::new();
    for item in KNOWN_MODELS {
        if let Some(group) = groups.iter_mut().find(|g| g.provider == item.provider) {
            if !group.models.contains(&item.name.to_string()) {
                group.models.push(item.name.to_string());
            }
        } else {
            groups.push(ProviderGroup {
                provider: item.provider.to_string(),
                models: vec![item.name.to_string()],
            });
        }
    }
    if !current.is_empty() {
        let exists = groups.iter().any(|g| g.models.iter().any(|m| m == current));
        if !exists {
            if let Some(custom_group) = groups
                .iter_mut()
                .find(|g| g.provider == "bytecat" || g.provider == "自定义")
            {
                custom_group.models.insert(0, current.to_string());
            } else {
                groups.push(ProviderGroup {
                    provider: "自定义".to_string(),
                    models: vec![current.to_string()],
                });
            }
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unknown_configured_model() {
        let models = model_options("custom-model");
        assert_eq!(models.first().map(String::as_str), Some("custom-model"));
        assert!(models.iter().any(|model| model == "gpt-5.6-luna"));
    }

    #[test]
    fn does_not_duplicate_known_model() {
        let models = model_options("deepseek-chat");
        assert_eq!(
            models
                .iter()
                .filter(|model| *model == "deepseek-chat")
                .count(),
            1
        );
    }

    #[test]
    fn groups_models_by_provider() {
        let groups = provider_groups("custom-model");
        assert!(groups.iter().any(|g| g.provider == "DeepSeek"));
        assert!(groups
            .iter()
            .any(|g| g.provider == "DeepSeek (modlens vision)"));
        assert!(groups
            .iter()
            .any(|g| g.provider == "bytecat" && g.models.contains(&"custom-model".to_string())));
    }
}
