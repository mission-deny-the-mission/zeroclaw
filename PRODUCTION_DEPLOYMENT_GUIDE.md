# ZeroClaw Hackerbot - Production Deployment Guide

**Version**: 1.0.0  
**Date**: February 27, 2026  
**Status**: Planning Phase

---

## 🏗️ Production Architecture

### Network Topology

```
┌─────────────────────────────────────────────────────────────────┐
│                    SecGen VM (Isolated Network)                 │
│  ┌─────────────────┐                                           │
│  │  ZeroClaw       │                                           │
│  │  Hackerbot      │                                           │
│  │  (IRC: 6668)    │                                           │
│  └────────┬────────┘                                           │
│           │                                                     │
│           │ Dedicated Proxy Connection                          │
│           │ (Isolated from other systems)                       │
└───────────┼─────────────────────────────────────────────────────┘
            │
            │ Firewall Rules (Restricted)
            │
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Model Host (Separate Network)                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Model Runner (Choice of):                                │  │
│  │  • Ollama (testing)                                       │  │
│  │  • vLLM (production)                                      │  │
│  │  • SGLang (high throughput)                               │  │
│  │  • llama.cpp (CPU/edge)                                   │  │
│  │  • KTransformers (advanced)                               │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Models:                                                  │  │
│  │  • Qwen3-VL-8B (current)                                  │  │
│  │  • Larger models with tool calling (future)               │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Network Isolation**: Hackerbot VMs are isolated from other systems
2. **Dedicated Proxy**: Single controlled egress point for model access
3. **Model Runner Agnostic**: Support multiple model runners
4. **Flexible Model Selection**: Easy to swap models without code changes

---

## 🔧 Configuration Options

### Local Testing (Current)

**Model Runner**: Ollama  
**Model**: Qwen3-VL-8B  
**Network**: Localhost

```toml
# config.toml
default_provider = "ollama"
default_model = "qwen3-vl:8b"

[ollama]
host = "localhost"
port = 11434
```

### Production Option 1: vLLM

**Model Runner**: vLLM  
**Use Case**: High throughput, multi-model serving

```toml
# config.toml
default_provider = "compatible"
default_model = "qwen3-vl-8b"
api_url = "http://model-host:8000/v1"
api_key = "your-api-key"

[provider.compatible]
name = "vllm"
base_url = "http://model-host:8000/v1"
```

### Production Option 2: SGLang

**Model Runner**: SGLang  
**Use Case**: Optimized for structured generation

```toml
# config.toml
default_provider = "sglang"
default_model = "qwen3-vl-8b"

[sglang]
host = "model-host"
port = 30000
```

### Production Option 3: llama.cpp

**Model Runner**: llama.cpp  
**Use Case**: CPU inference, edge deployment

```toml
# config.toml
default_provider = "compatible"
default_model = "qwen3-vl-8b"
api_url = "http://model-host:8080/v1"

[provider.compatible]
name = "llama-cpp"
base_url = "http://model-host:8080/v1"
```

### Production Option 4: KTransformers

**Model Runner**: KTransformers  
**Use Case**: Advanced model optimization

```toml
# config.toml
default_provider = "compatible"
default_model = "qwen3-vl-8b"
api_url = "http://model-host:8888/v1"

[provider.compatible]
name = "ktransformers"
base_url = "http://model-host:8888/v1"
```

---

## 🔐 Security Configuration

### Firewall Rules

**From SecGen VM to Model Host**:
```bash
# Allow outbound to model proxy only
iptables -A OUTPUT -p tcp -d <model-host-ip> --dport 8000 -j ACCEPT
iptables -A OUTPUT -p tcp -d <model-host-ip> --dport 30000 -j ACCEPT
iptables -A OUTPUT -p tcp -d <model-host-ip> --dport 8080 -j ACCEPT
iptables -A OUTPUT -p tcp -d <model-host-ip> --dport 8888 -j ACCEPT

# Deny all other outbound to model network
iptables -A OUTPUT -d <model-network> -j DROP
```

### Proxy Configuration

**ZeroClaw Proxy Settings**:
```toml
# config.toml
[proxy]
enabled = true
http_proxy = "http://proxy-host:3128"
https_proxy = "http://proxy-host:3128"
no_proxy = "localhost,127.0.0.1"

