use regex::RegexBuilder;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn is_ignored_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | ".dsh" | ".vscode" | ".idea" | "dist" | "build"
    )
}

pub fn execute_grep_search(workspace: &Path, input: &Value) -> Result<Value, String> {
    let pattern_str = input
        .get("pattern")
        .or_else(|| input.get("query"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'pattern' argument in grep_search".to_string())?;

    let sub_path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    let case_sensitive = input
        .get("case_sensitive")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let max_results = input
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(100) as usize;

    let regex = RegexBuilder::new(pattern_str)
        .case_insensitive(!case_sensitive)
        .build()
        .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern_str, e))?;

    let search_root = if Path::new(sub_path).is_absolute() {
        PathBuf::from(sub_path)
    } else {
        workspace.join(sub_path)
    };

    if !search_root.exists() {
        return Err(format!("Search path does not exist: {}", sub_path));
    }

    let mut matches = Vec::new();
    let mut files_to_visit = vec![search_root];

    while let Some(current_path) = files_to_visit.pop() {
        if matches.len() >= max_results {
            break;
        }

        if current_path.is_dir() {
            if let Some(name) = current_path.file_name().and_then(|n| n.to_str()) {
                if is_ignored_dir(name) {
                    continue;
                }
            }

            if let Ok(entries) = fs::read_dir(&current_path) {
                for entry in entries.flatten() {
                    files_to_visit.push(entry.path());
                }
            }
        } else if current_path.is_file() {
            // Read file content
            if let Ok(content) = fs::read_to_string(&current_path) {
                let rel_path = current_path
                    .strip_prefix(workspace)
                    .unwrap_or(&current_path)
                    .to_string_lossy()
                    .to_string();

                for (idx, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        matches.push(json!({
                            "file": rel_path,
                            "line_number": idx + 1,
                            "line_text": line.trim_end(),
                        }));
                        if matches.len() >= max_results {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "pattern": pattern_str,
        "total_matches": matches.len(),
        "matches": matches,
    }))
}

pub fn execute_list_dir(workspace: &Path, input: &Value) -> Result<Value, String> {
    let sub_path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    let target_dir = if Path::new(sub_path).is_absolute() {
        PathBuf::from(sub_path)
    } else {
        workspace.join(sub_path)
    };

    if !target_dir.exists() {
        return Err(format!("Directory does not exist: {}", sub_path));
    }
    if !target_dir.is_dir() {
        return Err(format!("Path is not a directory: {}", sub_path));
    }

    let entries = fs::read_dir(&target_dir)
        .map_err(|e| format!("Failed to read directory {}: {}", sub_path, e))?;

    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();
        let size = if is_dir {
            0
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };

        items.push(json!({
            "name": name,
            "is_dir": is_dir,
            "size_bytes": size,
        }));
    }

    items.sort_by(|a, b| {
        let a_dir = a["is_dir"].as_bool().unwrap_or(false);
        let b_dir = b["is_dir"].as_bool().unwrap_or(false);
        if a_dir != b_dir {
            b_dir.cmp(&a_dir)
        } else {
            a["name"].as_str().cmp(&b["name"].as_str())
        }
    });

    Ok(json!({
        "path": sub_path,
        "count": items.len(),
        "entries": items,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_and_list_dir() {
        let temp_dir =
            std::env::temp_dir().join(format!("dsh_test_search_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let ws = temp_dir.as_path();

        fs::write(
            ws.join("test1.rs"),
            "fn hello_world() {}\nfn goodbye() {}\n",
        )
        .unwrap();
        fs::write(ws.join("test2.rs"), "let val = \"hello there\";\n").unwrap();

        let list_res = execute_list_dir(ws, &json!({})).unwrap();
        assert_eq!(list_res["count"], 2);

        let grep_res = execute_grep_search(ws, &json!({ "pattern": "hello" })).unwrap();
        assert_eq!(grep_res["total_matches"], 2);
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
