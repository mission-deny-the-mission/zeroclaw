# Cybersecurity Training Skill

## Overview

This skill transforms ZeroClaw into a comprehensive cybersecurity training bot capable of:

- **Interactive Attack Scenarios**: Progressive hands-on exercises
- **Multi-Personality Support**: Switch between Red Team, Blue Team, Researcher, and Instructor personas
- **Tool Demonstrations**: Safe execution of security tools (nmap, tcpdump, iptables, ssh)
- **Quiz System**: Answer validation with contextual feedback
- **Knowledge Retrieval**: MITRE ATT&CK integration, man pages, and custom documentation
- **IRC Communication**: Classic IRC-based interface with modern AI capabilities

## Installation

```bash
# The skill will be auto-loaded from ~/.zeroclaw/workspace/skills/cybersecurity/
# Or install via:
zeroclaw skills install ./hackerbot_skills/cybersecurity
```

## Usage

### Basic Commands

```
hello                  - Start conversation and get first scenario
list                   - Show all available scenarios
goto <number>          - Jump to specific scenario
next                   - Move to next scenario
previous               - Go back to previous scenario
ready                  - Execute demonstration for current scenario
answer <text>          - Submit quiz answer
```

### Personality Commands

```
personalities          - List available personalities
switch <name>          - Change personality (red_team, blue_team, researcher, instructor)
personality            - Show current personality
```

### Help Commands

```
help                   - Show available commands
clear_history          - Clear conversation history
show_history           - View conversation history
```

## Available Scenarios

1. **Network Reconnaissance** - Learn nmap scanning techniques
2. **Traffic Analysis** - Capture and analyze packets with tcpdump
3. **Firewall Configuration** - Create iptables rules
4. **Secure Communication** - SSH hardening best practices

## Personalities

### Red Team Specialist
Offensive security expert focusing on penetration testing and attack simulation.

### Blue Team Defender
Defensive security expert specializing in threat detection and incident response.

### Security Researcher
Academic researcher focused on vulnerability analysis and threat intelligence.

### Cybersecurity Instructor
Professional trainer for certification preparation and skill development.

## Configuration

Add to your `~/.zeroclaw/config.toml`:

```toml
[channels_config.irc]
server = "localhost"
port = 6667
nickname = "Hackerbot"
channels = ["#hackerbot"]
allowed_users = ["*"]

[agents.red_team]
provider = "ollama"
model = "gemma3:1b"
system_prompt = "You are an offensive security expert..."
agentic = true
allowed_tools = ["shell", "scenario_manager"]

[agents.blue_team]
provider = "ollama"
model = "gemma3:1b"
system_prompt = "You are a defensive security expert..."
agentic = true
allowed_tools = ["shell", "scenario_manager"]
```

## Security

This skill operates in supervised autonomy mode:
- Shell commands require explicit approval for medium/high-risk operations
- Command allowlist restricts execution to approved security tools
- Rate limiting prevents abuse
- All actions are logged for audit purposes

## Examples

### Starting a Session

```
User: hello
Hackerbot: Welcome to Red Team operations! I'm your offensive security specialist.
           Let's start with fundamental security concepts...
```

### Navigating Scenarios

```
User: list
Hackerbot: Available scenarios:
             1. Network Reconnaissance with nmap
             2. Traffic Analysis with tcpdump
             3. Firewall Configuration with iptables
             4. Secure Communication with SSH

User: goto 1
Hackerbot: Jumping to scenario 1: Network Reconnaissance...
```

### Executing Demonstrations

```
User: ready
Hackerbot: Setting up the demonstration environment...
           [Executes: nmap -sS -p 1-1000 target.example.com]
           FYI: Starting Nmap 7.94...
```

### Answering Quizzes

```
User: answer -sS
Hackerbot: Excellent! That's the correct answer. The -sS option performs a TCP SYN scan...
```

### Switching Personalities

```
User: personalities
Hackerbot: Available personalities:
             - red_team (Red Team Specialist) [CURRENT]
             - blue_team (Blue Team Defender)
             - researcher (Security Researcher)
             - instructor (Cybersecurity Instructor)

User: switch blue_team
Hackerbot: Switched to blue_team personality (Blue Team Defender)
           Welcome to defensive operations!
```

## License

MIT License - See LICENSE file for details

## Contributing

Contributions welcome! Please read CONTRIBUTING.md before submitting PRs.
