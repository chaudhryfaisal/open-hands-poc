use anyhow::Result;
use reverse_shell_client::shell::ShellExecutor;

#[tokio::test]
async fn test_shell_executor_simple_command() -> Result<()> {
    let (output, exit_code) = ShellExecutor::execute_command("echo hello world").await?;
    assert!(output.contains("hello world"));
    assert_eq!(exit_code, 0);
    Ok(())
}

#[tokio::test]
async fn test_shell_executor_failing_command() -> Result<()> {
    let (_, exit_code) = ShellExecutor::execute_command("false").await?;
    assert_ne!(exit_code, 0);
    Ok(())
}

#[tokio::test]
async fn test_shell_executor_command_with_stderr() -> Result<()> {
    let command = if cfg!(target_os = "windows") {
        "echo error 1>&2"
    } else {
        "echo error >&2"
    };
    
    let (output, _) = ShellExecutor::execute_command(command).await?;
    assert!(output.contains("error"));
    Ok(())
}

#[test]
fn test_command_validation() {
    assert!(ShellExecutor::validate_command("ls -la").is_ok());
    assert!(ShellExecutor::validate_command("echo hello").is_ok());
    assert!(ShellExecutor::validate_command("pwd").is_ok());
    
    // Test empty command
    assert!(ShellExecutor::validate_command("").is_err());
    assert!(ShellExecutor::validate_command("   ").is_err());
    
    // Test dangerous commands
    assert!(ShellExecutor::validate_command("rm -rf /").is_err());
    assert!(ShellExecutor::validate_command("format c:").is_err());
    assert!(ShellExecutor::validate_command("shutdown -h now").is_err());
}

#[tokio::test]
async fn test_shell_executor_multiline_output() -> Result<()> {
    let command = if cfg!(target_os = "windows") {
        "echo line1 && echo line2"
    } else {
        "echo line1; echo line2"
    };
    
    let (output, exit_code) = ShellExecutor::execute_command(command).await?;
    assert!(output.contains("line1"));
    assert!(output.contains("line2"));
    assert_eq!(exit_code, 0);
    Ok(())
}

#[tokio::test]
async fn test_shell_executor_long_running_command() -> Result<()> {
    let command = if cfg!(target_os = "windows") {
        "ping -n 2 127.0.0.1"
    } else {
        "sleep 1 && echo done"
    };
    
    let (output, exit_code) = ShellExecutor::execute_command(command).await?;
    assert_eq!(exit_code, 0);
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn test_dangerous_command_patterns() {
    let dangerous_commands = [
        "rm -rf /",
        "format c:",
        "del /s /q c:\\",
        "shutdown -h now",
        "reboot",
        "halt",
        "poweroff",
    ];
    
    for cmd in &dangerous_commands {
        assert!(
            ShellExecutor::validate_command(cmd).is_err(),
            "Command '{}' should be blocked",
            cmd
        );
    }
}

#[test]
fn test_safe_command_patterns() {
    let safe_commands = [
        "ls -la",
        "ps aux",
        "whoami",
        "pwd",
        "cat /etc/passwd",
        "netstat -an",
        "df -h",
        "free -m",
        "uname -a",
    ];
    
    for cmd in &safe_commands {
        assert!(
            ShellExecutor::validate_command(cmd).is_ok(),
            "Command '{}' should be allowed",
            cmd
        );
    }
}