# ZeroClaw Hackerbot - Quick Start Guide

**Get your cybersecurity training bot running in 5 minutes!**

## Prerequisites Checklist

- [ ] Linux/macOS/WSL environment
- [ ] Git installed
- [ ] Node.js 14+ installed
- [ ] ZeroClaw repository cloned

## Step 1: Install Dependencies (2 minutes)

### Install Ollama (Local LLM)

```bash
# Download and install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull required models
ollama pull gemma3:1b
ollama pull nomic-embed-text

# Start Ollama service (runs in background)
ollama serve
```

### Verify Installation

```bash
# Check Ollama is running
ollama list

# Should show:
# NAME              ID           SIZE
# gemma3:1b         xxx          1.2 GB
# nomic-embed-text  xxx          274 MB
```

## Step 2: Setup Hackerbot (1 minute)

```bash
# Navigate to ZeroClaw directory
cd /path/to/zeroclaw

# Copy configuration
cp config/hackerbot.config.toml ~/.zeroclaw/config.toml

# Create workspace directory
mkdir -p ~/.zeroclaw/workspace/skills
ln -s /path/to/zeroclaw/hackerbot_skills/cybersecurity ~/.zeroclaw/workspace/skills/
```

## Step 3: Start IRC Server (Optional, 30 seconds)

If you want to test with IRC:

```bash
# Start simple IRC server (ZeroClaw includes one)
cd /path/to/zeroclaw
python3 simple_irc_server.py &

# Or use your own IRC server
# Update config.toml with your server details
```

## Step 4: Start Hackerbot (30 seconds)

```bash
# Start the bot
zeroclaw channel start

# You should see:
# [INFO] Config loaded from ~/.zeroclaw/config.toml
# [INFO] Starting IRC channel...
# [INFO] Connected to localhost:6667
# [INFO] Joined #hackerbot
# [INFO] Hackerbot ready!
```

## Step 5: Connect and Test (1 minute)

### Option A: IRC Client

```bash
# Install IRC client if needed
sudo apt install irssi    # Debian/Ubuntu
sudo dnf install irssi    # Fedora/RHEL
brew install irssi        # macOS

# Connect to IRC
irssi -c localhost -p 6667 -n testuser

# Join channel
/join #hackerbot

# Test commands
hello
list
```

### Option B: Web IRC Client

1. Open https://kiwiirc.com/ in your browser
2. Network: `localhost`, Port: `6667`
3. Nickname: `testuser`
4. Channel: `#hackerbot`
5. Click "Start"

### Test Commands

```
hello                          # Start conversation
list                           # Show scenarios
goto 1                         # Jump to scenario 1
ready                          # Execute demonstration
answer -sS                     # Submit quiz answer
personalities                  # List personalities
switch blue_team               # Change personality
help                           # Show help
```

## Expected Output

### When you type `hello`:

```
<Hackerbot> Welcome to Red Team operations! I'm your offensive security 
            specialist. Let's start with fundamental security concepts.

<Hackerbot> Current scenario: Network Reconnaissance with nmap

<Hackerbot> Let's explore network reconnaissance using nmap. What are the 
            common nmap scanning techniques, and how would you use nmap to 
            discover open ports and services on a target network?

<Hackerbot> When you're ready to proceed, just say 'ready'.
```

### When you type `list`:

```
<Hackerbot> Available cybersecurity training scenarios:

              1. Network Reconnaissance with nmap [CURRENT]
              2. Traffic Analysis with tcpdump
              3. Firewall Configuration with iptables
              4. Secure Communication with SSH

            Progress: 0/4 completed
```

### When you type `answer -sS`:

```
<Hackerbot> ✓ Correct!

<Hackerbot> Excellent! The -sS option performs a TCP SYN scan (half-open 
            scan), which doesn't complete the TCP three-way handshake, 
            making it stealthier than a full connect scan.

<Hackerbot> Explanation: TCP SYN scans send only the initial SYN packet 
            and wait for a response. If a SYN-ACK is received, the port 
            is open. If RST is received, the port is closed.

<Hackerbot> Scenario completed! Use "next" to continue.
```

### When you type `switch blue_team`:

```
<Hackerbot> Switched to blue_team personality (Blue Team Defender)

<Hackerbot> Welcome to defensive operations! I'm your blue team security 
            specialist. Let's discuss threat detection, incident response, 
            and defensive strategies.
```

