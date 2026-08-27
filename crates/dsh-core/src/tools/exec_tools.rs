use serde_json::{json, Value};
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;

pub async fn execute_command(workspace: &Path, input: &Value) -> Result<Value, String> {
    let cmd_str = input
        .get("cmd")
        .or_else(|| input.get("command"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'cmd' argument in exec_command".to_string())?;

    let workdir_str = input.get("workdir").and_then(|v| v.as_str());
    let run_dir = match workdir_str {
        Some(wd) => {
            let p = Path::new(wd);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                workspace.join(p)
            }
        }
        None => workspace.to_path_buf(),
    };

    let timeout_ms = input
        .get("timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(30000);

    let start = Instant::now();

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut c = Command::new("powershell.exe");
        c.arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(cmd_str);
        c
    };

    #[cfg(not(target_os = "windows"))]
    let mut command = {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd_str);
        c
    };

    command
        .current_dir(&run_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = command
        .spawn()
        .map_err(|e| format!("Failed to spawn command '{}': {}", cmd_str, e))?;

    let output_res = timeout(Duration::from_millis(timeout_ms), child.wait_with_output()).await;

    let duration_ms = start.elapsed().as_millis() as u64;

    match output_res {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            Ok(json!({
                "cmd": cmd_str,
                "exit_code": exit_code,
                "stdout": stdout,
                "stderr": stderr,
                "duration_ms": duration_ms,
                "success": output.status.success(),
            }))
        }
        Ok(Err(e)) => Err(format!("Command execution error: {}", e)),
        Err(_) => Err(format!(
            "Command timed out after {}ms: {}",
            timeout_ms, cmd_str
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_exec_echo() {
        let temp_dir = std::env::temp_dir().join(format!("dsh_test_exec_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let ws = temp_dir.as_path();

        let input = json!({
            "cmd": "echo 'dsh_native_test'"
        });
        let res = execute_command(ws, &input).await.unwrap();
        assert_eq!(res["success"], true);
        assert!(res["stdout"].as_str().unwrap().contains("dsh_native_test"));
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
