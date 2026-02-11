# CUDA Optimization for Anna

Your system is CUDA-capable! Here's how to make Anna faster and more powerful.

## Current State

Anna uses Ollama with CPU inference by default. With CUDA, you can:
- 5-10x faster LLM responses
- Support larger models (70B+)
- Run multiple queries in parallel

## Quick Setup

### 1. Verify CUDA Installation

```bash
nvidia-smi  # Should show your GPU
nvcc --version  # CUDA compiler
```

### 2. Install CUDA-Enabled Ollama

```bash
# Check if Ollama is using CUDA
ollama ps

# If not, reinstall with CUDA support
curl -fsSL https://ollama.com/install.sh | sh
```

Ollama automatically detects and uses CUDA if available.

### 3. Verify GPU Usage

```bash
# Run a test query
ollama run qwen2.5:14b "test"

# Watch GPU usage in another terminal
watch -n 1 nvidia-smi
```

You should see GPU memory usage and utilization increase.

## Model Recommendations for CUDA

### Current Default: qwen2.5:14b
- Good balance of speed and quality
- ~8GB VRAM required
- Anna's default choice

### Upgrade Options:

**For RTX 3080+ (10GB+ VRAM)**:
```bash
# Faster, more capable
ollama pull qwen2.5:32b
export ANNA_OLLAMA_MODEL=qwen2.5:32b
```

**For RTX 4090 (24GB VRAM)**:
```bash
# Top-tier performance
ollama pull qwen2.5:72b
export ANNA_OLLAMA_MODEL=qwen2.5:72b
```

**For Multi-GPU Setups**:
```bash
# Llama 3.1 70B - excellent reasoning
ollama pull llama3.1:70b
export ANNA_OLLAMA_MODEL=llama3.1:70b
```

## Performance Tuning

### Increase Parallel Requests

In `/etc/anna/config.toml`:
```toml
[llm]
max_parallel_requests = 4  # Increase from default 2
timeout_secs = 30  # Reduce from 60 (faster with GPU)
```

### Enable Flash Attention (Faster Inference)

```bash
# Set Ollama environment variables
export OLLAMA_FLASH_ATTENTION=1
sudo systemctl restart ollama
```

### Optimize Context Window

Larger models support larger context:
```toml
[llm]
context_window = 8192  # For 32B+ models
```

## Benchmark Your Setup

Test Anna's response times:
```bash
# Measure LLM speed
time annactl "what is my disk usage"

# Should be <3 seconds with CUDA
# vs 10-15 seconds with CPU
```

## Monitor GPU Usage

Watch Anna using your GPU:
```bash
# Terminal 1: Run Anna queries
annactl "analyze my system logs"

# Terminal 2: Watch GPU
watch -n 0.5 'nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total --format=csv,noheader'
```

## Troubleshooting

### Ollama Not Using GPU

```bash
# Check Ollama logs
journalctl -u ollama -f

# Should see: "CUDA available: true"
```

If not using GPU:
```bash
# Reinstall with CUDA
sudo systemctl stop ollama
sudo rm /usr/local/bin/ollama
curl -fsSL https://ollama.com/install.sh | sh
sudo systemctl start ollama
```

### Out of VRAM

If you get CUDA OOM errors:
```bash
# Use smaller model
export ANNA_OLLAMA_MODEL=qwen2.5:7b

# Or reduce context
export OLLAMA_NUM_CTX=2048
```

### Multiple GPUs

Ollama uses all GPUs automatically. To use specific GPU:
```bash
export CUDA_VISIBLE_DEVICES=0  # Use only GPU 0
sudo systemctl restart ollama
```

## Expected Performance

| Model | VRAM | CPU Time | GPU Time | Quality |
|-------|------|----------|----------|---------|
| qwen2.5:7b | 4GB | 8s | 1s | Good |
| qwen2.5:14b | 8GB | 15s | 2s | Better |
| qwen2.5:32b | 16GB | 40s | 4s | Excellent |
| qwen2.5:72b | 40GB | 120s | 8s | Best |

*Times are approximate for typical Anna queries.*

## Future: Multi-GPU & Quantization

Anna will support:
- **Multi-GPU inference**: Distribute model across GPUs
- **Quantization**: Run 70B models on 16GB VRAM (GPTQ/AWQ)
- **Batching**: Process multiple queries simultaneously

## Advanced: Build Ollama with Custom CUDA Flags

For maximum performance:
```bash
git clone https://github.com/ollama/ollama
cd ollama
export CGO_CFLAGS="-O3 -march=native"
export CUDA_ARCH="sm_86"  # For RTX 3090/4090
make build
```

This optimizes for your specific GPU architecture.

## Summary

With CUDA, Anna becomes:
- **5-10x faster** - Sub-2-second responses
- **Smarter** - Can run larger, more capable models
- **More responsive** - Handle multiple queries in parallel

Your CUDA-capable system is Anna's superpower. Use it!

---

**Pro Tip**: Monitor GPU temperature and usage. If thermal throttling occurs, increase fan curves or improve cooling. Anna runs best on a cool GPU.