## Troubleshooting

### "Connection refused" when starting

**Problem**: IRC server not running or wrong port

**Solution**:
```bash
# Check if IRC server is running
netstat -tlnp | grep 6667

# Start IRC server
python3 simple_irc_server.py &

# Or update config.toml with correct server
```

### "Command not found: zeroclaw"

**Problem**: ZeroClaw not installed or not in PATH

**Solution**:
```bash
# Build ZeroClaw
cd /path/to/zeroclaw
cargo build --release

# Add to PATH
export PATH="/path/to/zeroclaw/target/release:$PATH"

# Or install system-wide
cargo install --path .
```

### Ollama connection errors

**Problem**: Ollama service not running

**Solution**:
```bash
# Start Ollama
ollama serve

# Check it's running
curl http://localhost:11434/api/tags

# Should return list of models
```

### Quiz answers not recognized

**Problem**: Answer format mismatch

**Solution**:
- Try exact answer from quiz
- Use lowercase
- Include dashes if shown (e.g., `-sS` not `sS`)
- Ask for hint if stuck

## Next Steps

### Explore More Features

```
# Try different personalities
switch red_team
switch researcher
switch instructor

# Navigate scenarios
next
previous
goto 2

# Check progress
progress

# Get help
help
```

### Configure Additional Channels

Edit `~/.zeroclaw/config.toml`:

```toml
# Add Telegram
[channels_config.telegram]
bot_token = "YOUR_BOT_TOKEN"
allowed_users = ["@yourusername"]

# Add Discord
[channels_config.discord]
bot_token = "YOUR_BOT_TOKEN"
guild_id = "YOUR_GUILD_ID"
channel_id = "YOUR_CHANNEL_ID"
```

### Create Custom Scenarios

Edit `hackerbot_skills/cybersecurity/tools/scenario_manager.js`:

```javascript
const DEFAULT_SCENARIOS = [
  // ... existing scenarios ...
  {
    index: 4,
    title: "Your Custom Scenario",
    prompt: "Your scenario description...",
    quiz: {
      question: "Your question?",
      answer: "Correct answer",
      accepted_variants: ["variant1", "variant2"]
    }
  }
];
```

## Learning Resources

### Documentation
- **[HACKERBOT_README.md](HACKERBOT_README.md)** - Complete user guide
- **[config/hackerbot.config.toml](config/hackerbot.config.toml)** - Configuration reference
- **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Technical details

### Cybersecurity Topics Covered
1. **Network Reconnaissance** - nmap scanning techniques
2. **Traffic Analysis** - tcpdump packet capture
3. **Firewall Configuration** - iptables rules
4. **Secure Communication** - SSH hardening

### Certification Alignment
- **CompTIA Security+**: Network security, cryptography
- **CEH**: Reconnaissance, scanning, enumeration
- **OSCP**: Methodology, tool usage
- **CISSP**: Security architecture, operations

## Getting Help

### Common Issues
- **Bot not responding**: Check IRC connection, verify Ollama running
- **Slow responses**: Try smaller model or increase temperature
- **Commands not working**: Check allowed_users in config
- **Quiz failing**: Review scenario learning points

### Support Channels
- **GitHub Issues**: [Report bugs](https://github.com/openagen/zeroclaw/issues)
- **Discord**: [ZeroClaw Community](https://discord.gg/zeroclaw)
- **Telegram**: [@zeroclawlabs](https://t.me/zeroclawlabs)

## Quick Reference Card

```
┌─────────────────────────────────────────────────────┐
│           ZeroClaw Hackerbot Quick Reference        │
├─────────────────────────────────────────────────────┤
│ hello          Start conversation                   │
│ list           Show all scenarios                   │
│ goto <n>       Jump to scenario n                   │
│ next           Next scenario                        │
│ previous       Previous scenario                    │
│ ready          Execute demonstration                │
│ answer <text>  Submit quiz answer                   │
│ progress       Show your progress                   │
│ personalities  List personalities                   │
│ switch <name>  Change personality                   │
│ personality    Show current personality             │
│ help           Show help                            │
│ clear_history  Clear conversation history           │
└─────────────────────────────────────────────────────┘
```

---

**Congratulations! You're ready to start cybersecurity training with ZeroClaw Hackerbot! 🎉**

Happy learning! 🚀