# Only allow model traffic through proxy
[proxy.rules]
allow = ["model-host", "ollama-host"]
deny = ["*"]
```

---

## 📦 Deployment Steps

### Phase 1: Local Testing (Current)

1. **Install Ollama**:
   ```bash
   curl -fsSL https://ollama.ai/install.sh | sh
   ollama pull qwen3-vl:8b
   ```

2. **Configure ZeroClaw**:
   ```toml
   # Use test/test_config.toml
   default_provider = "ollama"
   default_model = "qwen3-vl:8b"
   ```

3. **Test Locally**:
   ```bash
   cd zeroclaw
   ./test/run_tests.sh
   ```

### Phase 2: Production Setup

1. **Deploy Model Host**:
   ```bash
   # Example: vLLM deployment
   pip install vllm
   python -m vllm.entrypoints.openai.api_server \
     --model Qwen/Qwen3-VL-8B \
     --host 0.0.0.0 \
     --port 8000
   ```

2. **Configure Firewall**:
   ```bash
   # On SecGen VM
   iptables -A OUTPUT -p tcp -d <model-host> --dport 8000 -j ACCEPT
   ```

3. **Update ZeroClaw Config**:
   ```toml
   default_provider = "compatible"
   default_model = "qwen3-vl-8b"
   api_url = "http://<model-host>:8000/v1"
   ```

4. **Test Connectivity**:
   ```bash
   curl http://<model-host>:8000/v1/models
   ```

---

## 🧪 Testing Checklist

### Local Testing
- [ ] Ollama running with Qwen3-VL-8B
- [ ] ZeroClaw connects to Ollama
- [ ] IRC channel works
- [ ] SecGen tools function correctly
- [ ] Flag validation works

### Production Testing
- [ ] Model host accessible through proxy
- [ ] Firewall rules correctly configured
- [ ] ZeroClaw connects to production model runner
- [ ] Latency acceptable (< 2s response time)
- [ ] Throughput meets requirements (100+ concurrent students)
- [ ] Isolation verified (no access to other systems)

---

## 📊 Model Comparison

| Model Runner | Pros | Cons | Best For |
|--------------|------|------|----------|
| **Ollama** | Easy setup, good for testing | Limited production features | Local testing, development |
| **vLLM** | High throughput, multi-model | More complex setup | Production, multi-tenant |
| **SGLang** | Optimized for structured output | Newer, less mature | Structured generation |
| **llama.cpp** | CPU inference, portable | Slower than GPU | Edge deployment, CPU-only |
| **KTransformers** | Advanced optimization | Complex setup | Specialized workloads |

---

## 🔧 Model Selection Guide

### For Cybersecurity Training

**Recommended**: Qwen3-VL-8B or larger
- Good tool calling capabilities
- Visual understanding for screenshots/diagrams
- Reasonable resource requirements

**Future Consideration**: Larger models (70B+)
- Better reasoning for complex scenarios
- More accurate flag validation
- Better personality differentiation

### Resource Requirements

| Model Size | VRAM Required | Throughput | Latency |
|------------|---------------|------------|---------|
| 8B | 16GB | High | Low (<500ms) |
| 32B | 48GB | Medium | Medium (<1s) |
| 70B+ | 80GB+ | Low | High (>1s) |

---

## 📝 Configuration Templates

### Template 1: Single Model, Single Runner
```toml
default_provider = "ollama"
default_model = "qwen3-vl:8b"

[ollama]
host = "model-host"
port = 11434
```

### Template 2: Multi-Model, Single Runner
```toml
default_provider = "ollama"
default_model = "qwen3-vl:8b"

[ollama]
host = "model-host"
port = 11434

[model_routes]
# Route complex reasoning to larger model
"complex_reasoning" = "qwen3-vl:72b"
# Route simple queries to smaller model
"simple_query" = "qwen3-vl:8b"
```

### Template 3: Multi-Model, Multi-Runner
```toml
default_provider = "compatible"
default_model = "qwen3-vl:8b"

[provider.compatible]
name = "vllm"
base_url = "http://vllm-host:8000/v1"

[provider.sglang]
host = "sglang-host"
port = 30000

[model_routes]
# Use vLLM for general queries
"*" = "vllm"
# Use SGLang for structured output
"structured_output" = "sglang"
```

---

## 🚨 Troubleshooting

### Connection Issues

**Problem**: ZeroClaw can't connect to model host

**Solutions**:
1. Check firewall rules: `iptables -L -n`
2. Test connectivity: `curl http://model-host:port/v1/models`
3. Verify proxy settings: `env | grep -i proxy`

### Model Loading Issues

**Problem**: Model fails to load or respond

**Solutions**:
1. Check model runner logs
2. Verify VRAM availability: `nvidia-smi`
3. Try smaller model or reduce batch size

### Latency Issues

**Problem**: Responses too slow (> 5s)

**Solutions**:
1. Use smaller model
2. Increase model runner resources
3. Enable quantization (4-bit, 8-bit)
4. Use GPU instead of CPU

---

**Last Updated**: February 27, 2026  
**Maintained By**: Infrastructure Team
