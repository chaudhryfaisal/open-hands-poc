use crate::auth::AuthManager;
use crate::types::*;
use anyhow::Result;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct ReverseShellServer {
    clients: Arc<RwLock<ClientMap>>,
    admins: Arc<RwLock<AdminMap>>,
    auth_manager: Arc<AuthManager>,
    client_port: u16,
    admin_port: u16,
}

impl ReverseShellServer {
    pub fn new(client_port: u16, admin_port: u16) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            admins: Arc::new(RwLock::new(HashMap::new())),
            auth_manager: Arc::new(AuthManager::new()),
            client_port,
            admin_port,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting reverse shell server...");
        info!("Client endpoint: 0.0.0.0:{}", self.client_port);
        info!("Admin endpoint: 0.0.0.0:{}", self.admin_port);

        let client_listener = TcpListener::bind(format!("0.0.0.0:{}", self.client_port)).await?;
        let admin_listener = TcpListener::bind(format!("0.0.0.0:{}", self.admin_port)).await?;

        let clients = Arc::clone(&self.clients);
        let admins = Arc::clone(&self.admins);
        let auth_manager = Arc::clone(&self.auth_manager);

        // Start client endpoint
        let client_task = {
            let clients = Arc::clone(&clients);
            let admins = Arc::clone(&admins);
            let auth_manager = Arc::clone(&auth_manager);
            tokio::spawn(async move {
                loop {
                    match client_listener.accept().await {
                        Ok((stream, addr)) => {
                            info!("New client connection from: {}", addr);
                            let clients = Arc::clone(&clients);
                            let admins = Arc::clone(&admins);
                            let auth_manager = Arc::clone(&auth_manager);
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_client_connection(
                                    stream,
                                    addr,
                                    clients,
                                    admins,
                                    auth_manager,
                                )
                                .await
                                {
                                    error!("Client connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => error!("Failed to accept client connection: {}", e),
                    }
                }
            })
        };

        // Start admin endpoint
        let admin_task = {
            let clients = Arc::clone(&clients);
            let admins = Arc::clone(&admins);
            let auth_manager = Arc::clone(&auth_manager);
            tokio::spawn(async move {
                loop {
                    match admin_listener.accept().await {
                        Ok((stream, addr)) => {
                            info!("New admin connection from: {}", addr);
                            let clients = Arc::clone(&clients);
                            let admins = Arc::clone(&admins);
                            let auth_manager = Arc::clone(&auth_manager);
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_admin_connection(
                                    stream,
                                    addr,
                                    clients,
                                    admins,
                                    auth_manager,
                                )
                                .await
                                {
                                    error!("Admin connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => error!("Failed to accept admin connection: {}", e),
                    }
                }
            })
        };

        // Start cleanup task
        let cleanup_task = {
            let clients = Arc::clone(&clients);
            let admins = Arc::clone(&admins);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    Self::cleanup_disconnected_clients(&clients, &admins).await;
                }
            })
        };

        tokio::select! {
            _ = client_task => error!("Client task ended unexpectedly"),
            _ = admin_task => error!("Admin task ended unexpectedly"),
            _ = cleanup_task => error!("Cleanup task ended unexpectedly"),
        }

        Ok(())
    }

    async fn handle_client_connection(
        stream: TcpStream,
        addr: SocketAddr,
        clients: Arc<RwLock<ClientMap>>,
        admins: Arc<RwLock<AdminMap>>,
        auth_manager: Arc<AuthManager>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // Handle outgoing messages
        let sender_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let json = serde_json::to_string(&message).unwrap();
                if ws_sender.send(WsMessage::Text(json)).await.is_err() {
                    break;
                }
            }
        });

        let mut client_id: Option<Uuid> = None;
        let mut authenticated = false;

