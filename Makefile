# Makefile for Reverse Shell System
# Simple tasks for running client, server, and CLI

.PHONY: help build clean test server client cli demo stop logs install

# Default target
help:
	@echo "Reverse Shell System - Available Commands:"
	@echo ""
	@echo "Build Commands:"
	@echo "  build         - Build all components in release mode"
	@echo "  build-debug   - Build all components in debug mode"
	@echo "  test          - Run all tests"
	@echo "  clean         - Clean build artifacts"
	@echo ""
	@echo "Runtime Commands:"
	@echo "  server        - Start the server (ports 12000/12001)"
	@echo "  client        - Start a client"
	@echo "  cli           - Start the CLI interface"
	@echo "  demo          - Run demonstration script"
	@echo ""
	@echo "Management Commands:"
	@echo "  stop          - Stop all running components"
	@echo "  logs          - Show logs from server and client"
	@echo "  status        - Show status of running processes"
	@echo ""
	@echo "Installation:"
	@echo "  install       - Install binaries to ~/.cargo/bin"
	@echo ""
	@echo "Configuration:"
	@echo "  CLIENT_PORT   - Client port (default: 12000)"
	@echo "  ADMIN_PORT    - Admin port (default: 12001)"
	@echo "  CLIENT_TOKEN  - Client authentication token"
	@echo "  ADMIN_TOKEN   - Admin authentication token"
	@echo "  LOG_LEVEL     - Log level (default: info)"

# Configuration variables
CLIENT_PORT ?= 12000
ADMIN_PORT ?= 12001
CLIENT_TOKEN ?= client_token_123
ADMIN_TOKEN ?= admin_token_456
LOG_LEVEL ?= info

# Build targets
build:
	cargo build --release

build-debug:
	cargo build

test:
	cargo test

clean:
	cargo clean
	rm -f *.log

# Runtime targets
server:
	@echo "Starting reverse shell server..."
	@echo "Client endpoint: localhost:$(CLIENT_PORT)"
	@echo "Admin endpoint: localhost:$(ADMIN_PORT)"
	@echo "Press Ctrl+C to stop"
	./target/release/reverse-shell-server \
		--client-port $(CLIENT_PORT) \
		--admin-port $(ADMIN_PORT) \
		--log-level $(LOG_LEVEL)

server-bg:
	@echo "Starting reverse shell server in background..."
	./target/release/reverse-shell-server \
		--client-port $(CLIENT_PORT) \
		--admin-port $(ADMIN_PORT) \
		--log-level $(LOG_LEVEL) > server.log 2>&1 &
	@echo "Server started with PID: $$(pgrep -f reverse-shell-server)"

client:
	@echo "Starting reverse shell client..."
	@echo "Connecting to: localhost:$(CLIENT_PORT)"
	@echo "Press Ctrl+C to stop"
	./target/release/reverse-shell-client \
		--server localhost:$(CLIENT_PORT) \
		--token $(CLIENT_TOKEN) \
		--log-level $(LOG_LEVEL)

client-bg:
	@echo "Starting reverse shell client in background..."
	./target/release/reverse-shell-client \
		--server localhost:$(CLIENT_PORT) \
		--token $(CLIENT_TOKEN) \
		--log-level $(LOG_LEVEL) > client.log 2>&1 &
	@echo "Client started with PID: $$(pgrep -f reverse-shell-client)"

cli:
	@echo "Starting reverse shell CLI..."
	@echo "Connecting to admin interface: localhost:$(ADMIN_PORT)"
	./target/release/reverse-shell-cli \
		--server localhost:$(ADMIN_PORT) \
		--token $(ADMIN_TOKEN) \
		--log-level $(LOG_LEVEL)

# Demo and testing
demo: build
	@echo "Running demonstration..."
	./demo.sh

test-integration: build server-bg
	@echo "Waiting for server to start..."
	@sleep 2
	@echo "Starting client..."
	@$(MAKE) client-bg
	@sleep 3
	@echo "Testing CLI connection..."
	@timeout 5 ./target/release/reverse-shell-cli \
		--server localhost:$(ADMIN_PORT) \
		--token $(ADMIN_TOKEN) \
		--log-level warn || true
	@$(MAKE) stop

# Management targets
stop:
	@echo "Stopping all reverse shell processes..."
	@pkill -f reverse-shell || echo "No processes to stop"

logs:
	@echo "=== Server Logs ==="
	@if [ -f server.log ]; then tail -n 10 server.log; else echo "No server logs found"; fi
	@echo ""
	@echo "=== Client Logs ==="
	@if [ -f client.log ]; then tail -n 10 client.log; else echo "No client logs found"; fi

status:
	@echo "=== Process Status ==="
	@echo "Server PID: $$(pgrep -f reverse-shell-server || echo 'Not running')"
	@echo "Client PID: $$(pgrep -f reverse-shell-client || echo 'Not running')"
	@echo ""
	@echo "=== Network Status ==="
	@netstat -tlnp 2>/dev/null | grep -E ":($(CLIENT_PORT)|$(ADMIN_PORT))" || echo "No listening ports found"

# Installation
install: build
	@echo "Installing binaries to ~/.cargo/bin..."
	cp target/release/reverse-shell-server ~/.cargo/bin/
	cp target/release/reverse-shell-client ~/.cargo/bin/
	cp target/release/reverse-shell-cli ~/.cargo/bin/
	@echo "Installation complete!"
	@echo "Binaries available as:"
	@echo "  - reverse-shell-server"
	@echo "  - reverse-shell-client"
	@echo "  - reverse-shell-cli"

# Development helpers
dev-server: build-debug
	./target/debug/reverse-shell-server \
		--client-port $(CLIENT_PORT) \
		--admin-port $(ADMIN_PORT) \
		--log-level debug

dev-client: build-debug
	./target/debug/reverse-shell-client \
		--server localhost:$(CLIENT_PORT) \
		--token $(CLIENT_TOKEN) \
		--log-level debug

dev-cli: build-debug
	./target/debug/reverse-shell-cli \
		--server localhost:$(ADMIN_PORT) \
		--token $(ADMIN_TOKEN) \
		--log-level debug

# Quick start
quick-start: build server-bg
	@sleep 2
	@$(MAKE) client-bg
	@sleep 2
	@echo ""
	@echo "=== Quick Start Complete ==="
	@echo "Server and client are running in background"
	@echo ""
	@echo "To connect with CLI:"
	@echo "  make cli"
	@echo ""
	@echo "To stop all processes:"
	@echo "  make stop"
	@echo ""
	@$(MAKE) status

# SSL variants
server-ssl:
	./target/release/reverse-shell-server \
		--client-port $(CLIENT_PORT) \
		--admin-port $(ADMIN_PORT) \
		--log-level $(LOG_LEVEL)

client-ssl:
	./target/release/reverse-shell-client \
		--server localhost:$(CLIENT_PORT) \
		--token $(CLIENT_TOKEN) \
		--ssl \
		--log-level $(LOG_LEVEL)

cli-ssl:
	./target/release/reverse-shell-cli \
		--server localhost:$(ADMIN_PORT) \
		--token $(ADMIN_TOKEN) \
		--ssl \
		--log-level $(LOG_LEVEL)