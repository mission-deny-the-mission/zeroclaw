#!/usr/bin/env python3
"""
Simple IRC Server for ZeroClaw Testing
A minimal IRC server implementation for local testing.
"""

import socket
import select
import threading
import time
from datetime import datetime

class SimpleIRCServer:
    def __init__(self, host='127.0.0.1', port=6668):
        self.host = host
        self.port = port
        self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.clients = {}
        self.channels = {}
        self.running = False
        
    def start(self):
        self.server_socket.bind((self.host, self.port))
        self.server_socket.listen(5)
        self.server_socket.setblocking(0)
        self.running = True
        
        print(f"[*] Simple IRC Server started on {self.host}:{self.port}")
        print(f"[*] Waiting for connections...")
        
        while self.running:
            readable = [self.server_socket] + list(self.clients.keys())
            try:
                ready = select.select(readable, [], [], 1.0)
            except:
                continue
                
            for sock in ready[0]:
                if sock is self.server_socket:
                    client, addr = self.server_socket.accept()
                    self.clients[client] = {'nick': None, 'user': None, 'addr': addr}
                    print(f"[*] New connection from {addr}")
                else:
                    try:
                        data = sock.recv(4096)
                        if data:
                            self.handle_message(sock, data.decode('utf-8', errors='ignore'))
                    except:
                        self.remove_client(sock)
                        
    def remove_client(self, sock):
        if sock in self.clients:
            client_info = self.clients[sock]
            if client_info['nick']:
                self.broadcast(f":{client_info['nick']} QUIT :Connection closed")
            del self.clients[sock]
            sock.close()
            
    def handle_message(self, sock, message):
        lines = message.strip().split('\r\n')
        for line in lines:
            print(f"< {line}")
            parts = line.split(' ', 3)
            cmd = parts[0].upper()
            
            if cmd == 'NICK':
                nick = parts[1]
                self.clients[sock]['nick'] = nick
                self.send(sock, f":localhost 001 {nick} :Welcome to Simple IRC Server!")
                self.send(sock, f":localhost 002 {nick} :Your host is simpleirc")
                self.send(sock, f":localhost 003 {nick} :This server was created {datetime.now()}")
                
            elif cmd == 'USER':
                self.clients[sock]['user'] = parts[1]
                
            elif cmd == 'JOIN':
                channel = parts[1]
                if channel not in self.channels:
                    self.channels[channel] = []
                self.channels[channel].append(sock)
                nick = self.clients[sock]['nick']
                self.broadcast(f":{nick} JOIN {channel}", channel)
                self.send(sock, f":localhost 353 {nick} = {channel} :{' '.join([self.clients[c]['nick'] for c in self.channels[channel] if self.clients[c]['nick']])}")
                self.send(sock, f":localhost 366 {nick} {channel} :End of /NAMES list")
                
            elif cmd == 'PRIVMSG':
                target = parts[1]
                message = parts[3] if len(parts) > 3 else ''
                nick = self.clients[sock]['nick']
                if target.startswith('#'):
                    self.broadcast(f":{nick} PRIVMSG {target} :{message}", target)
                else:
                    self.send_to_nick(target, f":{nick} PRIVMSG {target} :{message}")
                    
            elif cmd == 'PART':
                channel = parts[1]
                nick = self.clients[sock]['nick']
                if channel in self.channels and sock in self.channels[channel]:
                    self.channels[channel].remove(sock)
                    self.broadcast(f":{nick} PART {channel}", channel)
                    
            elif cmd == 'QUIT':
                self.remove_client(sock)
                
            elif cmd == 'PING':
                server = parts[1] if len(parts) > 1 else 'localhost'
                self.send(sock, f":localhost PONG localhost :{server}")
                
            else:
                # Unknown command - just acknowledge
                pass
                
    def send(self, sock, message):
        try:
            sock.send((message + '\r\n').encode('utf-8'))
            print(f"> {message}")
        except:
            self.remove_client(sock)
            
    def send_to_nick(self, nick, message):
        for sock, info in self.clients.items():
            if info['nick'] == nick:
                self.send(sock, message)
                return
                
    def broadcast(self, message, channel=None):
        if channel and channel in self.channels:
            for sock in self.channels[channel]:
                self.send(sock, message)
        elif not channel:
            for sock in list(self.clients.keys()):
                self.send(sock, message)
                
    def stop(self):
        self.running = False
        for sock in list(self.clients.keys()):
            sock.close()
        self.server_socket.close()
        print("[*] Server stopped")

if __name__ == '__main__':
    import sys
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 6668
    server = SimpleIRCServer(port=port)
    try:
        server.start()
    except KeyboardInterrupt:
        print("\n[*] Shutting down...")
        server.stop()