        // Handle incoming messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    match serde_json::from_str::<Message>(&text) {
                        Ok(Message::Auth(auth_req)) => {
                            if auth_manager.verify_client_token(&auth_req.token) {
                                authenticated = true;
                                let new_client_id = Uuid::new_v4();
                                client_id = Some(new_client_id);

                                let response = AuthResponse {
                                    success: true,
                                    message: "Authentication successful".to_string(),
                                    client_id: Some(new_client_id),
                                };

                                if tx.send(Message::AuthResponse(response)).is_err() {
                                    break;
                                }

                                info!("Client {} authenticated successfully", new_client_id);
                            } else {
                                let response = AuthResponse {
                                    success: false,
                                    message: "Authentication failed".to_string(),
                                    client_id: None,
                                };

                                if tx.send(Message::AuthResponse(response)).is_err() {
                                    break;
                                }

                                warn!("Client authentication failed from {}", addr);
                                break;
                            }
                        }
                        Ok(Message::ClientRegistration(reg)) => {
                            if !authenticated {
                                warn!("Unauthenticated client attempted registration");
                                break;
                            }

                            if let Some(id) = client_id {
                                let client_info = ClientInfo {
                                    id,
                                    hostname: reg.hostname,
                                    os: reg.os,
                                    arch: reg.arch,
                                    uptime: reg.uptime,
                                    connected_at: Utc::now(),
                                    last_seen: Utc::now(),
                                };

                                let connected_client = ConnectedClient {
                                    info: client_info.clone(),
                                    sender: tx.clone(),
                                };

                                clients.write().await.insert(id, connected_client);
                                info!("Client registered: {} ({})", client_info.hostname, id);
                            }
                        }
                        Ok(Message::ShellResponse(response)) => {
                            if !authenticated {
                                warn!("Unauthenticated client attempted shell response");
                                break;
                            }

                            // Forward response to the appropriate admin session
                            let admins_read = admins.read().await;
                            for admin in admins_read.values() {
                                if let Some(connected_client_id) = admin.connected_client_id {
                                    if Some(connected_client_id) == client_id {
                                        let _ = admin
                                            .sender
                                            .send(Message::ShellResponse(response.clone()));
                                    }
                                }
                            }
                        }
                        Ok(Message::Pong) => {
                            if let Some(id) = client_id {
                                if let Some(client) = clients.write().await.get_mut(&id) {
                                    client.info.last_seen = Utc::now();
                                }
                            }
                        }
                        Ok(Message::Ping) => {
                            let _ = tx.send(Message::Pong);
                        }
                        _ => {
                            debug!("Received unexpected message from client");
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Client connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        if let Some(id) = client_id {
            clients.write().await.remove(&id);
            info!("Client {} disconnected", id);
        }

        sender_task.abort();
        Ok(())
    }

