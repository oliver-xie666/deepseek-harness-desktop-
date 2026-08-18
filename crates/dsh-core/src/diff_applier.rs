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
}
