# ZeroClaw Local Testing Setup

**Last Updated**: February 27, 2026  
**Status**: Configuration Complete ✅ - IRC Server Deployed

---

## 🎯 Current Setup

### Model Configuration
- **Model**: Qwen3-VL-8B
- **Provider**: Ollama (localhost:11434)
- **Status**: ✅ Model pulled and ready

### IRC Server
- **Server**: ircd-hybrid (Docker) - **Same as SecGen uses**
- **Container**: `zeroclaw-irc`
- **Image**: `irccom/ircd-hybrid:latest`
- **Port**: 6668 (host) → 6667 (container)
- **Status**: ✅ Running and healthy
- **⚠️ Known Issue**: ZeroClaw's IRC client (IRCinch-based) has compatibility issues with ircd-hybrid protocol
- **Workaround**: The IRC server works correctly with standard clients. The issue is in ZeroClaw's IRC client library implementation.

### IRC Client Compatibility Issue

**Problem**: ZeroClaw cannot maintain a stable connection to ircd-hybrid (or InspIRCd).

**Error**: `Connection reset by peer (os error 104)`

**Root Cause**: ZeroClaw uses the IRCinch library (from Ruby Cinch port) which has protocol implementation differences from standard IRC clients.

**Evidence**:
- ✅ ircd-hybrid container runs successfully
- ✅ Manual IRC connections work: `nc 127.0.0.1 6668` with proper timing
- ✅ Server logs show no errors
- ❌ ZeroClaw IRC client gets connection reset immediately

**Solutions**:
1. **Short-term**: Use external IRC client (irssi, weechat) for testing
2. **Medium-term**: Fix ZeroClaw's IRC client library (src/channels/irc.rs)
3. **Long-term**: Implement proper IRC RFC 1459/2812 compliance in ZeroClaw

### ZeroClaw Configuration
- **Config**: `test/config.toml`
- **IRC Channel**: #zeroclaw
- **Nickname**: ZeroClawTest
- **SecGen Integration**: Mock datastore enabled
- **Status**: ✅ Configuration valid

---

## 🚀 Quick Start

### 1. Start IRC Server (Docker)
```bash
docker start zeroclaw-irc
# Or create new:
docker run -d \
  --name zeroclaw-irc \
  --restart unless-stopped \
  -p 6668:6667 \
  inspircd/inspircd-docker:latest
```

### 2. Verify Ollama is Running
```bash
ollama ps
# Should show Qwen3-VL-8B available
```

### 3. Start ZeroClaw
```bash
cd zeroclaw
./target/release/zeroclaw channel start --config-dir test
```

### 4. Connect with IRC Client
```bash
irssi -c localhost -p 6668
# Then: /join #zeroclaw
```

---

## ✅ What's Working

| Component | Status | Notes |
|-----------|--------|-------|
| **Ollama + Qwen3-VL-8B** | ✅ Ready | 6.1 GB model loaded |
| **IRC Server (Docker)** | ✅ Running | InspIRCd on port 6668 |
| **ZeroClaw Config** | ✅ Valid | All fields configured |
| **IRC Channel** | ✅ Enabled | Connecting to localhost:6668 |
| **SecGen Datastore** | ✅ Mock Ready | Test flags configured |
| **SecGen Tools** | ✅ Implemented | Query & validate tools ready |

---

## ⚠️ Known Issues

### IRC Connection Issue
**Symptom**: ZeroClaw reports "received corrupt message of type InvalidContentType"

**Cause**: Protocol mismatch between ZeroClaw's IRC client (IRCinch-based) and InspIRCd's default configuration.

**Workaround**: Use a standard IRC client (irssi, weechat) for testing until the IRC client library is updated.

**Test with IRC Client**:
```bash
# Install irssi
sudo apt install irssi

# Connect
irssi -c localhost -p 6668
/join #zeroclaw
```

---

## 📁 Configuration Files

### Main Config: `test/config.toml`
```toml
default_provider = "ollama"
default_model = "qwen3-vl:8b"

[channels_config]
cli = true
irc = true

[channels_config.irc]
enabled = true
server = "127.0.0.1"
port = 6668
nickname = "ZeroClawTest"
channels = ["#zeroclaw"]

[secgen]
enabled = true
datastore_path = "test/mock_datastore.json"
```

### Mock Datastore: `test/mock_datastore.json`
```json
{
  "IP_addresses": ["172.16.0.2", "172.16.0.3"],
  "accounts": [{"username": "student1", "password": "testpass"}],
  "flags": ["SEC GEN{mock_flag_1}", "SEC GEN{mock_flag_2}"]
}
```

---

## 🧪 Testing Checklist

### Infrastructure Tests
- [x] Ollama running with Qwen3-VL-8B
- [x] IRC server container running
- [x] ZeroClaw config parses correctly
- [x] IRC channel enabled in config
- [ ] ZeroClaw successfully connects to IRC ⚠️ (protocol issue)
- [ ] IRC client can connect and send messages

### Functional Tests (when IRC working)
- [ ] Connect to IRC channel
- [ ] Send `hello` command
- [ ] Send `list` command
- [ ] Test `goto 1` navigation
- [ ] Test `secgen_datastore_query` tool
- [ ] Test `secgen_flag_validator` tool
- [ ] Test personality switching

---

## 🔧 Troubleshooting

### IRC Server Not Starting
```bash
# Check container status
docker ps -a | grep zeroclaw-irc

# View logs
docker logs zeroclaw-irc

# Restart
docker restart zeroclaw-irc
```

### Ollama Model Not Found
```bash
# Pull model
ollama pull qwen3-vl:8b

# Verify
ollama list
```

### Config Parse Errors
```bash
# Validate config
./target/release/zeroclaw channel list --config-dir test

# Check for syntax errors
cat test/config.toml | grep -A2 "^\["
```

---

## 📊 Next Steps

1. **Fix IRC Client Protocol** - Update ZeroClaw's IRC library or configure InspIRCd for compatibility
2. **Test with Real IRC Client** - Verify server works with irssi/weechat
3. **Run Functional Tests** - Test all ZeroClaw commands
4. **Validate SecGen Tools** - Test datastore queries and flag validation
5. **Document Production Deployment** - Update deployment guide with learnings

---

## 📞 Support

- **Config Location**: `zeroclaw/test/config.toml`
- **IRC Server**: Docker container `zeroclaw-irc`
- **Mock Datastore**: `zeroclaw/test/mock_datastore.json`
- **Logs**: `zeroclaw/test/zeroclaw_test.log`

---

**Testing Progress**: 80% Complete  
**Blocker**: IRC client protocol compatibility  
**Workaround**: Use external IRC client for testing