    async fn handle_admin_connection(
        stream: TcpStream,
        addr: SocketAddr,
        clients: Arc<RwLock<ClientMap>>,
        admins: Arc<RwLock<AdminMap>>,
        auth_manager: Arc<AuthManager>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // Handle outgoing messages
        let sender_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let json = serde_json::to_string(&message).unwrap();
                if ws_sender.send(WsMessage::Text(json)).await.is_err() {
                    break;
                }
            }
        });

        let admin_id = Uuid::new_v4();
        let mut authenticated = false;

        // Handle incoming messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    match serde_json::from_str::<Message>(&text) {
                        Ok(Message::ClientListRequest(req)) => {
                            if !auth_manager.verify_admin_token(&req.admin_token) {
                                warn!("Admin authentication failed from {}", addr);
                                let error_msg = Message::Error {
                                    message: "Authentication failed".to_string(),
                                };
                                let _ = tx.send(error_msg);
                                continue;
                            }

                            if !authenticated {
                                authenticated = true;
                                let admin_session = AdminSession {
                                    id: admin_id,
                                    connected_client_id: None,
                                    sender: tx.clone(),
                                };
                                admins.write().await.insert(admin_id, admin_session);
                                info!("Admin {} authenticated successfully", admin_id);
                            }

                            let clients_read = clients.read().await;
                            let client_list: Vec<ClientInfo> =
                                clients_read.values().map(|c| c.info.clone()).collect();

                            let response = ClientListResponse {
                                clients: client_list,
                            };

                            let _ = tx.send(Message::ClientListResponse(response));
                        }
                        Ok(Message::ConnectToClientRequest(req)) => {
                            if !auth_manager.verify_admin_token(&req.admin_token) {
                                warn!("Admin authentication failed from {}", addr);
                                let error_msg = Message::Error {
                                    message: "Authentication failed".to_string(),
                                };
                                let _ = tx.send(error_msg);
                                continue;
                            }

                            let clients_read = clients.read().await;
                            if clients_read.contains_key(&req.client_id) {
                                // Update admin session to connect to client
                                if let Some(admin) = admins.write().await.get_mut(&admin_id) {
                                    admin.connected_client_id = Some(req.client_id);
                                }

                                let response = ConnectToClientResponse {
                                    success: true,
                                    message: "Connected to client".to_string(),
                                    session_id: Some(Uuid::new_v4()),
                                };

                                let _ = tx.send(Message::ConnectToClientResponse(response));
                                info!("Admin {} connected to client {}", admin_id, req.client_id);
                            } else {
                                let response = ConnectToClientResponse {
                                    success: false,
                                    message: "Client not found".to_string(),
                                    session_id: None,
                                };

                                let _ = tx.send(Message::ConnectToClientResponse(response));
                            }
                        }
                        Ok(Message::ShellCommand(cmd)) => {
                            if !authenticated {
                                warn!("Unauthenticated admin attempted shell command");
                                break;
                            }

                            // Forward command to connected client
                            if let Some(admin) = admins.read().await.get(&admin_id) {
                                if let Some(client_id) = admin.connected_client_id {
                                    if let Some(client) = clients.read().await.get(&client_id) {
                                        let _ = client.sender.send(Message::ShellCommand(cmd));
                                    }
                                }
                            }
                        }
                        Ok(Message::Ping) => {
                            let _ = tx.send(Message::Pong);
                        }
                        _ => {
                            debug!("Received unexpected message from admin");
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Admin connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        admins.write().await.remove(&admin_id);
        info!("Admin {} disconnected", admin_id);

        sender_task.abort();
        Ok(())
    }

    async fn cleanup_disconnected_clients(
        clients: &Arc<RwLock<ClientMap>>,
        _admins: &Arc<RwLock<AdminMap>>,
    ) {
        let mut to_remove = Vec::new();

        {
            let clients_read = clients.read().await;
            for (id, client) in clients_read.iter() {
                // Send ping to check if client is still alive
                if client.sender.send(Message::Ping).is_err() {
                    to_remove.push(*id);
                }
            }
        }

        if !to_remove.is_empty() {
            let mut clients_write = clients.write().await;
            for id in to_remove {
                clients_write.remove(&id);
                info!("Removed disconnected client: {}", id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = ReverseShellServer::new(8080, 8081);
        assert_eq!(server.client_port, 8080);
        assert_eq!(server.admin_port, 8081);
    }

    #[tokio::test]
    async fn test_client_map_operations() {
        let clients: Arc<RwLock<ClientMap>> = Arc::new(RwLock::new(HashMap::new()));
        let client_id = Uuid::new_v4();

        let (tx, _rx) = mpsc::unbounded_channel();
        let client_info = ClientInfo {
            id: client_id,
            hostname: "test-host".to_string(),
            os: "Linux".to_string(),
            arch: "x86_64".to_string(),
            uptime: 12345,
            connected_at: Utc::now(),
            last_seen: Utc::now(),
        };

        let connected_client = ConnectedClient {
            info: client_info,
            sender: tx,
        };

        clients.write().await.insert(client_id, connected_client);
        assert!(clients.read().await.contains_key(&client_id));

        clients.write().await.remove(&client_id);
        assert!(!clients.read().await.contains_key(&client_id));
    }
}
