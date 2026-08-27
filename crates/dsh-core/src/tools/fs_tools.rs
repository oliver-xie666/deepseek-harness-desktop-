use crate::diff_applier::DiffApplier;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn resolve_path(workspace: &Path, rel_path: &str) -> PathBuf {
    let p = Path::new(rel_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace.join(p)
    }
}

pub fn execute_read_file(workspace: &Path, input: &Value) -> Result<Value, String> {
    let path_str = input
        .get("path")
        .or_else(|| input.get("filepath"))
        .or_else(|| input.get("file"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'path' argument in read_file".to_string())?;

    let path = resolve_path(workspace, path_str);
    if !path.exists() {
        return Err(format!("File does not exist: {}", path_str));
    }
    if path.is_dir() {
        return Err(format!("Path is a directory, not a file: {}", path_str));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file {}: {}", path_str, e))?;

    let lines: Vec<&str> = content.lines().collect();
    let offset = input
        .get("offset")
        .and_then(|v| v.as_u64())
        .unwrap_or(1)
        .max(1) as usize;
    let limit = input
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    let start_idx = (offset - 1).min(lines.len());
    let end_idx = match limit {
        Some(lim) => (start_idx + lim).min(lines.len()),
        None => lines.len(),
    };

    let mut numbered_content = String::new();
    for (i, line) in lines[start_idx..end_idx].iter().enumerate() {
        let line_num = start_idx + i + 1;
        numbered_content.push_str(&format!("{:>5}| {}\n", line_num, line));
    }

    Ok(json!({
        "path": path_str,
        "total_lines": lines.len(),
        "displayed_lines": end_idx - start_idx,
        "content": numbered_content,
        "raw_content": content,
    }))
}

pub fn execute_write_file(
    workspace: &Path,
    input: &Value,
) -> Result<(Value, Option<String>), String> {
    let path_str = input
        .get("path")
        .or_else(|| input.get("filepath"))
        .or_else(|| input.get("file"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'path' argument in write_file".to_string())?;

    let content = input
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'content' argument in write_file".to_string())?;

    let path = resolve_path(workspace, path_str);
    let old_content = if path.exists() {
        fs::read_to_string(&path).ok()
    } else {
        None
    };

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Failed to create parent directories for {}: {}",
                    path_str, e
                )
            })?;
        }
    }

    // Atomic write via tempfile
    let temp_file = path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
    fs::write(&temp_file, content).map_err(|e| format!("Failed to write temporary file: {}", e))?;
    fs::rename(&temp_file, &path)
        .map_err(|e| format!("Failed to move temp file to target {}: {}", path_str, e))?;

    let diff = old_content.map(|old| {
        format!(
            "--- a/{}\n+++ b/{}\n@@ -1 +1 @@\n- <old content ({} bytes)>\n+ <new content ({} bytes)>",
            path_str,
            path_str,
            old.len(),
            content.len()
        )
    });

    Ok((
        json!({
            "path": path_str,
            "success": true,
            "bytes_written": content.len(),
        }),
        diff,
    ))
}

pub fn execute_edit_file(
    workspace: &Path,
    input: &Value,
) -> Result<(Value, Option<String>), String> {
    let path_str = input
        .get("path")
        .or_else(|| input.get("filepath"))
        .or_else(|| input.get("file"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'path' argument in edit_file".to_string())?;

    let old_str = input
        .get("old_str")
        .or_else(|| input.get("old_string"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'old_str' argument in edit_file".to_string())?;

    let new_str = input
        .get("new_str")
        .or_else(|| input.get("new_string"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'new_str' argument in edit_file".to_string())?;

    let path = resolve_path(workspace, path_str);
    if !path.exists() {
        return Err(format!("File does not exist: {}", path_str));
    }

    let original = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file {}: {}", path_str, e))?;

    let occurrences = original.matches(old_str).count();
    if occurrences == 0 {
        return Err(format!(
            "Target 'old_str' not found in {}. Ensure exact whitespace and linebreaks match.",
            path_str
        ));
    }
    if occurrences > 1 {
        return Err(format!(
            "Target 'old_str' found {} times in {}. Please provide a larger unique context.",
            occurrences, path_str
        ));
    }

    let replaced = original.replacen(old_str, new_str, 1);
    fs::write(&path, &replaced)
        .map_err(|e| format!("Failed to save edited file {}: {}", path_str, e))?;

    let diff = format!(
        "--- a/{}\n+++ b/{}\n@@ edit @@\n- {}\n+ {}",
        path_str, path_str, old_str, new_str
    );

    Ok((
        json!({
            "path": path_str,
            "success": true,
            "replaced_occurrences": 1,
        }),
        Some(diff),
    ))
}

fn extract_path_from_diff(diff: &str) -> Option<String> {
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            if rest != "/dev/null" {
                return Some(rest.trim().to_string());
            }
        }
        if let Some(rest) = line.strip_prefix("--- a/") {
            if rest != "/dev/null" {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

pub fn execute_apply_patch(
    workspace: &Path,
    input: &Value,
) -> Result<(Value, Option<String>), String> {
    let patch = input
        .get("patch")
        .or_else(|| input.get("diff"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'patch' argument in apply_patch".to_string())?;

    let path_str = input
        .get("path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| extract_path_from_diff(patch))
        .ok_or_else(|| "Could not determine target file path for diff".to_string())?;

    DiffApplier::apply_unified_diff(workspace, &path_str, patch)
        .map_err(|e| format!("Patch application failed: {}", e))?;

    Ok((
        json!({
            "path": path_str,
            "success": true,
            "applied_patch": true,
        }),
        Some(patch.to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_tools_crud() {
        let temp_dir = std::env::temp_dir().join(format!("dsh_test_fs_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let ws = temp_dir.as_path();

        // 1. write_file
        let write_input = json!({
            "path": "hello.txt",
            "content": "Line 1\nLine 2\nLine 3"
        });
        let (write_res, _) = execute_write_file(ws, &write_input).unwrap();
        assert_eq!(write_res["success"], true);

        // 2. read_file
        let read_input = json!({
            "path": "hello.txt",
            "offset": 2,
            "limit": 2
        });
        let read_res = execute_read_file(ws, &read_input).unwrap();
        assert_eq!(read_res["total_lines"], 3);
        assert_eq!(read_res["displayed_lines"], 2);

        // 3. edit_file
        let edit_input = json!({
            "path": "hello.txt",
            "old_str": "Line 2",
            "new_str": "Line Two (Updated)"
        });
        let (edit_res, diff) = execute_edit_file(ws, &edit_input).unwrap();
        assert_eq!(edit_res["success"], true);
        assert!(diff.is_some());

        // Verify edited content
        let verify_read = execute_read_file(ws, &json!({ "path": "hello.txt" })).unwrap();
        assert!(verify_read["raw_content"]
            .as_str()
            .unwrap()
            .contains("Line Two (Updated)"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
