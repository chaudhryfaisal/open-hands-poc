#!/bin/bash

# Demonstration script for the Reverse Shell System
source $HOME/.cargo/env

echo "=== Reverse Shell System Demonstration ==="
echo

# Check if server is running
if ! pgrep -f "reverse-shell-server" > /dev/null; then
    echo "Starting server..."
    ./target/release/reverse-shell-server --client-port 12000 --admin-port 12001 --log-level info > server.log 2>&1 &
    sleep 2
fi

# Check if client is running
if ! pgrep -f "reverse-shell-client" > /dev/null; then
    echo "Starting client..."
    ./target/release/reverse-shell-client --server localhost:12000 --token client_token_123 --log-level info > client.log 2>&1 &
    sleep 3
fi

echo "Server and client are running!"
echo
echo "Server logs:"
echo "============"
tail -n 5 server.log
echo
echo "Client logs:"
echo "============"
tail -n 5 client.log
echo

echo "To test the CLI manually, run:"
echo "./target/release/reverse-shell-cli --server localhost:12001 --token admin_token_456"
echo
echo "Then select client '1' and try commands like:"
echo "- whoami"
echo "- pwd"
echo "- ls -la"
echo "- ps aux"
echo "- exit (to quit)"
echo

echo "=== System Status ==="
echo "Server PID: $(pgrep -f reverse-shell-server)"
echo "Client PID: $(pgrep -f reverse-shell-client)"
echo
echo "To stop all processes:"
echo "pkill -f reverse-shell"