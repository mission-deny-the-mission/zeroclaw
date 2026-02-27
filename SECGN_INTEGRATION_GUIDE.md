# ZeroClaw Hackerbot - SecGen Integration Guide

**Version**: 1.0.0  
**Date**: February 26, 2026  
**Status**: Phase 1 Complete - Ready for Testing

---

## 📋 Overview

This guide documents the integration of ZeroClaw with SecGen and Hacktivity, enabling ZeroClaw to replace the existing Ruby Hackerbot while maintaining full compatibility with existing scenarios.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SecGen Generator                         │
│  (Generates configs for BOTH Ruby and ZeroClaw bots)       │
├─────────────────────────────────────────────────────────────┤
│  Scenario XML → Generator → bot_X.xml (Ruby)               │
│                      │                                       │
│                      └→ bot_X.toml (ZeroClaw)              │
└─────────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┴──────────────────┐
         ↓                                      ↓
┌─────────────────┐                  ┌─────────────────┐
│  Ruby Hackerbot │                  │  ZeroClaw       │
│  (Legacy)       │                  │  (New)          │
│  /opt/hackerbot │                  │  /opt/zeroclaw  │
│  IRC: 6667      │                  │  IRC: 6668      │
└─────────────────┘                  └─────────────────┘
```

---

## 🎯 Integration Components

### 1. SecGen Generator (Ruby)

**Location**: `SecGen/lib/objects/`

#### Files Modified/Created:
- `local_hackerbot_config_generator.rb` - Updated to generate both Ruby XML and ZeroClaw TOML
- `zeroclaw_config_builder.rb` - New class for generating ZeroClaw configs
- `modules/generators/structured_content/hackerbot_config/generic/templates/zeroclaw_config.toml.erb` - ERB template

#### Usage:
The generator automatically creates both config formats when SecGen generates VMs:

```ruby
# In SecGen scenario generation
generator = HackerbotConfigGenerator.new
generator.accounts = [...]
generator.flags = [...]
generator.generate  # Outputs both xml_config and zeroclaw_toml_config
```

---

### 2. ZeroClaw SecGen Tools (Rust)

**Location**: `zeroclaw/src/tools/secgen/`

#### Tools Implemented:

1. **secgen_datastore_query** - Query SecGen datastore for randomized values
   - Access IP addresses, usernames, passwords, flags
   - Supports array indexing and field access
   
2. **secgen_flag_validator** - Validate CTF flags and track progress
   - Validates flags against SecGen datastore
   - Records student progress
   - Prevents duplicate submissions

#### Usage Examples:

Query datastore:
```json
{
  "name": "secgen_datastore_query",
  "arguments": {
    "key": "IP_addresses",
    "index": 0
  }
}
```

Validate flag:
```json
{
  "name": "secgen_flag_validator",
  "arguments": {
    "flag": "SEC GEN{abc123}",
    "scenario_id": "hacker_vs_hackerbot_1",
    "username": "student1"
  }
}
```

---

### 3. Puppet Module (SecGen Deployment)

**Location**: `SecGen/modules/utilities/unix/zeroclaw/`

#### Manifests:
- `init.pp` - Main class with parameters
- `install.pp` - Installs ZeroClaw binary, IRC server, Ollama
- `config.pp` - Deploys generated configs
- `service.pp` - Manages systemd service

#### Files:
- `files/zeroclaw.service` - Systemd unit file
- `files/ircd.conf` - InspIRCd configuration
- `templates/config.toml.erb` - Main config template

#### Usage in SecGen Scenarios:

Add to your scenario XML:

```xml
<utility module_path=".*/zeroclaw">
  <input into="irc_server_ip">
    <datastore access="2">IP_addresses</datastore>
  </input>
  <input into="hackerbot_configs">
    <datastore>hackerbot_instructions</datastore>
  </input>
  <input into="secgen_datastore_path">
    <value>/var/lib/secgen/datastore.json</value>
  </input>
</utility>
```

---

## 🚀 Deployment

### Step 1: Generate SecGen VM with ZeroClaw

```bash
# Generate VM with both Ruby and ZeroClaw bots
ruby secgen.rb run \
  --scenario scenarios/labs/response_and_investigation/hacker_vs_hackerbot_1.xml \
  --project zeroclaw_test_$(date +%Y%m%d)
```

### Step 2: Verify Installation

```bash
# Check both bots are installed
ls -la /opt/hackerbot/  # Ruby bot
ls -la /opt/zeroclaw/   # ZeroClaw

# Check services
systemctl status hackerbot  # Ruby bot service
systemctl status zeroclaw   # ZeroClaw service
systemctl status inspircd   # IRC server
```

### Step 3: Connect to Bots

```bash
# Connect to Ruby bot (port 6667)
irssi -c localhost -p 6667 -n student1
/join #hackerbot

