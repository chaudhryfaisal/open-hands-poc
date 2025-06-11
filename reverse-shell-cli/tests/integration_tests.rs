use anyhow::Result;
use chrono::Utc;
use serde_json;
use uuid::Uuid;

// Define the types locally for testing
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: Uuid,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub uptime: u64,
    pub connected_at: chrono::DateTime<Utc>,
    pub last_seen: chrono::DateTime<Utc>,
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
pub struct ClientListResponse {
    pub clients: Vec<ClientInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectToClientRequest {
    pub admin_token: String,
    pub client_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectToClientResponse {
    pub success: bool,
    pub message: String,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    ShellCommand(ShellCommand),
    ShellResponse(ShellResponse),
    ClientListRequest(ClientListRequest),
    ClientListResponse(ClientListResponse),
    ConnectToClientRequest(ConnectToClientRequest),
    ConnectToClientResponse(ConnectToClientResponse),
    Ping,
    Pong,
    Error { message: String },
}

#[tokio::test]
async fn test_client_list_request_serialization() -> Result<()> {
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
async fn test_client_list_response_serialization() -> Result<()> {
    let client_info = ClientInfo {
        id: Uuid::new_v4(),
        hostname: "test-host".to_string(),
        os: "Linux Ubuntu 20.04".to_string(),
        arch: "x86_64".to_string(),
        uptime: 12345,
        connected_at: Utc::now(),
        last_seen: Utc::now(),
    };
    
    let response = Message::ClientListResponse(ClientListResponse {
        clients: vec![client_info.clone()],
    });
    
    let json = serde_json::to_string(&response)?;
    let parsed: Message = serde_json::from_str(&json)?;
    
    match parsed {
        Message::ClientListResponse(resp) => {
            assert_eq!(resp.clients.len(), 1);
            assert_eq!(resp.clients[0].hostname, "test-host");
            assert_eq!(resp.clients[0].id, client_info.id);
        }
        _ => panic!("Wrong message type"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_connect_to_client_request_serialization() -> Result<()> {
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
async fn test_connect_to_client_response_serialization() -> Result<()> {
    let session_id = Uuid::new_v4();
    let response = Message::ConnectToClientResponse(ConnectToClientResponse {
        success: true,
        message: "Connected successfully".to_string(),
        session_id: Some(session_id),
    });
    
    let json = serde_json::to_string(&response)?;
    let parsed: Message = serde_json::from_str(&json)?;
    
    match parsed {
        Message::ConnectToClientResponse(resp) => {
            assert!(resp.success);
            assert_eq!(resp.message, "Connected successfully");
            assert_eq!(resp.session_id, Some(session_id));
        }
        _ => panic!("Wrong message type"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_shell_command_serialization() -> Result<()> {
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
async fn test_shell_response_serialization() -> Result<()> {
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

#[test]
fn test_client_info_creation() {
    let client_id = Uuid::new_v4();
    let now = Utc::now();
    
    let client_info = ClientInfo {
        id: client_id,
        hostname: "test-host".to_string(),
        os: "Linux Ubuntu 20.04".to_string(),
        arch: "x86_64".to_string(),
        uptime: 12345,
        connected_at: now,
        last_seen: now,
    };
    
    assert_eq!(client_info.id, client_id);
    assert_eq!(client_info.hostname, "test-host");
    assert_eq!(client_info.os, "Linux Ubuntu 20.04");
    assert_eq!(client_info.arch, "x86_64");
    assert_eq!(client_info.uptime, 12345);
}

#[tokio::test]
async fn test_error_message_serialization() -> Result<()> {
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