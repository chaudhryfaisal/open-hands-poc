# Secure Reverse Shell System

A production-ready reverse shell system implemented in Rust with WebSocket communication, SSL support, authentication, and comprehensive logging.

## ⚠️ AI-Generated Code Disclaimer

**This project was entirely generated using AI assistance (OpenHands/Claude) based on the following prompt:**

> Write simple reverse Shell client and server in rust, must have some authentication, all communication should be over websockets and ssl, ssl should be optional, implement properly logging and production ready code , add unit and integration test cases, implement full functionality and don't leave anything for user to implement, compile and test the code 
> 
> Client starts and connects with server, sends its hostname, up and some details to server, client should reconnect if connection is ever unstable or lost, client should always be connected to server
> 
> Server has two endpoints 
> One endpoint for all clients to connect and second for the cli
> 
> Server starts and when client connect it receives its hostname other details and adds to list of connected clients along with the date
> 
> When cli connects it gives list connected clients with session duration to choose from, cli can choose client and will be connected to client shell so it can run debugging commands, cli should have an option to choose client by id or by index

**Important Notes:**
- This code is for educational and authorized testing purposes only
- Always ensure you have proper authorization before using reverse shell tools
- Review and understand the code before deploying in any environment
- The AI-generated code should be thoroughly tested and audited for production use
- Consider security implications and compliance requirements for your specific use case

## 🔒 Security Warning

This is a reverse shell system that allows remote command execution. Use responsibly and only on systems you own or have explicit permission to test.

## Features

- **Secure Authentication**: Token-based authentication for both clients and administrators
- **WebSocket Communication**: Real-time bidirectional communication over WebSockets
- **SSL Support**: Optional SSL/TLS encryption for secure communication
- **Automatic Reconnection**: Clients automatically reconnect on connection loss
- **Production Logging**: Comprehensive structured logging with configurable levels
- **Shell Command Execution**: Secure shell command execution with safety checks
- **Multi-Client Management**: Server can handle multiple clients simultaneously
- **Interactive CLI**: User-friendly command-line interface for administrators
- **Comprehensive Testing**: Unit and integration tests for all components

## Architecture

The system consists of three main components:

1. **Server** (`reverse-shell-server`): Manages client connections and admin sessions
2. **Client** (`reverse-shell-client`): Connects to server and executes commands
3. **CLI** (`reverse-shell-cli`): Administrative interface for interacting with clients

## Security Features

- **Token-based Authentication**: Secure authentication using bcrypt-hashed tokens
- **Command Validation**: Basic protection against dangerous commands
- **SSL/TLS Support**: Optional encryption for all communications
- **Input Sanitization**: Proper handling and validation of all inputs
- **Secure Error Handling**: No sensitive information exposed in error messages

## Quick Start

### Building the Project

```bash
# Clone the repository
git clone <repository-url>
cd open-hands-poc

# Build all components
cargo build --release
```

### Running the Server

```bash
# Start the server with default settings
cargo run --bin reverse-shell-server

# Or with custom ports and SSL
cargo run --bin reverse-shell-server -- --client-port 8080 --admin-port 8081 --log-level debug
```

### Running a Client

```bash
# Connect a client to the server
cargo run --bin reverse-shell-client -- --server localhost:8080 --token client_token_123

# With SSL enabled
cargo run --bin reverse-shell-client -- --server localhost:8080 --token client_token_123 --ssl
```

### Using the CLI

```bash
# Connect to the admin interface
cargo run --bin reverse-shell-cli -- --server localhost:8081 --token admin_token_456

# With SSL enabled
cargo run --bin reverse-shell-cli -- --server localhost:8081 --token admin_token_456 --ssl
```

## Quick Start with Makefile

The project includes a comprehensive Makefile for easy management:

```bash
# Build all components
make build

# Quick start (builds and runs server + client in background)
make quick-start

# Run individual components
make server          # Start server (foreground)
make client          # Start client (foreground)
make cli             # Start CLI interface

# Background processes
make server-bg       # Start server in background
make client-bg       # Start client in background

# Management
make status          # Show process status
make logs            # Show recent logs
make stop            # Stop all processes

# Testing
make test            # Run all tests
make demo            # Run demonstration

# Installation
make install         # Install binaries to ~/.cargo/bin

# Help
make help            # Show all available commands
```

### Environment Variables

You can customize the Makefile behavior with environment variables:

```bash
# Custom ports and tokens
CLIENT_PORT=9000 ADMIN_PORT=9001 make server
CLIENT_TOKEN=my_token ADMIN_TOKEN=my_admin_token make client

# Custom log level
LOG_LEVEL=debug make server
```

## Configuration

### Default Tokens

For demonstration purposes, the system comes with default tokens:
- Client token: `client_token_123`
- Admin token: `admin_token_456`

**⚠️ Important**: Change these tokens in production environments!

### Server Configuration

The server accepts the following command-line arguments:

- `--client-port`: Port for client connections (default: 8080)
- `--admin-port`: Port for admin connections (default: 8081)
- `--log-level`: Logging level (trace, debug, info, warn, error)

### Client Configuration

The client accepts the following command-line arguments:

- `--server`: Server address (default: localhost:8080)
- `--token`: Authentication token (default: client_token_123)
- `--ssl`: Enable SSL/TLS encryption
- `--log-level`: Logging level

