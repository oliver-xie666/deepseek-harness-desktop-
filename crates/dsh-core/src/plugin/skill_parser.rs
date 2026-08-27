use super::types::SkillDefinition;
use std::fs;
use std::path::Path;

pub fn parse_skill_file(path: &Path) -> Result<SkillDefinition, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read SKILL.md at {}: {}", path.display(), e))?;

    parse_skill_content(&content, path)
}

pub fn parse_skill_content(content: &str, path: &Path) -> Result<SkillDefinition, String> {
    let trimmed = content.trim();

    // Check for YAML frontmatter
    if trimmed.starts_with("---") {
        if let Some(second_dash) = trimmed[3..].find("---") {
            let frontmatter = &trimmed[3..3 + second_dash];
            let body = trimmed[3 + second_dash + 3..].trim();

            let mut name = None;
            let mut description = None;

            for line in frontmatter.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("name:") {
                    name = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                } else if let Some(val) = line.strip_prefix("description:") {
                    description = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }

            let fallback_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed-skill")
                .to_string();

            return Ok(SkillDefinition {
                name: name.unwrap_or(fallback_name),
                description: description.unwrap_or_else(|| "No description provided".to_string()),
                prompt_injection: body.to_string(),
                source_path: path.to_path_buf(),
            });
        }
    }

    // Fallback: Markdown header parsing
    let mut lines = trimmed.lines();
    let mut name = None;
    let mut description = None;
    let mut body_lines = Vec::new();

    for line in lines.by_ref() {
        let l = line.trim();
        if name.is_none() && l.starts_with('#') {
            name = Some(l.trim_start_matches('#').trim().to_string());
        } else if description.is_none() && !l.is_empty() && !l.starts_with('#') {
            description = Some(l.to_string());
            body_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    let default_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("skill")
        .to_string();

    Ok(SkillDefinition {
        name: name.unwrap_or(default_name),
        description: description.unwrap_or_else(|| "Skill instructions".to_string()),
        prompt_injection: body_lines.join("\n"),
        source_path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_yaml_frontmatter_skill() {
        let content = r#"---
name: code-review
description: Expert code review specialist for quality and standards
---
# Instructions
Review the diff thoroughly.
"#;
        let skill =
            parse_skill_content(content, &PathBuf::from("/skills/code-review/SKILL.md")).unwrap();
        assert_eq!(skill.name, "code-review");
        assert_eq!(
            skill.description,
            "Expert code review specialist for quality and standards"
        );
        assert!(skill
            .prompt_injection
            .contains("Review the diff thoroughly"));
    }

    #[test]
    fn test_parse_markdown_header_skill() {
        let content = r#"# Archify System
Create polished architecture and workflow diagrams.

## Guidelines
- Output standalone SVG
"#;
        let skill =
            parse_skill_content(content, &PathBuf::from("/skills/archify/SKILL.md")).unwrap();
        assert_eq!(skill.name, "Archify System");
        assert!(skill.prompt_injection.contains("Output standalone SVG"));
    }
}
