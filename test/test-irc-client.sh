#!/bin/bash
# test-irc-client.sh - Proper IRC client that handles PING/PONG

exec 3<>/dev/tcp/localhost/6697
TLS_CONN="openssl s_client -connect localhost:6697 -quiet 2>/dev/null"

# Start TLS connection
eval "$TLS_CONN" <&3 >&3 &
OPENSSL_PID=$!

# Function to read from server
read_server() {
    timeout 2 cat <&3
}

# Function to write to server
write_server() {
    echo "$1" >&3
}

# Register
write_server "NICK testuser"
write_server "USER test 0 * :Test User"
sleep 2

# Read server response and handle PING
RESPONSE=$(read_server)
echo "Server: $RESPONSE"

# Extract PING token and respond
if [[ $RESPONSE =~ PING\ :([^\r\n]+) ]]; then
    PING_TOKEN="${BASH_REMATCH[1]}"
    echo "Responding to PING :$PING_TOKEN"
    write_server "PONG :$PING_TOKEN"
    sleep 2
fi

# Now join and send messages
write_server "JOIN #zeroclaw"
sleep 2

write_server "PRIVMSG #zeroclaw :hello zeroclaw"
sleep 2

write_server "PRIVMSG #zeroclaw :test message"
sleep 2

write_server "QUIT"

# Cleanup
kill $OPENSSL_PID 2>/dev/null
exec 3<&-
