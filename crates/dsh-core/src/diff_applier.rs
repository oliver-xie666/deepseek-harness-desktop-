use dsh_common::{DshError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct DiffApplier;

impl DiffApplier {
    /// Safely writes updated content to the target file using atomic replacement
    pub fn apply_file_content(
        workspace_root: &Path,
        rel_path: &str,
        new_content: &str,
    ) -> Result<PathBuf> {
        let target_path = workspace_root.join(rel_path);

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Temporary file in same directory for atomic replace
        let tmp_path = target_path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));

        fs::write(&tmp_path, new_content)?;

        // Rename tmp to target (atomic on POSIX, replaced on Windows)
        if target_path.exists() {
            let _ = fs::remove_file(&target_path);
        }

        fs::rename(&tmp_path, &target_path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            DshError::Io(e)
        })?;

        info!(
            "Successfully applied file changes to {}",
            target_path.display()
        );
        Ok(target_path)
    }

    /// Reads existing content of a relative workspace file
    pub fn read_file(workspace_root: &Path, rel_path: &str) -> Result<String> {
        let path = workspace_root.join(rel_path);
        fs::read_to_string(&path).map_err(DshError::Io)
    }

    pub fn apply_unified_diff(
        workspace_root: &Path,
        rel_path: &str,
        diff_content: &str,
    ) -> Result<PathBuf> {
        let target_path = workspace_root.join(rel_path);
        let original = if target_path.exists() {
            fs::read_to_string(&target_path)?
        } else {
            String::new()
        };
        let source = original.lines().collect::<Vec<_>>();
        let mut source_index = 0;
        let mut output = Vec::new();
        let mut in_hunk = false;

        for line in diff_content.lines() {
            if line.starts_with("--- ") || line.starts_with("+++ ") {
                continue;
            }
            if line.starts_with("@@ ") {
                let start = parse_hunk_start(line)?;
                if start < source_index || start > source.len() {
                    return Err(DshError::Protocol(
                        "diff hunk position is outside source file".into(),
                    ));
                }
                output.extend(
                    source[source_index..start]
                        .iter()
                        .map(|line| (*line).to_string()),
                );
                source_index = start;
                in_hunk = true;
                continue;
            }
            if !in_hunk {
                continue;
            }

            let (prefix, content) = line.split_at(1);
            match prefix {
                " " => {
                    validate_source_line(&source, source_index, content)?;
                    output.push(content.to_string());
                    source_index += 1;
                }
                "-" => {
                    validate_source_line(&source, source_index, content)?;
                    source_index += 1;
                }
                "+" => output.push(content.to_string()),
                "\\" => {}
                _ => return Err(DshError::Protocol("unsupported unified diff line".into())),
            }
        }

        if !in_hunk {
            return Err(DshError::Protocol("unified diff has no hunk".into()));
        }
        output.extend(
            source[source_index..]
                .iter()
                .map(|line| (*line).to_string()),
        );
        let mut updated = output.join("\n");
        if original.ends_with('\n') {
            updated.push('\n');
        }
        Self::apply_file_content(workspace_root, rel_path, &updated)
    }
}

fn parse_hunk_start(header: &str) -> Result<usize> {
    let range = header
        .split_whitespace()
        .nth(1)
        .and_then(|part| part.strip_prefix('-'))
        .ok_or_else(|| DshError::Protocol("invalid unified diff hunk header".into()))?;
    let line_number = range
        .split(',')
        .next()
        .ok_or_else(|| DshError::Protocol("invalid unified diff hunk range".into()))?
        .parse::<usize>()
        .map_err(|_| DshError::Protocol("invalid unified diff hunk line number".into()))?;
    Ok(line_number.saturating_sub(1))
}

fn validate_source_line(source: &[&str], index: usize, expected: &str) -> Result<()> {
    match source.get(index) {
        Some(actual) if *actual == expected => Ok(()),
        _ => Err(DshError::Protocol(
            "diff context does not match source file".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_atomic_file_apply() {
        let temp_dir = env::temp_dir().join(format!("dsh_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        let rel_file = "src/hello.rs";
        let content = "fn main() { println!(\"Applied Successfully!\"); }\n";

        let applied_path = DiffApplier::apply_file_content(&temp_dir, rel_file, content).unwrap();
        assert!(applied_path.exists());

        let read_back = DiffApplier::read_file(&temp_dir, rel_file).unwrap();
        assert_eq!(read_back, content);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn applies_unified_diff_to_existing_file() {
        let temp_dir = env::temp_dir().join(format!("dsh_diff_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("notes.txt"), "one\ntwo\nthree\n").unwrap();

        let diff = "--- a/notes.txt\n+++ b/notes.txt\n@@ -1,3 +1,3 @@\n one\n-two\n+TWO\n three\n";
        DiffApplier::apply_unified_diff(&temp_dir, "notes.txt", diff).unwrap();

        assert_eq!(
            fs::read_to_string(temp_dir.join("notes.txt")).unwrap(),
            "one\nTWO\nthree\n"
        );
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