# Connect to ZeroClaw (port 6668)
irssi -c localhost -p 6668 -n student1
/join #hackerbot
```

---

## 🧪 Testing Checklist

### For Each Scenario

#### Basic Functionality
- [ ] ZeroClaw service starts successfully
- [ ] IRC connection works on port 6668
- [ ] Bot responds to `hello` command
- [ ] Bot responds to `help` command
- [ ] Bot responds to `list` command

#### Scenario Navigation
- [ ] `goto N` navigates to correct scenario
- [ ] `next` moves to next scenario
- [ ] `previous` moves to previous scenario
- [ ] Scenario prompts display correctly

#### Flag Validation
- [ ] `secgen_flag_validator` tool accesses datastore
- [ ] Valid flags are accepted
- [ ] Invalid flags are rejected
- [ ] Duplicate submissions are prevented
- [ ] Progress is recorded correctly

#### Datastore Queries
- [ ] `secgen_datastore_query` returns correct IPs
- [ ] Account usernames/passwords accessible
- [ ] Field access works (e.g., `accounts[0].username`)

#### Personality System
- [ ] `personalities` lists all personalities
- [ ] `switch red_team` changes personality
- [ ] `switch blue_team` changes personality
- [ ] Different personalities have different responses

#### Security
- [ ] Shell allowlist enforced
- [ ] Shell denylist blocked
- [ ] Rate limiting works
- [ ] Workspace scoping enforced

---

## 📁 File Reference

### SecGen Side (Ruby)

```
SecGen/
├── lib/objects/
│   ├── local_hackerbot_config_generator.rb  (MODIFIED)
│   └── zeroclaw_config_builder.rb           (NEW)
├── modules/
│   └── generators/structured_content/hackerbot_config/
│       └── generic/
│           └── templates/
│               └── zeroclaw_config.toml.erb  (NEW)
└── modules/utilities/unix/zeroclaw/         (NEW)
    ├── manifests/
    │   ├── init.pp
    │   ├── install.pp
    │   ├── config.pp
    │   └── service.pp
    ├── files/
    │   ├── zeroclaw.service
    │   └── ircd.conf
    └── templates/
        └── config.toml.erb
```

### ZeroClaw Side (Rust)

```
zeroclaw/
└── src/tools/
    ├── secgen/                              (NEW)
    │   ├── mod.rs
    │   ├── secgen_datastore_query.rs
    │   └── secgen_flag_validator.rs
    └── mod.rs                               (MODIFIED)
```

---

## 🔧 Configuration Reference

### ZeroClaw TOML Config (Generated)

```toml
[hackerbot]
scenario_id = "hacker_vs_hackerbot_1"
irc_server = "localhost"
irc_port = 6668
default_personality = "red_team"

[secgen]
enabled = true
datastore_path = "/var/lib/secgen/datastore.json"
main_username = "student1"
main_password = "password123"

[[flags]]
id = "flag_1"
value = "SEC GEN{abc123}"
description = "Flag 1"
points = 100

[[scenarios]]
index = 0
title = "Initial Reconnaissance"
prompt = "Begin by gathering information..."
flag_id = "flag_1"
```

### SecGen Datastore Format

```json
{
  "IP_addresses": ["172.16.0.2", "172.16.0.3"],
  "accounts": [
    {"username": "student1", "password": "pass1"},
    {"username": "student2", "password": "pass2"}
  ],
  "flags": [
    "SEC GEN{flag1}",
    "SEC GEN{flag2}"
  ]
}
```

---

## ⚠️ Troubleshooting

### ZeroClaw Service Won't Start

```bash
# Check logs
journalctl -u zeroclaw -f

# Common issues:
# 1. Config file syntax error
# 2. Port 6668 already in use
# 3. Datastore path not accessible
```

### Can't Connect to IRC

```bash
# Check IRC server
systemctl status inspircd
netstat -tlnp | grep 6668

# Test connection
nc localhost 6668
```

### Datastore Query Fails

```bash
# Check datastore exists
ls -la /var/lib/secgen/datastore.json

# Check permissions
cat /var/lib/secgen/datastore.json

# Validate JSON
jq . /var/lib/secgen/datastore.json
```

### Flag Validation Fails

```bash
# Check flags in datastore
jq '.flags' /var/lib/secgen/datastore.json

# Check progress file
cat /var/lib/zeroclaw/progress.json

# Verify flag format (case-sensitive)
```

---

## 📊 Migration Status

### Phase 1: Foundation ✅ COMPLETE

- [x] SecGen generator module updated
- [x] ZeroClaw config builder created
- [x] ERB template for TOML configs
- [x] secgen_datastore_query tool
- [x] secgen_flag_validator tool
- [x] Puppet module for deployment

### Phase 2: Integration 🔄 IN PROGRESS

- [ ] cybersecurity-secgen skill
- [ ] lab sheet generator tool
- [ ] IRC compatibility testing

### Phase 3: Scenario Migration ⏳ PENDING

- [ ] hacker_vs_hackerbot_1
- [ ] hacker_vs_hackerbot_2
- [ ] response_and_investigation/* (9 scenarios)
- [ ] systems_security/* (8 scenarios)
- [ ] Remaining scenarios (26+ total)

---

## 🎯 Next Steps

1. **Build ZeroClaw binary** with new SecGen tools
2. **Test on SecGen VM** with hacker_vs_hackerbot_1 scenario
3. **Validate all commands** work identically to Ruby bot
4. **Document any discrepancies** and fix
5. **Proceed to Phase 2** (lab sheet generator)

---

## 📞 Support

- **GitHub Issues**: [ZeroClaw Issues](https://github.com/openagen/zeroclaw/issues)
- **Documentation**: [ZeroClaw Docs](https://github.com/openagen/zeroclaw/tree/main/docs)
- **SecGen Docs**: [SecGen README](https://github.com/cliffe/SecGen/blob/master/README.md)

---

**Last Updated**: February 26, 2026  
**Maintained By**: ZeroClaw Development Team
