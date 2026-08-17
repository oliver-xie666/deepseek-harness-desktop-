use dsh_common::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub icon: String,
}

pub struct WorkspaceScanner;

impl WorkspaceScanner {
    const IGNORED_NAMES: &'static [&'static str] = &[
        ".git",
        "target",
        "node_modules",
        "dist",
        "build",
        ".dsh_data",
        ".dsh_config",
        ".idea",
        ".vscode",
    ];

    pub fn scan_dir(root: &Path, max_depth: usize) -> Result<FileNode> {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
            .to_string();

        let mut node = FileNode {
            path: root.to_path_buf(),
            name,
            is_dir: true,
            children: Vec::new(),
            icon: "📁".to_string(),
        };

        if max_depth == 0 {
            return Ok(node);
        }

        if let Ok(entries) = fs::read_dir(root) {
            let mut sub_nodes = Vec::new();

            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if Self::IGNORED_NAMES.contains(&file_name.as_str()) || file_name.starts_with('.') {
                    continue;
                }

                let path = entry.path();
                let is_dir = path.is_dir();

                if is_dir {
                    if let Ok(child_tree) = Self::scan_dir(&path, max_depth - 1) {
                        sub_nodes.push(child_tree);
                    }
                } else {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    let icon = match ext.as_str() {
                        "rs" => "🦀",
                        "ts" | "tsx" | "js" | "jsx" => "📜",
                        "json" | "toml" | "yaml" | "yml" => "⚙️",
                        "md" => "📝",
                        "sh" | "bat" | "ps1" => "💻",
                        _ => "📄",
                    };

                    sub_nodes.push(FileNode {
                        path,
                        name: file_name,
                        is_dir: false,
                        children: Vec::new(),
                        icon: icon.to_string(),
                    });
                }
            }

            // Sort: directories first, then alphabetical
            sub_nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            node.children = sub_nodes;
        }

        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_current_dir() {
        let root = Path::new(".");
        let tree = WorkspaceScanner::scan_dir(root, 2).unwrap();
        assert!(tree.is_dir);
        assert!(!tree.children.is_empty());
    }
}
