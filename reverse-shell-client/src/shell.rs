use anyhow::{anyhow, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, warn};

pub struct ShellExecutor;

impl ShellExecutor {
    pub async fn execute_command(command: &str) -> Result<(String, i32)> {
        debug!("Executing command: {}", command);

        // Security: Basic command validation
        if command.trim().is_empty() {
            return Err(anyhow!("Empty command"));
        }

        // Determine shell based on OS
        let (shell, shell_arg) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        let mut child = Command::new(shell)
            .arg(shell_arg)
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stderr"))?;

        let mut stdout_reader = BufReader::new(stdout);
        let mut stderr_reader = BufReader::new(stderr);

        let mut output = String::new();
        let mut stdout_line = String::new();
        let mut stderr_line = String::new();

        // Read output line by line to handle large outputs
        loop {
            tokio::select! {
                result = stdout_reader.read_line(&mut stdout_line) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            output.push_str(&stdout_line);
                            stdout_line.clear();
                        }
                        Err(e) => {
                            error!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
                result = stderr_reader.read_line(&mut stderr_line) => {
                    match result {
                        Ok(0) => {}, // EOF for stderr
                        Ok(_) => {
                            output.push_str(&stderr_line);
                            stderr_line.clear();
                        }
                        Err(e) => {
                            error!("Error reading stderr: {}", e);
                        }
                    }
                }
            }
        }

        // Read any remaining stderr
        loop {
            match stderr_reader.read_line(&mut stderr_line).await {
                Ok(0) => break,
                Ok(_) => {
                    output.push_str(&stderr_line);
                    stderr_line.clear();
                }
                Err(_) => break,
            }
        }

        let exit_status = child.wait().await?;
        let exit_code = exit_status.code().unwrap_or(-1);

        debug!("Command completed with exit code: {}", exit_code);

        // Limit output size to prevent memory issues
        if output.len() > 1024 * 1024 {
            warn!("Command output truncated (too large)");
            output.truncate(1024 * 1024);
            output.push_str("\n[OUTPUT TRUNCATED - TOO LARGE]");
        }

        Ok((output, exit_code))
    }

    pub fn validate_command(command: &str) -> Result<()> {
        // Basic security checks
        let command = command.trim();

        if command.is_empty() {
            return Err(anyhow!("Empty command"));
        }

        // Block potentially dangerous commands (basic protection)
        let dangerous_patterns = [
            "rm -rf /", "format", "del /s", "shutdown", "reboot", "halt", "poweroff",
        ];

        let command_lower = command.to_lowercase();
        for pattern in &dangerous_patterns {
            if command_lower.contains(pattern) {
                warn!("Blocked potentially dangerous command: {}", command);
                return Err(anyhow!("Command blocked for security reasons"));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_simple_command() {
        let (output, exit_code) = ShellExecutor::execute_command("echo hello").await.unwrap();
        assert!(output.contains("hello"));
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_execute_failing_command() {
        let (_, exit_code) = ShellExecutor::execute_command("false").await.unwrap();
        assert_ne!(exit_code, 0);
    }

    #[test]
    fn test_validate_command() {
        assert!(ShellExecutor::validate_command("ls -la").is_ok());
        assert!(ShellExecutor::validate_command("echo hello").is_ok());
        assert!(ShellExecutor::validate_command("").is_err());
        assert!(ShellExecutor::validate_command("rm -rf /").is_err());
    }

    #[tokio::test]
    async fn test_command_with_stderr() {
        let (output, _) = ShellExecutor::execute_command("echo error >&2")
            .await
            .unwrap();
        assert!(output.contains("error"));
    }
}
