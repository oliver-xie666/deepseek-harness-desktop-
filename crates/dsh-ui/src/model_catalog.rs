const DEFAULT_MODELS: &[&str] = &["gpt-5.6-luna", "deepseek-chat", "deepseek-reasoner"];

pub fn model_options(current: &str) -> Vec<String> {
    let mut models = DEFAULT_MODELS
        .iter()
        .map(|model| (*model).to_string())
        .collect::<Vec<_>>();
    if !current.is_empty() && !models.iter().any(|model| model == current) {
        models.insert(0, current.to_string());
    }
    models
}

#[cfg(test)]
mod tests {
    use super::model_options;

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
}
