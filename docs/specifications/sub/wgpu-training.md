# Sub-spec: Sovereign-Stack WGPU Training

**Parent:** [shell-safety-inference.md](../shell-safety-inference.md) Section 5

---

## Purpose

Train a shell safety classifier (Qwen3-4B NF4 QLoRA) using entirely
sovereign Rust stack — no Python, no PyTorch, no NVIDIA CUDA. First LLM
fine-tuning pipeline running on non-NVIDIA hardware via WebGPU/Vulkan.

## Pipeline (3,500 lines of Rust)

| Module | Lines | Purpose |
|--------|-------|---------|
| `wgpu_trainer.rs` | 1298 | Orchestrator: 36-layer forward → loss → backward → AdamW |
| `wgpu_attention.rs` | 255 | Attention: QKV + LoRA + QK-norm + RoPE + GQA + O |
| `wgpu_backward.rs` | 268 | 36-layer backward: FFN grad + LoRA grad + CPU AdamW |
| `wgpu_checkpoint.rs` | 248 | LoRA adapter save/load (JSON, round-trip verified) |
| `wgpu_nf4.rs` | 492 | NF4 quantized weights: 7 projections/layer, GPU dequant |
| `wgpu_runner.rs` | 170 | Entry point: tokenize → embed → train → checkpoint |
| trueno backward.rs | 1000 | 7 WGSL compute shaders |

## Hardware

| Spec | Value |
|------|-------|
| CPU | Intel Xeon W-3245, 32 cores @ 3.2GHz |
| RAM | 283GB DDR4 |
| GPU | AMD Radeon Pro W5700X, 16GB GDDR6, Navi 10 |
| Backend | Vulkan (via wgpu-hal) |

## Training Runs (8 total)

| Run | Steps | Loss | Time/step | Notes |
|-----|-------|------|-----------|-------|
| v1 | 357 | 11→1.96 | 85s | FFN only, lm_head backward |
| v5 | 1556 | 3.09 plateau | 39s | Proof-of-concept |
| v6 | 866 | 55→4.5 | 100s | **Production**: real LoRA bwd, seq128, accum=4 |
| v7 | 200 | 55→13 | ~100s | 7-module LoRA (66M params), rank=32 |
| **v8** | **1058** | **79→4.7** | **~100s** | **All improvements** (NEFTune, focal, LoRA+) |

## Known Limitations

1. **Full-forward eval slow**: 36s/entry → 29h for full test set
2. **Norm-guard clips LoRA signal**: limits contribution in eval
3. **Single GPU**: only 1 of 2 W5700X GPUs used
4. **Loss plateau at 4.6**: needs cosine LR decay (entrenar upstream)

## Contracts

- `wgpu-training-v1.yaml` — 3 FALSIFY (shader parity, convergence, NF4)
- `wgpu-production-training-v1.yaml` — 7 FALSIFY (ship criteria, LoRA bwd)
- `wgpu-attention-stability-v1.yaml` — 2 FALSIFY (norm-guard, QK-norm)
- `wgpu-model-improvement-v1.yaml` — 2 FALSIFY (forward-only, 7-module)
- `cpu-gpu-forward-parity-v1.yaml` — 6 FALSIFY (RoPE, QK-norm, parity)

Full training history with Five Whys and debugging details:
[sub/ssc-v12-full-history.md](ssc-v12-full-history.md) Sections 16-18.
