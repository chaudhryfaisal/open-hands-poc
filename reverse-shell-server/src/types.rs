use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: Uuid,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub uptime: u64,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

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
    Auth(AuthRequest),
    AuthResponse(AuthResponse),
    ClientRegistration(ClientRegistration),
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

#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub info: ClientInfo,
    pub sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

#[derive(Debug, Clone)]
pub struct AdminSession {
    pub id: Uuid,
    pub connected_client_id: Option<Uuid>,
    pub sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

pub type ClientMap = HashMap<Uuid, ConnectedClient>;
pub type AdminMap = HashMap<Uuid, AdminSession>;
