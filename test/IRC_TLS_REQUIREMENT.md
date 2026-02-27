# ZeroClaw IRC TLS Requirement

**Date**: February 27, 2026  
**Status**: ✅ **RESOLVED**

---

## 🔍 Issue Discovery

### Problem
ZeroClaw's IRC client was failing to connect to IRC servers with error:
```
Channel irc error: Connection reset by peer (os error 104)
```

### Root Cause
ZeroClaw's IRC client implementation in `src/channels/irc.rs` **hardcodes TLS** for all IRC connections (lines 274-295):

```rust
/// Create a TLS connection to the IRC server.
async fn connect(
    &self,
) -> anyhow::Result<tokio_rustls::client::TlsStream<tokio::net::TcpStream>> {
    let addr = format!("{}:{}", self.server, self.port);
    let tcp = tokio::net::TcpStream::connect(&addr).await?;
    
    // ... TLS configuration ...
    
    let connector = tokio_rustls::TlsConnector::from(Arc::new(tls_config));
    let domain = rustls::pki_types::ServerName::try_from(self.server.clone())?;
    let tls = connector.connect(domain, tcp).await?;  // ← Always uses TLS
    
    Ok(tls)
}
```

**Key Finding**: The `connect()` function **always** wraps the TCP connection in TLS, regardless of configuration. The `verify_tls` option only controls certificate verification, not whether TLS is used.

---

## ✅ Solution

### Configuration Change
Use IRC server with TLS enabled and configure ZeroClaw to use the TLS port:

```toml
[channels_config.irc]
server = "127.0.0.1"
port = 6697  # TLS port (not 6667)
verify_tls = false  # Skip cert verification for local testing
```

### Docker IRC Server Setup
```bash
docker run -d \
  --name zeroclaw-irc \
  --restart unless-stopped \
  -p 6667:6667 \  # Non-TLS for standard clients
  -p 6697:6697 \  # TLS for ZeroClaw
  inspircd/inspircd-docker:latest
```

---

## 📊 Testing Results

### Before (Non-TLS Port 6667)
```
❌ Channel irc error: Connection reset by peer (os error 104)
❌ Connection immediately rejected
```

### After (TLS Port 6697)
```
✅ IRC channel connecting to 127.0.0.1:6697 as ZeroClawTest...
✅ IRC registered as ZeroClawTest
✅ Connection stable
```

---

## 🔧 SecGen Integration Notes

### SecGen's Hackerbot
- Uses **Ruby Cinch** IRC library
- Supports both TLS and non-TLS connections
- Default: non-TLS on port 6667

### ZeroClaw Hackerbot
- Uses **custom Rust IRC client** (IRCinch-inspired)
- **Requires TLS** (hardcoded)
- Default: TLS on port 6697

### Compatibility
- ✅ Both can coexist on same IRC server
- ✅ Different ports (6667 for SecGen, 6697 for ZeroClaw)
- ✅ Same channels accessible from both

---

## 📝 Configuration Examples

### Local Testing (Docker)
```toml
[channels_config.irc]
server = "127.0.0.1"
port = 6697
nickname = "ZeroClawTest"
channels = ["#zeroclaw"]
verify_tls = false
```

### Production (SecGen VM)
```toml
[channels_config.irc]
server = "localhost"
port = 6697
nickname = "Hackerbot"
channels = ["#hackerbot"]
verify_tls = true  # Verify certificates in production
```

### Production (External IRC Server)
```toml
[channels_config.irc]
server = "irc.example.com"
port = 6697
nickname = "Hackerbot"
channels = ["#hackerbot"]
verify_tls = true
```

---

## 🚨 Important Notes

1. **TLS is Mandatory**: ZeroClaw cannot connect to non-TLS IRC servers without code changes

2. **Certificate Verification**: 
   - `verify_tls = false` for local testing (accepts self-signed certs)
   - `verify_tls = true` for production (validates certificates)

3. **Port Configuration**:
   - Standard IRC: 6667 (non-TLS)
   - IRC with TLS: 6697 (ZeroClaw requires this)

4. **SecGen Compatibility**: Both SecGen Hackerbot and ZeroClaw can use the same IRC server on different ports

---

## 🔮 Future Improvements

### Optional TLS Support
Modify `src/channels/irc.rs` to support non-TLS connections:

```rust
pub struct IrcChannelConfig {
    // ... existing fields ...
    pub use_tls: bool,  // Add this field
}

async fn connect(&self) -> anyhow::Result<IrcConnection> {
    if self.use_tls {
        // Current TLS code
    } else {
        // Plain TCP connection
        Ok(tokio::net::TcpStream::connect(&addr).await?)
    }
}
```

### Benefits
- Support for legacy IRC servers
- Simpler local testing
- Reduced resource usage (no TLS overhead)

---

## 📞 References

- **Code**: `zeroclaw/src/channels/irc.rs` (lines 274-295)
- **Config**: `zeroclaw/test/config.toml`
- **Docker Image**: `inspircd/inspircd-docker:latest`
- **Test Results**: `zeroclaw/test/TESTING_SETUP.md`

---

**Last Updated**: February 27, 2026  
**Status**: ✅ Resolved - TLS configuration documented and working