### CLI Configuration

The CLI accepts the following command-line arguments:

- `--server`: Server address (default: localhost:8081)
- `--token`: Admin authentication token (default: admin_token_456)
- `--ssl`: Enable SSL/TLS encryption
- `--log-level`: Logging level

## Usage Examples

### Basic Usage

1. Start the server:
```bash
cargo run --bin reverse-shell-server
```

2. Connect a client:
```bash
cargo run --bin reverse-shell-client
```

3. Use the CLI to interact with the client:
```bash
cargo run --bin reverse-shell-cli
```

### With Custom Configuration

1. Start server with custom ports:
```bash
cargo run --bin reverse-shell-server -- --client-port 9090 --admin-port 9091
```

2. Connect client to custom port:
```bash
cargo run --bin reverse-shell-client -- --server localhost:9090
```

3. Connect CLI to custom admin port:
```bash
cargo run --bin reverse-shell-cli -- --server localhost:9091
```

## Testing

### Running Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests for a specific component
cargo test -p reverse-shell-server
cargo test -p reverse-shell-client
cargo test -p reverse-shell-cli
```

### Running Integration Tests

```bash
# Run integration tests
cargo test --test integration_tests

# Run with output
cargo test --test integration_tests -- --nocapture
```

### Manual Testing

1. Start the server in one terminal
2. Start one or more clients in separate terminals
3. Use the CLI to connect and execute commands

## API Reference

### WebSocket Messages

The system uses JSON messages over WebSocket connections. All messages follow this structure:

```json
{
  "type": "MessageType",
  "field1": "value1",
  "field2": "value2"
}
```

#### Client Messages

- `Auth`: Client authentication
- `ClientRegistration`: Client system information
- `ShellResponse`: Command execution results
- `Ping`/`Pong`: Keep-alive messages

#### Admin Messages

- `ClientListRequest`: Request list of connected clients
- `ConnectToClientRequest`: Connect to a specific client
- `ShellCommand`: Execute command on connected client

#### Server Responses

- `AuthResponse`: Authentication result
- `ClientListResponse`: List of connected clients
- `ConnectToClientResponse`: Connection result
- `Error`: Error messages

## Security Considerations

### Production Deployment

1. **Change Default Tokens**: Always use strong, unique tokens in production
2. **Enable SSL**: Use SSL/TLS encryption for all communications
3. **Network Security**: Deploy behind firewalls and use VPNs when possible
4. **Access Control**: Limit access to admin interfaces
5. **Monitoring**: Monitor logs for suspicious activities
6. **Regular Updates**: Keep dependencies updated

### Command Safety

The system includes basic protection against dangerous commands, but additional security measures should be implemented:

1. **Whitelist Commands**: Implement command whitelisting for production
2. **User Permissions**: Run clients with minimal required permissions
3. **Sandboxing**: Consider running in containerized environments
4. **Audit Logging**: Log all executed commands for audit purposes

## Troubleshooting

### Common Issues

1. **Connection Refused**: Ensure the server is running and ports are accessible
2. **Authentication Failed**: Verify tokens match between client/server
3. **SSL Errors**: Ensure SSL is enabled on both client and server
4. **Command Execution Fails**: Check client permissions and command syntax

### Debugging

Enable debug logging for detailed information:

```bash
# Server with debug logging
cargo run --bin reverse-shell-server -- --log-level debug

# Client with debug logging
cargo run --bin reverse-shell-client -- --log-level debug
```

## Releases and Downloads

### Pre-built Binaries

Pre-built binaries are automatically generated for multiple platforms via GitHub Actions:

- **Linux**: x86_64, aarch64, armv7 (GNU and musl variants)
- **Windows**: x86_64, i686, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Android**: aarch64, armv7, x86_64, i686

Download the latest release from the [Releases page](https://github.com/chaudhryfaisal/open-hands-poc/releases).

### Docker Images

Docker images are available from GitHub Container Registry:

```bash
# Pull the latest image
docker pull ghcr.io/chaudhryfaisal/open-hands-poc:latest

# Run the server
docker run -p 8080:8080 -p 8081:8081 ghcr.io/chaudhryfaisal/open-hands-poc:latest

# Run with custom configuration
docker run -p 9000:9000 -p 9001:9001 \
  ghcr.io/chaudhryfaisal/open-hands-poc:latest \
  reverse-shell-server --client-port 9000 --admin-port 9001
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/chaudhryfaisal/open-hands-poc.git
cd open-hands-poc

# Build all components
make build

# Or use cargo directly
cargo build --release
```

## CI/CD Pipeline

The project includes comprehensive GitHub Actions workflows:

- **Continuous Integration**: Runs tests, linting, and security audits on every push
- **Multi-platform Builds**: Automatically builds for 15+ target architectures
- **Security Scanning**: Automated dependency vulnerability scanning
- **Code Coverage**: Tracks test coverage and uploads to Codecov
- **Documentation**: Auto-generates and deploys API documentation
- **Docker Images**: Builds and publishes multi-arch container images
- **Releases**: Creates GitHub releases with pre-built binaries

All builds are reproducible and include checksums for integrity verification.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This software is intended for legitimate system administration and debugging purposes only. Users are responsible for ensuring compliance with all applicable laws and regulations. The authors are not responsible for any misuse of this software.
