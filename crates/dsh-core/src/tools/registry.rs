use crate::llm::types::ToolDefinition;
use crate::tools::exec_tools::execute_command;
use crate::tools::fs_tools::{
    execute_apply_patch, execute_edit_file, execute_read_file, execute_write_file,
};
use crate::tools::search_tools::{execute_grep_search, execute_list_dir};
use dsh_protocol::ToolStatus;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub output: Value,
    pub status: ToolStatus,
    pub duration_ms: u64,
    pub diff: Option<String>,
    pub produced_file: Option<String>,
}

#[derive(Clone)]
pub struct ToolRegistry {
    custom_tools: HashMap<String, ToolDefinition>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            custom_tools: HashMap::new(),
        }
    }

    pub fn register_custom_tool(&mut self, def: ToolDefinition) {
        self.custom_tools.insert(def.function.name.clone(), def);
    }

    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        let mut tools = vec![
            ToolDefinition::new(
                "read_file",
                "Read file contents with line numbers and optional offset/limit.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative workspace path to read" },
                        "offset": { "type": "integer", "description": "1-based starting line number" },
                        "limit": { "type": "integer", "description": "Maximum lines to read" }
                    },
                    "required": ["path"]
                }),
            ),
            ToolDefinition::new(
                "write_file",
                "Write content to a file atomically. Creates parent directories automatically.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative workspace path to write" },
                        "content": { "type": "string", "description": "Full file content" }
                    },
                    "required": ["path", "content"]
                }),
            ),
            ToolDefinition::new(
                "edit_file",
                "Replace an exact unique string match in a file.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative workspace path" },
                        "old_str": { "type": "string", "description": "Exact text to replace" },
                        "new_str": { "type": "string", "description": "Replacement text" }
                    },
                    "required": ["path", "old_str", "new_str"]
                }),
            ),
            ToolDefinition::new(
                "apply_patch",
                "Apply a unified diff patch to a file or workspace.",
                json!({
                    "type": "object",
                    "properties": {
                        "patch": { "type": "string", "description": "Unified diff patch content" },
                        "path": { "type": "string", "description": "Optional file path" }
                    },
                    "required": ["patch"]
                }),
            ),
            ToolDefinition::new(
                "grep_search",
                "Search files in the workspace using regex or plain text pattern.",
                json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Search pattern or regex" },
                        "path": { "type": "string", "description": "Subdirectory or file path" },
                        "case_sensitive": { "type": "boolean", "description": "Case sensitivity" }
                    },
                    "required": ["pattern"]
                }),
            ),
            ToolDefinition::new(
                "list_dir",
                "List contents of a directory in the workspace.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Directory path (default .)" }
                    }
                }),
            ),
            ToolDefinition::new(
                "exec_command",
                "Execute a shell command in the workspace.",
                json!({
                    "type": "object",
                    "properties": {
                        "cmd": { "type": "string", "description": "Shell command to run" },
                        "workdir": { "type": "string", "description": "Optional sub-directory" },
                        "timeout_ms": { "type": "integer", "description": "Timeout in milliseconds" }
                    },
                    "required": ["cmd"]
                }),
            ),
        ];

        for (_, custom) in &self.custom_tools {
            tools.push(custom.clone());
        }

        tools
    }

    pub async fn dispatch(
        &self,
        workspace: &Path,
        tool_name: &str,
        input: &Value,
    ) -> ToolExecutionResult {
        let start = Instant::now();

        let (output_res, diff, produced_file) = match tool_name {
            "read_file" => (execute_read_file(workspace, input), None, None),
            "write_file" => {
                let file_path = input
                    .get("path")
                    .or_else(|| input.get("filepath"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                match execute_write_file(workspace, input) {
                    Ok((out, diff)) => (Ok(out), diff, file_path),
                    Err(e) => (Err(e), None, None),
                }
            }
            "edit_file" | "str_replace" => {
                let file_path = input
                    .get("path")
                    .or_else(|| input.get("filepath"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                match execute_edit_file(workspace, input) {
                    Ok((out, diff)) => (Ok(out), diff, file_path),
                    Err(e) => (Err(e), None, None),
                }
            }
            "apply_patch" => {
                let file_path = input
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                match execute_apply_patch(workspace, input) {
                    Ok((out, diff)) => (Ok(out), diff, file_path),
                    Err(e) => (Err(e), None, None),
                }
            }
            "grep_search" | "grep" => (execute_grep_search(workspace, input), None, None),
            "list_dir" | "ls" => (execute_list_dir(workspace, input), None, None),
            "exec_command" | "exec" | "shell" => {
                (execute_command(workspace, input).await, None, None)
            }
            unknown => (Err(format!("Unknown tool '{}'", unknown)), None, None),
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        match output_res {
            Ok(output) => ToolExecutionResult {
                output,
                status: ToolStatus::Success,
                duration_ms,
                diff,
                produced_file,
            },
            Err(err_msg) => ToolExecutionResult {
                output: json!({ "error": err_msg }),
                status: ToolStatus::Failed,
                duration_ms,
                diff: None,
                produced_file: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_registry_dispatch() {
        let registry = ToolRegistry::new();
        let temp_dir = std::env::temp_dir().join(format!("dsh_test_reg_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let ws = temp_dir.as_path();

        let res = registry
            .dispatch(
                ws,
                "write_file",
                &json!({ "path": "test.txt", "content": "Hello Registry" }),
            )
            .await;

        assert_eq!(res.status, ToolStatus::Success);
        assert_eq!(res.produced_file, Some("test.txt".to_string()));

        let read_res = registry
            .dispatch(ws, "read_file", &json!({ "path": "test.txt" }))
            .await;
        assert_eq!(read_res.status, ToolStatus::Success);
        assert!(read_res.output["raw_content"]
            .as_str()
            .unwrap()
            .contains("Hello Registry"));
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
