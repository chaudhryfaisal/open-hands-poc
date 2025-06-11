#!/bin/bash

# Test script to demonstrate command execution via CLI
source $HOME/.cargo/env

echo "=== Testing Command Execution ==="

# Create a test script that will interact with the CLI
cat > cli_test.exp << 'EOF'
#!/usr/bin/expect -f

set timeout 10

spawn ./target/release/reverse-shell-cli --server localhost:12001 --token admin_token_456 --log-level warn

# Wait for client list
expect "Connected Clients"

# Select client 1
expect "Select client"
send "1\r"

# Wait for shell prompt
expect "shell>"

# Test whoami command
send "whoami\r"
expect "shell>"

# Test pwd command
send "pwd\r"
expect "shell>"

# Test ls command
send "ls -la /tmp\r"
expect "shell>"

# Test echo command
send "echo 'Hello from reverse shell!'\r"
expect "shell>"

# Test date command
send "date\r"
expect "shell>"

# Exit
send "exit\r"

expect eof
EOF

chmod +x cli_test.exp

echo "Running automated CLI test..."
./cli_test.exp

echo
echo "Test completed!"

# Clean up
rm -f cli_test.exp