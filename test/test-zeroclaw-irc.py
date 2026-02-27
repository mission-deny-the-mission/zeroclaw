#!/usr/bin/env python3
"""Interactive IRC test client for ZeroClaw"""
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
    
    received = []
    
    def recv_loop():
        ssock.settimeout(30)
        try:
            while True:
                data = ssock.recv(4096).decode()
                if not data:
                    break
                print(f"< {data}")
                received.append(data)
                
                # Handle PING
                if 'PING' in data:
                    import re
                    match = re.search(r'PING\s+:(\S+)', data)
                    if match:
                        ping_token = match.group(1)
                        send(f"PONG :{ping_token}")
        except:
            pass
    
    def send(cmd):
        ssock.send(f"{cmd}\r\n".encode())
        print(f"> {cmd}")
    
    # Start receiver thread
    recv_thread = threading.Thread(target=recv_loop, daemon=True)
    recv_thread.start()
    
    # Register
    send("NICK testuser")
    send("USER test 0 * :Test User")
    time.sleep(3)
    
    # Join channel
    send("JOIN #zeroclaw")
    time.sleep(3)
    
    # Send test messages
    print("\n=== Sending 'hello' ===")
    send("PRIVMSG #zeroclaw :hello")
    time.sleep(10)
    
    print("\n=== Sending 'list' ===")
    send("PRIVMSG #zeroclaw :list")
    time.sleep(10)
    
    print("\n=== Sending 'help' ===")
    send("PRIVMSG #zeroclaw :help")
    time.sleep(5)
    
    send("QUIT")
    ssock.close()
    print("\nConnection closed")

if __name__ == '__main__':
    irc_client()
