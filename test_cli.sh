#!/bin/bash

# Test script to demonstrate the CLI functionality
source $HOME/.cargo/env

echo "Testing CLI connection to admin interface..."
echo "This will connect to the admin interface and list connected clients"

# Use expect to automate the CLI interaction
expect << 'EOF'
spawn ./target/release/reverse-shell-cli --server localhost:12001 --token admin_token_456 --log-level info

# Wait for client list to appear
expect "Connected Clients"

# Select the first client (index 1)
expect "Select client"
send "1\r"

# Wait for shell prompt
expect "shell>"

# Send a simple command
send "whoami\r"

# Wait for response
expect "shell>"

# Send another command
send "pwd\r"

# Wait for response
expect "shell>"

# Send ls command
send "ls -la\r"

# Wait for response
expect "shell>"

# Exit the session
send "exit\r"

expect eof
EOF