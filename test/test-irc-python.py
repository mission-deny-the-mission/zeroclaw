#!/usr/bin/env python3
"""Simple IRC test client that properly handles PING/PONG"""
import socket
import ssl
import time
import threading

def irc_client():
    # Create TLS connection
    context = ssl.create_default_context()
    context.check_hostname = False
    context.verify_mode = ssl.CERT_NONE
    
    sock = socket.create_connection(('localhost', 6697))
    ssock = context.wrap_socket(sock, server_hostname='localhost')
    
    def send(cmd):
        ssock.send(f"{cmd}\r\n".encode())
        print(f"> {cmd}")
    
    def recv():
        ssock.settimeout(5)
        try:
            data = ssock.recv(4096).decode()
            print(f"< {data}")
            return data
        except:
            return None
    
    # Register
    send("NICK testuser")
    send("USER test 0 * :Test User")
    time.sleep(2)
    
    # Handle PING
    response = recv()
    if response and 'PING' in response:
        # Extract PING token
        import re
        match = re.search(r'PING\s+:(\S+)', response)
        if match:
            ping_token = match.group(1)
            send(f"PONG :{ping_token}")
            time.sleep(2)
    
    # Join channel
    send("JOIN #zeroclaw")
    time.sleep(2)
    
    # Send messages
    send("PRIVMSG #zeroclaw :hello zeroclaw")
    time.sleep(2)
    
    send("PRIVMSG #zeroclaw :can you see this?")
    time.sleep(2)
    
    send("PRIVMSG #zeroclaw :test message 123")
    time.sleep(2)
    
    send("QUIT")
    ssock.close()
    print("Connection closed")

if __name__ == '__main__':
    irc_client()
