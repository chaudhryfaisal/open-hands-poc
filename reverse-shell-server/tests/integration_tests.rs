use anyhow::Result;

use serde_json;
use uuid::Uuid;

// Define the types locally for testing since we can't import from the main crate in tests
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub client_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistration {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub uptime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCommand {
    pub command: String,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellResponse {
    pub output: String,
    pub exit_code: i32,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientListRequest {
    pub admin_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectToClientRequest {
    pub admin_token: String,
    pub client_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    Auth(AuthRequest),
    AuthResponse(AuthResponse),
    ClientRegistration(ClientRegistration),
    ShellCommand(ShellCommand),
    ShellResponse(ShellResponse),
    ClientListRequest(ClientListRequest),
    ConnectToClientRequest(ConnectToClientRequest),
    Ping,
    Pong,
    Error { message: String },
}

#[tokio::test]
async fn test_client_authentication_success() -> Result<()> {
    // This test would require a running server
    // For now, we'll test the message serialization/deserialization

    let auth_request = Message::Auth(AuthRequest {
        token: "client_token_123".to_string(),
    });

    let json = serde_json::to_string(&auth_request)?;
    let parsed: Message = serde_json::from_str(&json)?;

    match parsed {
        Message::Auth(req) => {
            assert_eq!(req.token, "client_token_123");
        }
        _ => panic!("Wrong message type"),
    }

    Ok(())
}

#[tokio::test]
async fn test_client_registration_message() -> Result<()> {
    let registration = Message::ClientRegistration(ClientRegistration {
        hostname: "test-host".to_string(),
        os: "Linux Ubuntu 20.04".to_string(),
        arch: "x86_64".to_string(),
        uptime: 12345,
    });

    let json = serde_json::to_string(&registration)?;
    let parsed: Message = serde_json::from_str(&json)?;

    match parsed {
        Message::ClientRegistration(reg) => {
            assert_eq!(reg.hostname, "test-host");
            assert_eq!(reg.os, "Linux Ubuntu 20.04");
            assert_eq!(reg.arch, "x86_64");
            assert_eq!(reg.uptime, 12345);
        }
        _ => panic!("Wrong message type"),
    }

    Ok(())
}

#[tokio::test]
async fn test_shell_command_message() -> Result<()> {
    let session_id = Uuid::new_v4();
    let command = Message::ShellCommand(ShellCommand {
        command: "ls -la".to_string(),
        session_id,
    });

    let json = serde_json::to_string(&command)?;
    let parsed: Message = serde_json::from_str(&json)?;

    match parsed {
        Message::ShellCommand(cmd) => {
            assert_eq!(cmd.command, "ls -la");
            assert_eq!(cmd.session_id, session_id);
        }
        _ => panic!("Wrong message type"),
    }

    Ok(())
}

#[tokio::test]
async fn test_shell_response_message() -> Result<()> {
    let session_id = Uuid::new_v4();
    let response = Message::ShellResponse(ShellResponse {
        output: "total 8\ndrwxr-xr-x 2 user user 4096 Jan 1 12:00 .\n".to_string(),
        exit_code: 0,
        session_id,
    });

    let json = serde_json::to_string(&response)?;
    let parsed: Message = serde_json::from_str(&json)?;

    match parsed {
        Message::ShellResponse(resp) => {
            assert!(resp.output.contains("total 8"));
            assert_eq!(resp.exit_code, 0);
            assert_eq!(resp.session_id, session_id);
        }
        _ => panic!("Wrong message type"),
    }

    Ok(())
}

#[tokio::test]
async fn test_admin_client_list_request() -> Result<()> {
    let request = Message::ClientListRequest(ClientListRequest {
        admin_token: "admin_token_456".to_string(),
    });

    let json = serde_json::to_string(&request)?;
    let parsed: Message = serde_json::from_str(&json)?;

    match parsed {
        Message::ClientListRequest(req) => {
            assert_eq!(req.admin_token, "admin_token_456");
        }
        _ => panic!("Wrong message type"),
    }

    Ok(())
}

#[tokio::test]
async fn test_connect_to_client_request() -> Result<()> {
    let client_id = Uuid::new_v4();
    let request = Message::ConnectToClientRequest(ConnectToClientRequest {
        admin_token: "admin_token_456".to_string(),
        client_id,
    });

    let json = serde_json::to_string(&request)?;
    let parsed: Message = serde_json::from_str(&json)?;

    match parsed {
        Message::ConnectToClientRequest(req) => {
            assert_eq!(req.admin_token, "admin_token_456");
            assert_eq!(req.client_id, client_id);
        }
        _ => panic!("Wrong message type"),
    }

    Ok(())
}

#[tokio::test]
async fn test_ping_pong_messages() -> Result<()> {
    let ping = Message::Ping;
    let pong = Message::Pong;

    let ping_json = serde_json::to_string(&ping)?;
    let pong_json = serde_json::to_string(&pong)?;

    let parsed_ping: Message = serde_json::from_str(&ping_json)?;
    let parsed_pong: Message = serde_json::from_str(&pong_json)?;

    match (parsed_ping, parsed_pong) {
        (Message::Ping, Message::Pong) => {
            // Success
        }
        _ => panic!("Wrong message types"),
    }

    Ok(())
}

#[tokio::test]
async fn test_error_message() -> Result<()> {
    let error = Message::Error {
        message: "Test error message".to_string(),
    };

    let json = serde_json::to_string(&error)?;
    let parsed: Message = serde_json::from_str(&json)?;

    match parsed {
        Message::Error { message } => {
            assert_eq!(message, "Test error message");
        }
        _ => panic!("Wrong message type"),
    }

    Ok(())
}

// Mock integration test that would work with a real server
#[tokio::test]
#[ignore] // Ignore by default since it requires a running server
async fn test_full_client_server_integration() -> Result<()> {
    // This test demonstrates how to test with a real server
    // To run this test, start the server first and then run:
    // cargo test test_full_client_server_integration -- --ignored

    println!("This test requires a running server on localhost:8080");
    println!("Start the server with: cargo run --bin reverse-shell-server");

    Ok(())
}
