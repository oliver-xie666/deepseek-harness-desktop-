use serde_json::{json, Value};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

pub struct PluginRunner;

impl PluginRunner {
    pub async fn execute_stdio(
        executable: &str,
        args: &[String],
        workdir: &Path,
        input_payload: &Value,
        timeout_ms: u64,
    ) -> Result<Value, String> {
        let mut child = Command::new(executable)
            .args(args)
            .current_dir(workdir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn plugin process '{}': {}", executable, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            let input_bytes = serde_json::to_vec(input_payload)
                .map_err(|e| format!("Failed to serialize plugin input: {}", e))?;
            let _ = stdin.write_all(&input_bytes).await;
            let _ = stdin.write_all(b"\n").await;
            let _ = stdin.flush().await;
        }

        let exec_future = async {
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();

            if let Some(mut stdout) = child.stdout.take() {
                let _ = stdout.read_to_end(&mut stdout_buf).await;
            }
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_end(&mut stderr_buf).await;
            }

            let status = child.wait().await.map_err(|e| e.to_string())?;
            Ok::<_, String>((status, stdout_buf, stderr_buf))
        };

        match timeout(Duration::from_millis(timeout_ms), exec_future).await {
            Ok(Ok((status, stdout_bytes, stderr_bytes))) => {
                let stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
                let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();

                if let Ok(json_val) = serde_json::from_str::<Value>(&stdout) {
                    Ok(json_val)
                } else {
                    Ok(json!({
                        "stdout": stdout,
                        "stderr": stderr,
                        "exit_code": status.code().unwrap_or(-1),
                        "success": status.success()
                    }))
                }
            }
            Ok(Err(e)) => Err(format!("Plugin execution error: {}", e)),
            Err(_) => Err(format!("Plugin process timed out after {}ms", timeout_ms)),
        }
    }
}
