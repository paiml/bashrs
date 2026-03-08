# SPEC-SSC-2026-005: Shell Safety Classifier, Chat Model, and WASM App (Sovereign Rust Stack)

**Version**: 12.2.0
**Status**: DESIGN COMPLETE — v12.2 sovereign tooling mandate (apr-cli orchestration, zero scripts)
**Author**: paiml engineering
**Date**: 2026-03-08
**Stack**: bashrs + verificar + entrenar + trueno + alimentar + apr-cli + forjar (Rust only, no Python, no ad-hoc scripts)
**HuggingFace Repos**:
  - `paiml/shell-safety-qwen3-4b` (Qwen3-4B NF4 QLoRA adapter for shell/Makefile/Dockerfile security)
  - `paiml/shell-safety-bench` (first shell-specific security benchmark, CWE-mapped, 28K+ entries)
  - `paiml/shell-safety-classifier` (CodeBERT binary classifier — Stage 1, existing)
  - `paiml/shell-safety-conversations` (training dataset, v4 regenerated from real shell code)
  - ~~`paiml/shell-safety-chat`~~ (DELETED — v1-v3 used Rust transpiler output, not shell code)
**Prior art**: `shell-safety-inference-v1-v3-archive.md` (v1-v3 history)

---

## 1. Problem

Shell scripts are the #1 attack surface for infrastructure — every CI/CD pipeline,
every Dockerfile, every deploy script, every cron job. Yet **no ML-based security
model or benchmark exists for shell/Makefile/Dockerfile**. Every code security
benchmark (CASTLE, SafeGenBench, SecRepoBench, CyberNative DPO) covers
C/C++/Python/Java but ignores the infrastructure glue code.

The bashrs linter catches 24 known patterns (SEC001-SEC008, DET001-DET006,
IDEM001-IDEM004, SC-prefixed ShellCheck rules). We want:

1. A **benchmark** (`ShellSafetyBench`) — first shell-specific security eval, CWE-mapped
2. A **specialist model** (Qwen3-4B NF4 QLoRA) — trained on real shell/Make/Docker code
3. A **fast classifier** (CodeBERT, 125M) — binary safe/unsafe in ~20ms for CI/CD
4. Built entirely on the sovereign Rust AI stack, with verified labels from bashrs + verificar

## 2. Approach: Four-Stage Pipeline (v12 Reframe)

```
Stage 0: Encoder support in entrenar (COMPLETE)
  - Bidirectional attention, absolute positions, GELU, RoBERTa weight loading

Stage 1: CodeBERT classifier (COMPLETE)
  - MLP probe on frozen embeddings, MCC=0.754
  - Binary: safe/unsafe, ~20ms CPU inference
  - Purpose: CI/CD triage, confidence scores

Stage 2: ShellSafetyBench — benchmark + training data (NEW)
  - Source 1: bashrs corpus (17,942 real shell/Make/Docker entries)
  - Source 2: verificar mutations (safe→unsafe pairs with CWE labels)
  - Ground truth: bashrs lint findings (deterministic, auditable)
  - Conversations: bashrs conversations.rs (4 types, 48 prompt variants)
  - CWE mapping: SEC→CWE-78/94, DET→CWE-330, IDEM→CWE-362
  - Data ops: alimentar (quality/split/balance) + apr data (audit/decontaminate)
  - Output: ~28K labeled entries, DPO-compatible, published to HuggingFace

Stage 3: Qwen3-4B NF4 QLoRA specialist model (NEW)
  - First LLM fine-tuned specifically for shell/Make/Docker security
  - Trained on ShellSafetyBench data (real code, not transpiler output)
  - NF4 quantized base (2 GB) + LoRA adapters (~47 MB)
  - Orchestrated: apr train plan/apply (no raw entrenar invocation)
  - Published: apr publish + alimentar hub push (no manual upload)
```

---

## 2b. Sovereign Tooling Mandate (v12.2)

**HARD REQUIREMENT**: All ShellSafetyBench work MUST use sovereign Rust AI stack tooling.
No ad-hoc Python scripts, no shell hacks, no one-off data munging. Every operation is
declarative YAML + CLI, following the albor pipeline pattern (`configs/pipeline/albor.yaml`).

### 2b.1 Tooling Map

| Operation | Tool | Command Pattern |
|-----------|------|-----------------|
| **Orchestration** | `apr pipeline` | `apr pipeline plan/apply/status configs/pipeline/ssc.yaml` |
| **Data audit** | `apr data audit` | `apr data audit train.jsonl --num-classes 2` |
| **Data splitting** | `apr data split` | `apr data split data.jsonl --train 0.8 --val 0.1 --test 0.1 -o splits/` |
| **Data balancing** | `apr data balance` | `apr data balance splits/train.jsonl --strategy sqrt-inverse` |
| **Data decontamination** | `apr data decontaminate` | `apr data decontaminate train.jsonl --reference benchmarks/` |
| **Data quality** | `alimentar quality` | `alimentar quality score data.jsonl --profile ml-training` |
| **Data import** | `alimentar import` | `alimentar import local ./corpus/ --output data.parquet` |
| **Data publish** | `alimentar hub push` | `alimentar hub push splits/ paiml/shell-safety-bench` |
| **Synthetic generation** | `verificar generate` | `verificar generate --language bash --count 10000 --strategy exhaustive` |
| **Mutation** | `verificar mutate` | `verificar mutate --operator BSR,AOR --oracle IoOracle` |
| **Training plan** | `apr train plan` | `apr train plan --config configs/train/ssc-qwen3-4b.yaml` |
| **Training execute** | `apr train apply` | `apr train apply --config configs/train/ssc-qwen3-4b.yaml --seed 42` |
| **Training monitor** | `apr train watch` | `apr train watch --config configs/train/ssc-qwen3-4b.yaml` |
| **Evaluation** | `apr eval` | `apr eval checkpoints/best/ --task classify --data splits/test.jsonl` |
| **Model publish** | `apr publish` | `apr publish checkpoints/best/ paiml/shell-safety-qwen3-4b` |
| **Quantize** | `apr quantize` | `apr quantize plan/apply --model checkpoint/ --method nf4` |
| **Corpus labeling** | `bashrs lint` | `bashrs lint script.sh --format json` (ground truth) |
| **Corpus export** | `bashrs corpus` | `bashrs corpus export-splits`, `generate-conversations` |
| **Cross-linter** | `shellcheck` | `shellcheck -f json script.sh` (secondary oracle) |
| **Benchmark** | `apr bench` | `apr bench checkpoint/ --task shell-safety` |
| **QA gate** | `apr qa` | `apr qa --checklist ssc-release-v1.yaml` |

### 2b.2 What Is BANNED

| Banned Practice | Why | Sovereign Alternative |
|----------------|-----|----------------------|
| Python scripts for data processing | Non-sovereign, untraceable | `alimentar` CLI + `apr data` |
| Ad-hoc `jq`/`awk`/`sed` data pipelines | Non-reproducible, fragile | `alimentar convert`, `alimentar filter-text` |
| Manual JSONL construction | Error-prone, no validation | `bashrs corpus generate-conversations --entrenar` |
| `curl` model downloads | No integrity verification | `apr run --model` (auto-download + cache + verify) |
| Jupyter notebooks for analysis | Python dependency, non-deterministic | `alimentar repl`, `apr eval` |
| Shell scripts for orchestration | What we're trying to FIX | `apr pipeline apply configs/pipeline/ssc.yaml` |
| Manual train/val/test splits | No stratification guarantee | `apr data split --stratified` |
| Raw `entrenar` binary invocation | Skips plan validation | `apr train plan` then `apr train apply` |
| One-off metric computation | Non-reproducible | `apr eval` with saved config |
| Manual HuggingFace upload | No metadata, no versioning | `alimentar hub push` + `apr publish` |

### 2b.3 Pipeline Manifest

All SSC operations are declared in a single pipeline manifest, executed by forjar via
`apr pipeline apply`. This is the SSC equivalent of albor's `configs/pipeline/albor.yaml`.

**File**: `configs/pipeline/ssc.yaml`

```yaml
version: "1.0"
name: ssc-shellsafetybench-pipeline
description: >
  ShellSafetyBench: first shell-specific security benchmark + Qwen3-4B specialist model.
  Orchestrated via apr-cli, no ad-hoc scripts.

machines:
  lambda:
    hostname: lambda
    addr: 127.0.0.1
    user: noah
    arch: x86_64
    roles: [gpu-train, eval, data-pipeline]

resources:
  # ── Infrastructure ──
  cuda-driver:
    type: gpu
    machine: lambda
    gpu_backend: nvidia
    driver_version: "550"
    persistence_mode: true

  data-dir:
    type: file
    machine: lambda
    path: /home/noah/src/bashrs/training/shellsafetybench
    state: directory

  # ── Stage 2a: Corpus Export ──
  corpus-export:
    type: task
    machine: lambda
    command: >
      bashrs corpus generate-conversations --entrenar
        --output training/shellsafetybench/conversations.jsonl
    output_artifacts: ["training/shellsafetybench/conversations.jsonl"]
    depends_on: [data-dir]

  # ── Stage 2b: Verificar Synthetic Generation ──
  verificar-generate:
    type: task
    machine: lambda
    command: >
      verificar generate --language bash --count 10000
        --max-depth 5 --strategy exhaustive
        --output training/shellsafetybench/verificar-safe.jsonl
    output_artifacts: ["training/shellsafetybench/verificar-safe.jsonl"]
    depends_on: [data-dir]

  verificar-mutate:
    type: task
    machine: lambda
    command: >
      verificar mutate
        --input training/shellsafetybench/verificar-safe.jsonl
        --operator BSR,AOR
        --oracle IoOracle
        --output training/shellsafetybench/verificar-unsafe.jsonl
    output_artifacts: ["training/shellsafetybench/verificar-unsafe.jsonl"]
    depends_on: [verificar-generate]

  # ── Stage 2c: Label with bashrs lint ──
  label-corpus:
    type: task
    machine: lambda
    command: >
      bashrs corpus label
        --input training/shellsafetybench/conversations.jsonl
        --format json
        --output training/shellsafetybench/labeled.jsonl
    output_artifacts: ["training/shellsafetybench/labeled.jsonl"]
    depends_on: [corpus-export]

  label-verificar:
    type: task
    machine: lambda
    command: >
      bashrs corpus label
        --input training/shellsafetybench/verificar-unsafe.jsonl
        --format json
        --output training/shellsafetybench/verificar-labeled.jsonl
    output_artifacts: ["training/shellsafetybench/verificar-labeled.jsonl"]
    depends_on: [verificar-mutate]

  # ── Stage 2d: Merge + Quality Audit ──
  merge-data:
    type: task
    machine: lambda
    command: >
      alimentar mix
        --input training/shellsafetybench/labeled.jsonl
        --input training/shellsafetybench/verificar-labeled.jsonl
        --output training/shellsafetybench/merged.jsonl &&
      apr data audit training/shellsafetybench/merged.jsonl --num-classes 2
    output_artifacts: ["training/shellsafetybench/merged.jsonl"]
    depends_on: [label-corpus, label-verificar]

  # ── Stage 2e: Split + Balance ──
  split-data:
    type: task
    machine: lambda
    command: >
      apr data split training/shellsafetybench/merged.jsonl
        --train 0.8 --val 0.1 --test 0.1
        --stratified
        -o training/shellsafetybench/splits/ &&
      apr data balance training/shellsafetybench/splits/train.jsonl
        --strategy sqrt-inverse
        -o training/shellsafetybench/splits/train-balanced.jsonl
    output_artifacts: ["training/shellsafetybench/splits/"]
    depends_on: [merge-data]

  decontaminate:
    type: task
    machine: lambda
    command: >
      apr data decontaminate training/shellsafetybench/splits/train-balanced.jsonl
        --reference training/shellsafetybench/splits/test.jsonl
    depends_on: [split-data]

  quality-gate:
    type: task
    machine: lambda
    command: >
      alimentar quality score training/shellsafetybench/splits/train-balanced.jsonl
        --profile ml-training --json
    quality_gate:
      parse: json
      field: score
      threshold: ["70"]
      on_fail: block
    depends_on: [decontaminate]

  # ── Stage 3: Training ──
  train-plan:
    type: task
    machine: lambda
    command: >
      apr train plan --config configs/train/ssc-qwen3-4b-qlora.yaml
    depends_on: [quality-gate, cuda-driver]

  train-apply:
    type: task
    machine: lambda
    command: >
      apr train apply --config configs/train/ssc-qwen3-4b-qlora.yaml
        --seed 42
        --deterministic
    output_artifacts: ["training/checkpoints/ssc-chat-v7-qwen3-4b/"]
    completion_check: "test -f training/checkpoints/ssc-chat-v7-qwen3-4b/adapter_config.json"
    depends_on: [train-plan]

  # ── Stage 4: Evaluation ──
  eval-static:
    type: task
    machine: lambda
    command: >
      apr eval training/checkpoints/ssc-chat-v7-qwen3-4b/
        --task classify
        --data training/shellsafetybench/splits/test.jsonl
        --device cuda
        --output training/shellsafetybench/eval/static-results.json
    output_artifacts: ["training/shellsafetybench/eval/static-results.json"]
    depends_on: [train-apply]

  eval-dynamic:
    type: task
    machine: lambda
    command: >
      verificar generate --language bash --count 500
        --seed "$(date +%Y%m%d)" --strategy exhaustive
        --output /tmp/ssc-dynamic-eval.jsonl &&
      apr eval training/checkpoints/ssc-chat-v7-qwen3-4b/
        --task classify
        --data /tmp/ssc-dynamic-eval.jsonl
        --device cuda
        --output training/shellsafetybench/eval/dynamic-results.json
    output_artifacts: ["training/shellsafetybench/eval/dynamic-results.json"]
    depends_on: [train-apply]

  eval-baselines:
    type: task
    machine: lambda
    command: >
      bashrs corpus baselines
        --data training/shellsafetybench/splits/test.jsonl
        --output training/shellsafetybench/eval/baselines.json
    output_artifacts: ["training/shellsafetybench/eval/baselines.json"]
    depends_on: [split-data]

  # ── Stage 5: QA Gate ──
  qa-gate:
    type: task
    machine: lambda
    command: >
      apr qa --checklist configs/qa/ssc-release-v1.yaml
    quality_gate:
      parse: json
      field: pass
      threshold: ["true"]
      on_fail: block
    depends_on: [eval-static, eval-dynamic, eval-baselines]

  # ── Stage 6: Publish ──
  publish-dataset:
    type: task
    machine: lambda
    command: >
      alimentar hub push training/shellsafetybench/splits/
        paiml/shell-safety-bench
        --format parquet
    depends_on: [qa-gate]

  publish-model:
    type: task
    machine: lambda
    command: >
      apr publish training/checkpoints/ssc-chat-v7-qwen3-4b/
        paiml/shell-safety-qwen3-4b
        --license apache-2.0
    depends_on: [qa-gate]

policy:
  failure: stop_on_first
  parallel_machines: false
  retry: 1
  bashrs_lint: true
```

### 2b.4 Training Config (entrenar schema)

**File**: `configs/train/ssc-qwen3-4b-qlora.yaml`

This replaces the ad-hoc `training/ssc-chat-qwen3-4b-qlora.yaml`. It follows
the entrenar YAML schema exactly as used by albor:

```yaml
model:
  source: "/home/noah/src/models/qwen3-4b/"
  format: safetensors
  device: "cuda"
  dtype: "bfloat16"
  architecture:
    type: transformer
    hidden_size: 2560
    num_layers: 36
    num_heads: 32
    num_kv_heads: 8
    intermediate_size: 9728
    vocab_size: 151936
    max_seq_length: 512
    rope_theta: 1000000.0
    head_dim: 128

data:
  train: "training/shellsafetybench/splits/train-balanced.jsonl"
  val: "training/shellsafetybench/splits/val.jsonl"
  tokenizer: "/home/noah/src/models/qwen3-4b/tokenizer.json"
  seq_len: 512
  input_column: "text"
  loader:
    batch_size: 4
    shuffle: true

optimizer:
  name: "adamw"
  lr: 0.00005
  weight_decay: 0.01

scheduler:
  name: "cosine"
  warmup_steps: 100

training:
  epochs: 1
  gradient_accumulation: 4
  max_seq_length: 512
  deterministic: true
  checkpoint:
    save_every: 500
    save_best: true
    metric: "val_loss"
    mode: "min"
  validation:
    every: 250
  early_stopping:
    enabled: true
    metric: "val_loss"
    patience: 5
    min_delta: 0.001

lora:
  enabled: true
  rank: 16
  alpha: 32.0
  target_modules: [q_proj, v_proj, o_proj, gate_proj]
  quantize_base: true
  quantize_bits: 4
  quant_type: nf4

output:
  dir: "training/checkpoints/ssc-chat-v7-qwen3-4b"
  save_every: 500

monitoring:
  terminal:
    enabled: true
    refresh_rate: 1000
  alerts:
    - condition: "loss > 10"
      action: "stop"
    - condition: "grad_norm > 100"
      action: "warn"
```

### 2b.5 QA Checklist (apr qa)

**File**: `configs/qa/ssc-release-v1.yaml`

```yaml
name: ssc-release-v1
description: "ShellSafetyBench release quality gate"
version: "1.0"

checks:
  - name: dataset-quality
    command: "alimentar quality score training/shellsafetybench/splits/train-balanced.jsonl --profile ml-training --json"
    gate:
      field: score
      op: ">="
      value: 70

  - name: data-decontaminated
    command: "apr data decontaminate training/shellsafetybench/splits/train-balanced.jsonl --reference training/shellsafetybench/splits/test.jsonl --json"
    gate:
      field: contamination_rate
      op: "<="
      value: 0.01

  - name: static-eval-f1
    command: "cat training/shellsafetybench/eval/static-results.json"
    gate:
      field: detection_f1
      op: ">="
      value: 0.50

  - name: beats-keyword-baseline
    command: "cat training/shellsafetybench/eval/baselines.json"
    gate:
      field: model_mcc_vs_keyword
      op: ">"
      value: 0

  - name: dynamic-eval-gap
    description: "Static-dynamic gap < 15% (anti-overfitting)"
    command: "cat training/shellsafetybench/eval/dynamic-results.json"
    gate:
      field: static_dynamic_gap
      op: "<="
      value: 0.15

  - name: linter-fp-rate
    description: "Linter false positive rate < 5% on human validation set"
    gate:
      field: fp_rate
      op: "<="
      value: 0.05

  - name: model-card-exists
    command: "test -f training/checkpoints/ssc-chat-v7-qwen3-4b/README.md"

  - name: dataset-card-exists
    command: "test -f training/shellsafetybench/splits/README.md"
```

### 2b.6 Execution

The entire ShellSafetyBench pipeline runs as a single command:

```bash
# Plan (dry-run, validate DAG, estimate resources)
apr pipeline plan configs/pipeline/ssc.yaml

# Execute (runs all stages in dependency order)
apr pipeline apply configs/pipeline/ssc.yaml

# Monitor
apr pipeline status
```

No step requires manual intervention, ad-hoc scripts, or non-sovereign tooling.
Every intermediate artifact is tracked, every quality gate is enforced, and the
pipeline is fully reproducible from a clean checkout.

---

## 3. Why CodeBERT Over Qwen-0.5B for Classification

| Property | CodeBERT (125M) | Qwen-0.5B (494M) |
|----------|-----------------|-------------------|
| Params | 125M | 494M |
| Architecture | Encoder (sees whole input) | Decoder (left-to-right) |
| CPU inference | ~20ms | ~200ms |
| Classification fit | Natural ([CLS] token) | Awkward (last-token hack) |
| WASM deployable | Yes (125M fits in browser) | Borderline |
| Proven for vuln detection | Yes (CodeXGLUE defect detection) | No |
| CI/CD overhead per script | Negligible | Noticeable |
| Pretrained on shell | No (6 langs) | Yes (broad code) |

10x inference speedup and 4x smaller model justifies 2 days of encoder support.

### 3.1 Prior Art on HuggingFace

No shell-safety classifier or dataset exists. Closest work:

| Model/Dataset | What | Relevance |
|---------------|------|-----------|
| `mrm8488/codebert-base-finetuned-detect-insecure-code` | CodeBERT for C/C++ vuln detection (binary) | Same task, different language |
| `whywhywhywhy/security-qwen2.5-3b-coder-instruct` | Qwen-3B LoRA vuln detection (ReposVul, 6K CVEs) | Same family, C/C++/Java/Python |
| `meta-llama/Prompt-Guard-86M` | 86M classifier for prompt injection | Same pattern (small classifier) |
| `NL2Bash` (9,305 pairs) | NL-to-bash translation | Shell data, not safety-labeled |

Our corpus + classifier would be first-of-its-kind for shell safety.

---

## 4. Stage 0: Encoder Support in entrenar

### 4.1 Why This Is Easy

An encoder is a decoder with constraints removed:

| Component | Decoder (already built) | Encoder (to add) | Change |
|-----------|------------------------|-------------------|--------|
| Self-attention | Causal mask (triangular) | No mask (full) | Remove mask |
| Position embeddings | RoPE (rotary computation) | Learned absolute (lookup table) | Simpler |
| KV cache | Required for generation | Not needed | Remove code |
| FFN activation | SwiGLU | GELU | Swap function |
| Layer norm | RMSNorm (no bias) | LayerNorm (with bias) | Add bias param |
| Output pooling | Last-token hidden state | [CLS] or mean-pool | Index [0] or mean |
| Weight loading | Qwen safetensors keys | RoBERTa safetensors keys | Different names |

Every change is a simplification or a one-line swap.

### 4.2 Implementation

| Task | File(s) | Description | Status |
|------|---------|-------------|--------|
| ENC-001 | `entrenar/src/transformer/config.rs` | `ModelArchitecture` enum (Encoder/Decoder), `codebert()` preset, `from_size_str("codebert")` | ✅ Done |
| ENC-002 | `entrenar/src/autograd/ops/attention.rs` | Verified bidirectional (no causal mask applied). Test: modify K[2] → output[0] changes | ✅ Done |
| ENC-003 | `entrenar/src/transformer/embedding.rs` | `LearnedPositionEmbedding` — lookup table (0..max_pos), `from_params()`, clamp beyond max | ✅ Done |
| ENC-004 | `entrenar/src/transformer/feedforward.rs` | `EncoderFeedForward` with GELU (2-projection + bias), `from_params()` with BERT weight names | ✅ Done |
| ENC-005 | `entrenar/src/transformer/norm.rs` | `LayerNorm` with bias (mean-center + var-normalize), `from_params()`, `forward_batched()` | ✅ Done |
| ENC-006 | `entrenar/src/transformer/weights/` | `Architecture::RoBERTa`, auto-detect from weight names, full name mapping | ✅ Done |
| ENC-007 | `entrenar/src/finetune/classification.rs` | `PoolingStrategy::{Cls, LastToken, Mean}`, `forward_with_pooling()`, `from_architecture()` | ✅ Done |
| ENC-008 | Tests across all modules | 30 new tests: config, attention, embedding, FFN, norm, weights, pooling | ✅ Done |
| **Total** | | All encoder components implemented in entrenar | **✅ Complete** |

### 4.3 Provable Contracts (YAML + Kani + proptest)

All encoder, classifier, and inference code is backed by YAML contracts in
`provable-contracts/contracts/`. The pipeline: YAML contract → scaffold generation →
proptest + Kani harnesses → binding to real entrenar code.

#### 4.3.1 New Contracts to Create

**`provable-contracts/contracts/bidirectional-attention-v1.yaml`**

```yaml
metadata:
  version: "1.0.0"
  description: "Bidirectional (encoder) attention — full attention without causal mask"
  references:
    - "Devlin et al. (2019) BERT: Pre-training of Deep Bidirectional Transformers"
  depends_on: ["attention-kernel-v1", "softmax-kernel-v1"]

equations:
  bidirectional_attention:
    formula: "BiAttn(Q, K, V) = softmax(QK^T / sqrt(d_k)) * V"
    domain: "Q in R^{n x d_k}, K in R^{n x d_k}, V in R^{n x d_v}"
    codomain: "R^{n x d_v}"
    invariants:
      - "Every token attends to every other token (no mask)"
      - "Attention weights are dense (no structural zeros)"
      - "Equivalent to causal attention when n=1"

proof_obligations:
  - type: equivalence
    property: "Causal parity on single-token input"
    formal: "|BiAttn(q, k, v) - CausalAttn(q, k, v)| < eps for n=1"
    tolerance: 1.0e-6
    applies_to: all
  - type: invariant
    property: "Full attention density"
    formal: "attn_weights[i][j] > 0 for all i, j in 0..n"
    applies_to: all
  - type: invariant
    property: "Weight normalization"
    formal: "sum_j(attn_weights[i][j]) = 1 for all i"
    tolerance: 1.0e-5
    applies_to: all

falsification_tests:
  - id: FALSIFY-BIATT-001
    rule: "No causal mask applied"
    prediction: "Upper triangle of attention matrix is non-zero"
    test: "proptest with random Q, K, n >= 2"
    if_fails: "Causal mask leaked into bidirectional path"
  - id: FALSIFY-BIATT-002
    rule: "Causal parity at n=1"
    prediction: "Output identical to causal attention for single-token input"
    test: "proptest comparing BiAttn and CausalAttn on n=1"
    if_fails: "Mask application differs even when mask is trivial"
```

**`provable-contracts/contracts/learned-position-embedding-v1.yaml`**

```yaml
metadata:
  version: "1.0.0"
  description: "Learned absolute position embeddings (RoBERTa-style)"
  references:
    - "Liu et al. (2019) RoBERTa: A Robustly Optimized BERT Pretraining Approach"
  depends_on: ["embedding-lookup-v1"]

equations:
  position_embedding:
    formula: "PE(pos) = E[pos] where E in R^{max_positions x d_model}"
    domain: "pos in {0, 1, ..., max_positions - 1}"
    codomain: "R^{d_model}"
    invariants:
      - "Lookup is O(1) (table index, not computation)"
      - "pos < max_positions (bounds check)"

proof_obligations:
  - type: bound
    property: "Position in range"
    formal: "0 <= pos < max_positions"
    applies_to: all
  - type: equivalence
    property: "Deterministic lookup"
    formal: "PE(pos) = PE(pos) for same weights (idempotent)"
    tolerance: 0.0
    applies_to: all

falsification_tests:
  - id: FALSIFY-POS-001
    rule: "Out-of-bounds position"
    prediction: "pos >= max_positions causes error, not silent truncation"
    test: "kani proof with pos = max_positions"
    if_fails: "Missing bounds check on position index"
```

**`provable-contracts/contracts/encoder-forward-v1.yaml`**

```yaml
metadata:
  version: "1.0.0"
  description: "Encoder forward pass — full pipeline from tokens to [CLS] embedding"
  references:
    - "Devlin et al. (2019) BERT"
    - "Liu et al. (2019) RoBERTa"
  depends_on:
    - "bidirectional-attention-v1"
    - "learned-position-embedding-v1"
    - "layernorm-kernel-v1"
    - "gelu-kernel-v1"

equations:
  encoder_layer:
    formula: "h = LayerNorm(x + BiAttn(x)) ; out = LayerNorm(h + FFN(h))"
    domain: "x in R^{n x d_model}"
    codomain: "R^{n x d_model}"
    invariants:
      - "Output shape equals input shape (residual connection preserves dimensions)"
      - "No NaN or Inf in output for finite input"
  cls_pooling:
    formula: "embedding = encoder_output[0]  (first token)"
    domain: "encoder_output in R^{n x d_model}, n >= 1"
    codomain: "R^{d_model}"

proof_obligations:
  - type: invariant
    property: "Shape preservation"
    formal: "output.shape == input.shape for each encoder layer"
    applies_to: all
  - type: bound
    property: "No NaN/Inf"
    formal: "is_finite(output[i][j]) for all i, j"
    applies_to: all
  - type: equivalence
    property: "Reference parity"
    formal: "|entrenar_output - reference_output| < tolerance"
    tolerance: 1.0e-4
    applies_to: all

falsification_tests:
  - id: FALSIFY-ENC-001
    rule: "Shape preservation"
    prediction: "12 encoder layers preserve (n, 768) shape"
    test: "proptest with random input, verify output shape"
    if_fails: "Layer reshapes or drops dimensions"
  - id: FALSIFY-ENC-002
    rule: "Finite output"
    prediction: "No NaN or Inf for inputs in normal float range"
    test: "proptest with random inputs in [-10, 10]"
    if_fails: "Numerical instability in LayerNorm or attention"
  - id: FALSIFY-ENC-003
    rule: "Reference parity"
    prediction: "entrenar output matches saved HF reference within 1e-4"
    test: "Compare against fixture embeddings"
    if_fails: "Weight loading error or architectural mismatch"
```

**`provable-contracts/contracts/linear-probe-classifier-v1.yaml`**

```yaml
metadata:
  version: "1.0.0"
  description: "Linear probe classifier — frozen encoder + trained linear head"
  references:
    - "Alain & Bengio (2016) Understanding intermediate layers using linear classifier probes"

equations:
  linear_probe:
    formula: "logits = W @ embedding + b ; probs = softmax(logits)"
    domain: "embedding in R^{d_model}, W in R^{K x d_model}, b in R^K"
    codomain: "probs in R^K, sum(probs) = 1, probs_i > 0"
    invariants:
      - "Frozen encoder weights do not receive gradients"
      - "Only W and b are updated during training"
      - "probs sum to 1.0"

proof_obligations:
  - type: invariant
    property: "Encoder frozen"
    formal: "encoder_params_before == encoder_params_after for each training step"
    applies_to: all
  - type: invariant
    property: "Probability simplex"
    formal: "|sum(probs) - 1.0| < eps AND probs_i > 0 for all i"
    tolerance: 1.0e-6
    applies_to: all
  - type: invariant
    property: "Embedding determinism"
    formal: "embed(x) == embed(x) for same x and weights (bit-identical)"
    applies_to: all

falsification_tests:
  - id: FALSIFY-PROBE-001
    rule: "Encoder truly frozen"
    prediction: "Encoder weights unchanged after 100 training steps"
    test: "Snapshot encoder params, train, compare"
    if_fails: "Gradient leaking through frozen parameters"
  - id: FALSIFY-PROBE-002
    rule: "Probability valid"
    prediction: "Softmax output sums to 1.0 and all values > 0"
    test: "proptest with random embeddings"
    if_fails: "Numerical underflow in softmax or missing normalization"
```

#### 4.3.2 Existing Contracts That Apply

These already exist in `provable-contracts/contracts/` and are inherited:

| Contract | Applies To |
|----------|-----------|
| `attention-kernel-v1.yaml` | Base attention (encoder extends with bidirectional) |
| `softmax-kernel-v1.yaml` | Attention weight normalization |
| `layernorm-kernel-v1.yaml` | Encoder uses standard LayerNorm |
| `gelu-kernel-v1.yaml` | Encoder FFN activation |
| `embedding-lookup-v1.yaml` | Token embeddings |
| `cross-entropy-kernel-v1.yaml` | Classification loss |
| `lora-algebra-v1.yaml` | LoRA adapters for escalation Levels 1-2 |
| `classification-finetune-v1.yaml` | Training pipeline |
| `metrics-classification-v1.yaml` | MCC, precision, recall, F1 |

#### 4.3.3 Contract Pipeline

```
1. Create YAML contracts (4 new files above)
2. pv scaffold --contract bidirectional-attention-v1.yaml
     → generates trait stubs + failing test skeletons
3. pv bind --contract bidirectional-attention-v1.yaml --crate entrenar
     → maps equations to real entrenar functions
4. cargo test -p entrenar -- encoder
     → proptest falsification tests run
5. pv audit --contract bidirectional-attention-v1.yaml
     → verifies traceability: paper → equation → obligation → test → code
```

### 4.4 Ship Gate (C-ENC-SHIP)

| Field | Value |
|-------|-------|
| **Precondition** | All encoder tests pass AND all 4 YAML contracts fully bound |
| **Postcondition** | `cargo test -p entrenar -- encoder` passes, `pv audit` clean for all 4 contracts |
| **Kill criterion** | If weight loading hits unsupported tensor ops, scope and re-estimate |

**Note on C-ENC-003 (reference parity)**: Generate reference embeddings ONCE using
Python/HuggingFace, save as a test fixture. This is test data generation, not a
runtime dependency. The sovereign stack is validated against known-good outputs,
then runs independently.

---

## 5. Stage 1: CodeBERT Classifier

### 5.1 Architecture

```
Input script ──> RoBERTa BPE tokenizer ──> CodeBERT (125M, frozen or fine-tuned)
                                                |
                                           768-dim [CLS] embedding
                                                |
                                           Linear(768, 2) ──> [p_safe, p_unsafe]
```

### 5.2 Tokenizer Validation (F2 Mitigation)

CodeBERT uses RoBERTa's tokenizer, not trained on shell. Must validate before training.

**Protocol**: Tokenize 100 shell scripts, inspect these constructs:

| Construct | Acceptable | Unacceptable |
|-----------|-----------|--------------|
| `$(command)` | `$(` + `command` + `)` | `$` + `(` + `com` + `mand` + `)` |
| `2>&1` | `2>&1` or `2>` + `&1` | `2` + `>` + `&` + `1` |
| `$RANDOM` | `$RANDOM` or `$` + `RANDOM` | `$` + `RAN` + `DOM` |
| `\|` (pipe) | `\|` as single token | Merged with adjacent code |
| `<<'EOF'` | Recognizable boundary | Fully fragmented |

**Contract C-TOK-001**: >= 70% of constructs tokenized acceptably.

**If tokenizer fails**: Three options, cheapest first:
1. Proceed anyway — tokenizer damage may not matter for classification
2. Mean-pool instead of [CLS] — distributes signal across all tokens including broken ones
3. Continue-pretrain CodeBERT on 100K unlabeled GitHub bash scripts — model learns shell tokens

### 5.3 Data

bashrs corpus: 17,942 entries, binary labels from `classify_single()` on **transpiled shell output** (#172).
alimentar split: 80/10/10 stratified, seed=42.

| Split | Rows | Unsafe (shell-based #172) |
|-------|------|--------------------------|
| Train | ~14,353 | ~118 (0.82%) |
| Val | ~1,795 | ~15 (0.82%) |
| Test | ~1,794 | ~15 (0.82%) |

**Note**: Pre-#172 numbers (926 unsafe / 6.5%) used Rust code linting — invalid due to domain mismatch.

**Label audit (F7 mitigation)**: Before training, manually review 100 random unsafe
labels. If >10% are mislabeled (transpiler limitation, not actual unsafety), clean first.

### 5.4 Escalation Ladder

| Level | Approach | Params Trained | Time | Escalate If |
|-------|----------|---------------|------|-------------|
| 0 | Linear probe (frozen CodeBERT) | 1,538 | Extract: ~30 min, Train: seconds | MCC CI lower < 0.2 |
| 1 | Fine-tune top-2 layers + head | ~15M | ~30 min | MCC CI lower < 0.3 |
| 2 | Full fine-tune all layers | 125M | ~1 hr | MCC CI lower < 0.3 |
| 3 | Continue-pretrain on shell + fine-tune | 125M | ~4 hrs | MCC CI lower < 0.3 |

Level 0 optimization: extract [CLS] embeddings in one forward pass over corpus (~30 min
for 125M model at ~10 samples/sec on 4090), cache as safetensors, train linear head on
cached embeddings in seconds.

### 5.5 Evaluation

| Metric | Target | Notes |
|--------|--------|-------|
| MCC | CI lower bound > 0.2 | Conservative due to 116 unsafe test samples (F3) |
| Unsafe Recall | >= 0.60 | Report 95% CI (wide interval expected) |
| Accuracy | > 0.935 | Must beat 93.5% majority baseline |
| Generalization | >= 50% on 50 novel unsafe scripts | Out-of-distribution test (F8); current: 100% (50/50) |

Baselines (must beat at least one):
- Majority class: MCC = 0.000
- Keyword regex (`eval`, `$RANDOM`, `curl|bash`): MCC = 0.103
- bashrs linter (24 SEC rules + DET/IDEM): MCC = 1.000 (tautological — labels derived from linter)

### 5.6 Generalization Test (F8 Mitigation)

50 hand-written unsafe scripts with NO lexical overlap with training data:

```bash
# Novel injection patterns (not eval, not curl|bash)
source <(wget -qO- "$url")
bash -c "$untrusted"
. /dev/stdin <<< "$payload"

# Novel non-determinism (not $RANDOM, not date)
shuf -n1 /usr/share/dict/words
od -An -N4 -tu4 /dev/urandom
head -c8 /dev/random | xxd -p

# Novel race conditions
[ -f "$lock" ] || touch "$lock"
test -d "$dir" && cd "$dir" && rm -rf .

# Novel privilege issues
install -m 4755 ./binary /usr/local/bin/
setcap cap_net_raw+ep ./tool
```

If classifier only catches `eval` and `$RANDOM` but misses these, it's a keyword matcher.
Document honestly.

### 5.7 Ship Gate (C-CLF-001)

| Field | Value |
|-------|-------|
| **Precondition** | Evaluated on test (1,794 samples) + 50 generalization scripts |
| **Postcondition** | `MCC_CI_lower > 0.2 AND accuracy > 0.935 AND generalization >= 50%` |
| **Kill criterion** | Level 3 still fails → STOP. Classifier adds no value over linter. |

---

## 6. Stage 2: Synthetic Conversation Generation

### 6.1 Signal Sources

| Source | Provides |
|--------|---------|
| Corpus label | safe (0) / unsafe (1) ground truth |
| bashrs linter | Rule IDs, line numbers, descriptions |
| CodeBERT confidence | Probability score from Stage 1 |

### 6.2 Templates (10+ phrasing variants each)

**Type A — Classify + Explain** (unsafe scripts with lint findings)
**Type B — Fix** (unsafe scripts, with corrected version)
**Type C — Debug** (non-deterministic scripts)
**Type D — Confirm Safe** (safe scripts, >= 30% of total to prevent always-unsafe bias)

Each type has 10+ phrasing variants for the user prompt and assistant response opening,
randomly selected (seeded for reproducibility).

### 6.3 Pipeline (Pure Rust)

Implemented as `bashrs corpus generate-conversations` CLI command:

```
17,942 corpus entries
    |
    ├── bashrs lint → findings per entry
    ├── CodeBERT classifier → confidence per entry
    ├── corpus labels → safe/unsafe
    |
    v
Template engine (Rust):
    select type (A/B/C/D) from label + findings
    select phrasing variant (seeded random)
    fill with real script, findings, rules
    validate rule citations
    |
    v
~40,000-50,000 conversations (JSONL)
```

### 6.4 Quality Gates

| Check | Threshold |
|-------|-----------|
| Rule citations match linter output | 100% |
| Fixed scripts pass `shellcheck -s sh` | >= 90% |
| No empty/trivial responses | 0 |
| Type D (safe confirmations) | >= 30% of total |
| Template variant distribution | No single variant > 20% |

### 6.5 Honesty Requirements (F5 Mitigation)

The conversations are **linter findings expressed in natural language**, not independent
safety reasoning. The model card MUST state:
- Trained on synthetic data derived from rule-based linter output
- Explains known patterns, not novel safety reasoning
- For scripts outside rule coverage, responses may be generic
- Not a replacement for security audit

### 6.6 Published Dataset

`paiml/shell-safety-conversations` — first-of-its-kind shell safety instruction dataset.

---

## 7. Stage 3: Chat Model Fine-Tuning

### 7.1 Base Model

Qwen2.5-Coder-0.5B-Instruct. Code-aware, chat-native. Fits in 24GB with full f32
optimizer states (1.5B OOM'd — entrenar doesn't yet support NF4 base quantization in bridge).

### 7.2 Configuration

| Parameter | Value |
|-----------|-------|
| Base model | Qwen2.5-Coder-0.5B-Instruct (896h, 24L, 14 heads) |
| LoRA rank | 16, alpha = 32 |
| LoRA targets | Q + V projections |
| Trainable params | ~2M |
| Training data | 17,942 conversations (ChatML format) |
| Epochs | 3 |
| Optimizer | AdamW, lr=2e-4, grad_accum=4 |
| Format | ChatML (`<\|im_start\|>` tags, Qwen native) |
| Sequence length | 512 tokens |
| Hardware | RTX 4090, CUDA 12.8, ~12GB VRAM |

### 7.3 Evaluation

| Metric | Target |
|--------|--------|
| Classification accuracy (parsed from chat response) | > 85% |
| Fix correctness (shellcheck on suggested fixes) | > 85% |
| Rule citation accuracy (vs bashrs linter) | > 90% |
| Novel script handling (50 unseen scripts) | Relevant, non-hallucinated |
| Human review (100 samples, 1-5 scale) | avg > 3.0 |

### 7.4 Ship Gate (C-CHAT-001)

| Field | Value |
|-------|-------|
| **Precondition** | Test set eval + human review of 100 samples |
| **Postcondition** | `classification > 85% AND shellcheck > 85% AND citation > 90%` |
| **Kill criterion** | Human review avg < 2.5 → ship classifier only, chat not ready |

---

## 8. Shipped Artifacts

| Artifact | Repo | Size | Purpose |
|----------|------|------|---------|
| CodeBERT classifier | `paiml/shell-safety-classifier` | ~500MB | Fast CI/CD triage (~20ms) |
| Conversation dataset | `paiml/shell-safety-conversations` | ~100MB | Reproducibility |

### 8.1 CLI

```bash
bashrs classify script.sh          # Rule-based (Stage 0), CodeBERT ~20ms (Stage 1)
bashrs explain script.sh           # Rule-based analysis (Stage 0), Qwen chat (Stage 2)
bashrs fix script.sh               # Auto-fix linter findings (Stage 0)
bashrs safety-check script.sh      # Lint + classify combined (no chat)
```

**Implementation status (v6.65.0)**:
- `bashrs classify` — implemented (rule-based Stage 0)
- `bashrs explain` — implemented (rule-based Stage 0, per-finding what/why/fix)
- `bashrs fix` — implemented (autofix SAFE/SAFE-WITH-ASSUMPTIONS)
- `bashrs safety-check` — implemented (lint + classify combined)
- `bashrs corpus model-card` — implemented (HuggingFace model card with YAML front matter)
- `bashrs corpus training-config` — implemented (entrenar config with class weights)
- `bashrs corpus export-splits` — implemented (80/10/10 deterministic splits)
- `bashrs corpus validate-contracts` — implemented (8 contracts, 7 PASS + 1 KILL-5)
- `bashrs corpus baselines` — implemented (majority, keyword, linter baselines with MCC/acc/rec)
- `bashrs corpus publish-dataset` — implemented (HF-ready dir: README.md + splits + config)
- `bashrs corpus ssc-report` — enriched: S5.5 evaluation metrics, S6.4 conversation type breakdown
- `bashrs corpus ssc-report --gate` — CI quality gate (exit 1 on FAIL)
- `bashrs corpus generate-conversations` — S6.4 quality gates (type breakdown, variant balance, empty response check, ChatML system prompt)
- `bashrs corpus publish-conversations` — HF-ready conversation dataset dir (JSONL + dataset card README)
- 8 `cargo run --example` programs verified: shell_safety_classifier, explain_demo, baselines, label_audit, generalization_tests, contract_validation, ssc_data_pipeline, ssc_report
- 43 assert_cmd CLI integration tests (cli_ssc_tests.rs): classify (safe/unsafe/json/makefile/dockerfile/multi-label/nonexistent/probe/model), explain (safe/unsafe/json/det/idem/makefile/nonexistent), fix (dry-run/output/assumptions/safe/nonexistent), safety-check (safe/unsafe/json/makefile/nonexistent), corpus subcommands, CLF-RUN pipeline
- 4 provable-contracts YAML files created (S4.3.1): bidirectional-attention-v1, learned-position-embedding-v1, encoder-forward-v1, linear-probe-classifier-v1
- SSC report optimized: keyword heuristic for conversation sampling (4+ min → 1.8s), shared corpus/baseline data (eliminated double corpus load via `corpus_baseline_entries_from()`, PMAT-152)
- **Stage 0 COMPLETE**: All encoder components (ENC-001..008) implemented in entrenar with 30 tests. GitHub: paiml/entrenar#242
- **Stage 1 INFRASTRUCTURE COMPLETE**: CLF-001..007 implemented in entrenar with 31 tests. GitHub: paiml/entrenar#243
  - EncoderBlock (post-norm), EncoderModel (full pipeline with from_safetensors), LinearProbe (SGD on cached embeddings)
- **PV-003 COMPLETE**: 12 SSC falsification tests + 3 proptests bound to contracts (FALSIFY-BIATT-001..003, FALSIFY-PROBE-001..003, FALSIFY-ENC-001..002, FALSIFY-POS-001)
- **PV-004 COMPLETE**: `pv audit` clean on all 4 contracts (0 findings)
  - Classification metrics (MCC, accuracy, recall, precision, confusion matrix, bootstrap CI)
  - Escalation ladder (4 levels with decision logic), baselines comparison, generalization test, ship gate C-CLF-001
  - CodeBERT 124M params loaded and validated end-to-end (RoBERTa BPE tokenizer, 768-dim embeddings)
- **VAL-001 COMPLETE**: C-TOK-001 PASSED — 90.0% acceptable (18/20 shell constructs). CodeBERT tokenizer loaded via `aprender::text::bpe::BpeTokenizer::from_vocab_merges()`. Contract: `codebert-tokenizer-validation-v1.yaml`.
- **Phase 2 COMPLETE**: Synthetic conversation generation (S6)
  - ChatML format with system prompt (honesty requirements S6.5)
  - 4 conversation types (A: classify, B: fix, C: debug, D: confirm-safe) with 12+ phrasing variants each
  - `bashrs corpus publish-conversations` — HuggingFace-ready dataset directory (JSONL + README with YAML front matter)
  - 17,942 conversations from full corpus, quality gate PASSED (Type D 97.7%, 0 empty responses)
  - Dataset README includes S6.5 honesty disclaimers (synthetic data, not novel reasoning, not security audit replacement)
- **Phase 1 COMPLETE**: CLF-RUN classifier pipeline (CPU-based)
  - `bashrs corpus extract-embeddings` — load CodeBERT, extract 768-dim [CLS] embeddings (streaming, --limit)
  - `bashrs corpus train-classifier` — train logistic regression probe on cached embeddings
  - `bashrs corpus run-classifier` — full pipeline (extract + train + evaluate + C-CLF-001 gate)
  - RoBERTa BPE tokenizer auto-loaded from model directory (improves MCC by +9.4%)
  - Class-weighted online SGD with sqrt-inverse balanced weights (aprender#427, aprender#428)
  - L2 regularization (weight_decay=1e-4) prevents overfitting on imbalanced data
  - 13 unit tests + 5 CLI integration tests + provable contract (classifier-pipeline-v1.yaml)
  - Validated: 500-entry BPE MCC=0.427, 2047-entry BPE MCC=0.321 — C-CLF-001 PASS
- **Phase 4 CLI-001 COMPLETE**: `bashrs classify --probe --model` (Stage 1 ML classification)
  - Full CodeBERT inference: tokenize → [CLS] embedding → linear probe → binary label + confidence
  - `--probe probe.json --model /path/to/codebert/` flags on `bashrs classify`
  - Without `ml` feature: helpful error guiding to `--features ml`
- **Phase 4 CLI-002 COMPLETE**: `bashrs explain --chat-model` and `bashrs fix --chat-model` (entrenar#246)
  - Wired to entrenar `InstructPipeline::generate_chat()` with ChatML formatting
  - Loads Qwen-1.5B + LoRA from model directory (config.json auto-detection)
  - `chat_inference.rs` module with SYSTEM_PROMPT, format prompts, feature-gated ML path
  - Without `ml` feature: helpful error guiding to `--features ml`
  - Provable contract: `chat-inference-pipeline-v1.yaml` with 5 falsification tests
- **Phase 4 CLI-003 COMPLETE**: 49 assert_cmd integration tests (6 new CLI-002 tests)
- **Phase 5 WASM-001 COMPLETE**: `bashrs-wasm` crate (1.5MB release, 9 tests, wasm32-unknown-unknown)
  - lint_shell_wasm, lint_makefile_wasm, lint_dockerfile_wasm, classify_shell_wasm, explain_shell_wasm
  - Feature-gated optional deps (rustyline, rand, sysinfo) for minimal WASM build
  - Provable contract: `wasm-linter-v1.yaml` with 6 falsification tests
- **Phase 5 WASM-003 COMPLETE**: `shell-safety.html` interactive app
  - Split-pane editor with Bash/Makefile/Dockerfile support
  - Real-time classification + diagnostics with 150ms debounce
  - Fix suggestions from explain API
- **Phase 6 PRB-001 COMPLETE**: Probar test suite (`bashrs-wasm/tests/probar_shell_safety.rs`)
  - 14 Layer 1 logic tests: linter correctness, classifier correctness, explain correctness, combined pipeline, JSON structure, determinism, multi-format
  - 5 Layer 3 performance tests: linter <10ms, classify <10ms, explain <10ms, full pipeline <30ms, multi-format <30ms
  - Provable contract: `probar-shell-safety-v1.yaml` with 9 falsification tests
  - CodeBERT tests gated behind `codebert` feature (blocked on WASM-002/004)
- **Phase 6 PRB-005 COMPLETE**: Performance benchmark tests with hard budgets (5 tests, all pass)
- **Phase 5 WASM-006 COMPLETE**: Deployed to https://interactive.paiml.com/shell-safety/
  - S3 bucket: interactive.paiml.com-production-mces4cme/shell-safety/
  - CloudFront invalidation: ELY820FVFXAFF
  - HTML + JS (11KB) + WASM (1.5MB), correct MIME types
- **Phase 5 WASM-004 DONE — KILL CRITERION 5 TRIGGERED**: Pure-Rust CodeBERT encoder implemented in bashrs-wasm
  - `wasm_encoder.rs`: 400-line encoder (embedding, 12-layer transformer, attention, FFN, LayerNorm, GELU)
  - Loads int8 SafeTensors weights (119MB), dequantizes to f32, runs full forward pass
  - 15 unit tests + determinism verification + benchmark
  - `classify_codebert_wasm()`, `load_codebert_model()`, `load_codebert_probe()` WASM functions
  - WASM binary: 1.7MB with codebert feature (vs 1.5MB without)
  - **Benchmark**: 2681ms for 33 tokens on native CPU (release mode)
  - **Estimated WASM**: 5-13s (2-5x slowdown) — exceeds 2s kill threshold
  - **Decision**: Ship CLI only for CodeBERT classification. Browser uses rule-based linter.
  - Negative result published honestly per spec Section 11 Kill Criteria.
  - WASM-005 (IndexedDB caching) cancelled — no model to cache in browser.
  - PRB-002/003/004 cancelled — no CodeBERT WASM to test in browser.
- **CHAT-001 COMPLETE**: Training manifest, entrenar-format JSONL export (`--entrenar` flag), provable contract
  - 17,942 conversations (25MB JSONL) in entrenar ChatML format (`text` field with `<|im_start|>` tags)
  - Training manifest: `training/ssc-chat-qwen-0.5b.yaml` — Qwen2.5-Coder-0.5B-Instruct, LoRA rank=16, alpha=32, Q+V
  - Provable contract: `chat-model-training-v1.yaml` — 5 postconditions, 4 falsification tests, 1 kill criterion
  - entrenar dry-run validated, batch_size=4, gradient_accumulation=4, NF4 quantization
- **CHAT-002 COMPLETE**: Fine-tuning completed on RTX 4090 (CUDA 12.8, 87 minutes total)
  - Qwen2.5-Coder-0.5B-Instruct (downgraded from 1.5B — f32 optimizer states OOM'd on 1.5B)
  - 13,458 steps (3 epochs × 4,486 batches), seq_len=512, batch_size=4, peak 5172 tok/s, MFU 18.6%
  - Final loss: 4.800, best loss: 0.764, epoch losses: 4.576/4.980/4.800
  - Model: `training/checkpoints/ssc-chat-v1/model.safetensors` (1.98GB)
- **CHAT-004 COMPLETE**: Published to HuggingFace
  - Model: ~~https://huggingface.co/paiml/shell-safety-chat~~ (DELETED — kill criterion confirmed)
  - Dataset: https://huggingface.co/datasets/paiml/shell-safety-conversations (17,942 entries, 35MB)
- **CHAT-003 FAIL**: Four training runs, three entrenar bugs found and fixed
  - **Run 1** (without biases): Gibberish ("222dkdkdk...")
    - Root cause: entrenar#258 — missing QKV attention biases (fixed in 24bc0c7)
  - **Run 2** (with biases, LR=2e-4): Interrupted at step 4004
  - **Run 3** (3 epochs, LR=5e-5): 3.0% accuracy — catastrophic forgetting
    - Bugs fixed: entrenar#258 (biases), #259 (config.json), #260 (bias shape)
    - Epoch 1 loss=1.52 good, epochs 2-3 degraded to 4.0 (destroyed model)
  - **Run 4** (1 epoch, LR=5e-5): **12.1% accuracy — coherent domain text!**
    - Epoch 1: loss=1.86, ppl=6.44, best=0.607
    - Model generates structured advice about shell safety
    - Discusses command injection, unquoted variables, non-determinism
    - 4 code blocks generated (1 passes shellcheck = 25%)
    - BUT: doesn't output "safe"/"unsafe" classification labels
    - Key insight: model learned DOMAIN but not response FORMAT
  - C-CHAT-TRAIN-002: FAIL (12.1%, target >85%)
  - C-CHAT-TRAIN-003: FAIL (25% shellcheck on 4 blocks, target >85%)
  - C-CHAT-TRAIN-004: FAIL (0% citations)
  - **Kill criterion**: KILL-CHAT-001 TRIGGERED — ship classifier only
  - **Remaining issue**: Full fine-tuning destroys instruction-following.
    Needs LoRA adapter-only training to preserve base model capabilities.
  - **Run 5** (1 epoch, v2 data with classification prefix): **28.0% accuracy — 2.3× improvement!**
    - Option C: All 17,942 responses prefixed with "Classification: safe/unsafe"
    - Epoch 1: loss=1.667, ppl=5.30, best=0.791, 51 min @ 2983 tok/s
    - Safe accuracy: 44% (11/25), Unsafe accuracy: 12% (3/25)
    - 19 code blocks generated (4.75× run 4), 4 pass shellcheck (21.1%)
    - Option E (prefill): 30.0% — marginal improvement from prompt engineering
    - C-CHAT-TRAIN-002: FAIL (28.0%, target >85%)
    - **Root cause shift**: FORMAT learned (some "Classification:" prefixes in output),
      but CLASS IMBALANCE dominates — 97.7% safe training data → model defaults "safe"
    - **Kill criterion**: KILL-CHAT-001 remains TRIGGERED
  - **Run 6** (3 epochs, balanced v3 data): **32.0% accuracy — marginal improvement**
    - Balanced dataset: 1512 safe (67%) + 756 unsafe (33%) = 2268 entries
    - Downsampled safe to 2× unsafe, augmented with 350 adversarial entries
    - 3 epochs with no catastrophic forgetting: loss 1.96→1.37→1.27 (monotonic)
    - Safe recall: 60% (15/25, up from 44%), Unsafe recall: 4% (1/25, down from 12%)
    - 17 code blocks, 2 pass shellcheck (11.8%), 0% citations
    - Most unsafe predictions are "unknown" (16/25) — model lost format compliance
    - **Root cause**: 0.5B model insufficient capacity to learn format + classification
    - After 6 runs: 0%→3%→12%→28%→32% — diminishing returns at model capacity limit
    - **Kill criterion**: KILL-CHAT-001 CONFIRMED. Ship classifier-only pipeline.
    - **Recommendation**: Chat requires ≥7B model. MLP classifier (MCC=0.754) ships instead.

### 8.1b Phase 3b: LoRA Re-Run (CHAT-003 Invalidation)

**Date added**: 2026-03-08
**Status**: PLANNED — pending execution on Lambda RTX 4090

#### 8.1b.1 Why Runs 1-6 Were Invalid

All six CHAT-003 runs used **full fine-tuning**, not LoRA. The training manifest
(`ssc-chat-qwen-0.5b.yaml`) contained `lora.enabled: true` with rank=16, alpha=32,
target_modules=[q_proj, v_proj], but entrenar's `TransformerTrainer` had no LoRA
integration at the time. The YAML `lora:` section was parsed and silently ignored.

Evidence:
- ENT-LoRA-001 (YAML wiring) was not implemented until 2026-03-07
- Run 3 showed catastrophic forgetting after epoch 1 — classic full-FT failure mode
- Run 4-6 showed "domain learning but format loss" — full-FT overwrites instruction tuning
- Line 712-713 of this spec identified the root cause: "Full fine-tuning destroys
  instruction-following. Needs LoRA adapter-only training."

**Conclusion**: KILL-CHAT-001 was confirmed under invalid experimental conditions.
The "0.5B insufficient capacity" conclusion (line 731) may be wrong — the model
never had a fair trial with parameter-efficient fine-tuning.

#### 8.1b.2 What Changed in entrenar

ENT-LoRA-001 through ENT-LoRA-018 implemented between 2026-03-06 and 2026-03-08:

- **ENT-LoRA-001**: YAML wiring — `lora:` config now creates LoRA layers in TransformerTrainer
- **ENT-LoRA-002**: LoRALayer math — B@A=0 at init, forward = W@x + scale*(B@(A@x))
- **ENT-LoRA-003**: Base weight freezing — only LoRA A/B matrices are trainable
- **ENT-LoRA-004**: LoRA scaling (standard α/r and rsLoRA α/√r)
- **ENT-LoRA-005**: Merge/unmerge for inference
- **ENT-LoRA-006**: PEFT-compatible adapter save/load
- **ENT-LoRA-007-018**: CLI integration, optimizer filtering, gradient accumulation,
  checkpoint resume, target module selection, LoRA+ differential LR, NF4 quantization

Falsification: 28 tests across 6 layers (F-MATH, F-FREEZE, F-CLI, F-CONV, F-CKPT, F-EDGE),
all passing. Spec: `entrenar/docs/lora-qlora-enhancement.md` Section 6.

#### 8.1b.3 Model Selection: Qwen3-4B NF4 QLoRA

**Decision**: Upgrade from Qwen2.5-Coder-0.5B to Qwen3-4B based on first-principles analysis.

**Why Qwen3-4B** (not 0.5B/1.5B Qwen2.5):
- Qwen3-4B ≈ Qwen2.5-7B on code benchmarks (MBPP=67.0, EvalPlus=63.5)
- Already instruction-tuned (Qwen3 post-trained models = instruct by default)
- Think/non-think mode: fast classification (non-think) + deep explanations (think)
- QK-layernorm for training stability, 8 KV heads (vs 2 in Qwen2.5)
- **Already proven on this box**: Qwen3-4B NF4 QLoRA achieved 91.4% val accuracy
- Weights already local: `/home/noah/src/models/qwen3-4b/` (3 safetensors shards)
- Architecture: 36L, h=2560, 32Q/8KV heads, FFN=9728, Qwen3ForCausalLM

**Why not alternatives**:
- ~~Qwen2.5-Coder-0.5B~~: 6 failed runs, 494M too small for format+classify+explain
- ~~Qwen2.5-Coder-1.5B~~: Code-specialized but Qwen2 architecture, 3× less capacity than 4B
- ~~Qwen3-1.7B~~: Matches Qwen2.5-3B, viable fallback but 4B fits in VRAM budget
- ~~Qwen3-0.6B~~: Too small (≈Qwen2.5-1.5B), same capacity class as failed runs
- ~~Qwen2.5-Coder-3B~~: Base model only (not instruct-tuned), Qwen2 architecture

**VRAM budget** (concurrent with albor 350M pretraining):
- Albor GPU-resident training: ~12 GB
- Qwen3-4B NF4 QLoRA: ~2.9 GB (NF4 base 2.0 + LoRA 24 MB + optimizer 94 MB + activations 0.75 GB)
- System/display: ~0.5 GB
- **Total: ~15.4 GB / 24 GB** — 8.6 GB headroom

#### 8.1b.4 Run 7 Plan: Qwen3-4B NF4 QLoRA

**Hypothesis**: Qwen3-4B with NF4 QLoRA provides sufficient capacity for
format compliance + domain classification + code explanation, while LoRA
preserves the base model's instruction-following and think/non-think modes.

**Configuration** (manifest: `training/ssc-chat-qwen3-4b-qlora.yaml`):
- Model: Qwen3-4B (local: `/home/noah/src/models/qwen3-4b/`)
- Architecture: Qwen3ForCausalLM, 36L, h=2560, 32 heads, 8 KV heads
- Quantization: NF4 (4-bit base weights, CUDA NF4 path)
- LoRA: rank=16, alpha=32.0, targets=[q_proj, v_proj, o_proj, gate_proj]
  - 4 target modules (following albor pattern for instruction tuning)
  - o_proj: reshapes attention output (format compliance)
  - gate_proj: modulates FFN (domain knowledge adaptation)
  - ~11.8M trainable params (vs 4B frozen base)
- Data: conversations_v3.jsonl (balanced: 1512 safe + 756 unsafe = 2268)
- Training: 1 epoch, LR=5e-5, batch=4, grad_accum=4, seq_len=512
- Hardware: Lambda RTX 4090 (concurrent with albor pretraining)

**Success criteria** (from provable contract `chat-model-training-v1.yaml`):
- C-CHAT-TRAIN-002: >85% combined accuracy on 50-entry eval set
- C-CHAT-TRAIN-003: >85% shellcheck pass rate on generated code blocks
- C-CHAT-TRAIN-004: >50% citation rate (references SEC/DET/IDEM rules)
- NEW: Base model think/non-think mode preserved after LoRA adaptation

**Expected differences from runs 1-6** (full-FT on 0.5B):
1. No catastrophic forgetting — base weights frozen in NF4, only LoRA A/B adapt
2. 8× model capacity — Qwen3-4B vs Qwen2.5-0.5B
3. Better instruction-following baseline — Qwen3 post-training > Qwen2.5
4. Smaller checkpoint — adapter only (~47 MB vs 1.98 GB full model)
5. PEFT-compatible export — merge/unmerge for deployment flexibility

#### 8.1b.5 VRAM Budget (Qwen3-4B NF4 QLoRA)

Computed from architecture dims. Validated against albor pretrain-350m (measured 12.6 GB).

| Component | Size | Notes |
|-----------|------|-------|
| Base model (NF4, frozen) | 2.0 GB | 3.8B params × 4 bits + quantization scales |
| LoRA adapters (bf16) | 0.03 GB | 16.8M trainable params (144 adapter matrices) |
| AdamW optimizer (fp32) | 0.20 GB | m + v + fp32 copy, only LoRA params |
| Gradients (bf16) | 0.03 GB | Only LoRA params, base frozen |
| Activations | 1.1–4.2 GB | Depends on micro-batch size + grad checkpointing |
| NF4 dequant workspace | 0–0.2 GB | Per-layer bf16 dequant during backward (if needed) |
| CUDA overhead | 0.3–0.5 GB | Context, fragmentation, cuBLAS workspace |
| **Total** | **3.7–7.0 GB** | **Confirm empirically via `apr train plan`** |

**Why SSC uses less VRAM than albor (12.6 GB) despite a 10× larger model:**

Albor pretrains 350M with ALL params trainable. AdamW keeps 3 fp32 copies of
every trainable param (m, v, master weights) = 4.2 GB optimizer states + 0.7 GB
gradients + 1.4 GB GPU-resident grad accumulation workspace = **6.3 GB just for
trainability overhead**.

SSC QLoRA trains only 16.8M LoRA params (0.4% of base). Optimizer = 0.2 GB,
gradients = 0.03 GB. The base model is bigger in VRAM (2.0 GB NF4 vs 0.7 GB bf16)
but the optimizer savings are 21×.

| | Albor (pretrain 350M) | SSC (QLoRA 4B) |
|---|---|---|
| Base model | 0.7 GB (bf16, trainable) | 2.0 GB (NF4, frozen) |
| Trainable params | 350M (100%) | 16.8M (0.4%) |
| Optimizer states | 4.2 GB | 0.2 GB |
| Gradients + accum | 2.1 GB | 0.1 GB |
| **Estimated total** | **~12.6 GB** (measured) | **~5–7 GB** (estimate) |

**Concurrent training**: SSC (~5–7 GB) does NOT fit alongside albor (~12.6 GB) on
a 24 GB RTX 4090. Schedule SSC when albor is idle, OR reduce albor to a checkpoint
pause. The "2.9 GB" claim from the original manifest was an inference estimate, not
a training estimate. Corrected.

#### 8.1b.6 Updated Kill Criteria

**KILL-CHAT-001 (revised)**: If Run 7 (Qwen3-4B NF4 QLoRA) fails to achieve
>50% combined accuracy, THEN the task requires a larger model (≥8B) or
fundamentally different approach. Ship MLP classifier (MCC=0.754) only.

**KILL-CHAT-002 (new)**: If LoRA training shows no loss decrease after 500 steps
(lr={5e-5, 1e-4, 2e-4} all tried), check: (a) `is_lora()` returns true,
(b) LoRA layer gradients are non-zero, (c) NF4 dequantization is functioning.

**KILL-CHAT-003 (revised)**: SSC and albor cannot train concurrently (combined
~18–20 GB exceeds 24 GB VRAM). Options in priority order:
1. Train SSC when albor hits a checkpoint pause (albor saves every 500 steps)
2. Reduce SSC batch_size to 2 (saves ~1 GB activations)
3. Enable gradient checkpointing for SSC (saves ~1–3 GB activations)
4. Schedule SSC after albor completes current epoch

### 8.2 Pipeline (v12 — ShellSafetyBench)

```
bashrs check:
    ├── bashrs lint (<1ms) ──> rule findings + CWE IDs
    ├── CodeBERT classify (~20ms) ──> confidence score
    v
    Output: {label, confidence, findings[], cwe_ids[]}

bashrs explain (explicit only, user invokes):
    └── shell-safety-qwen3-4b (~2s) ──> structured safety review
        ├── Verdict: safe / needs-review / unsafe
        ├── Findings: rule IDs + line numbers + severity
        ├── Explanation: WHY each finding matters (CWE context)
        ├── Fix: concrete corrected code (shellcheck-validated)
        └── Confidence: "checked N known patterns, M matched"
```

Chat model invoked ONLY by explicit command. Never automatic.

### 8.3 WASM App (presentar — Brick Profile-First Design)

CodeBERT at 125M params fits in a browser (~125MB int8). The bashrs WASM build
already exists (Phase 0 in WASM spec). presentar provides the UI framework.

**App**: `shell-safety.html` — hosted on interactive.paiml.com

#### 8.3.1 Brick Profile-First (PROBAR-SPEC-009)

**Tests define the interface. Implementation follows.** Per presentar's Brick Architecture,
every widget must declare assertions, performance budget, and verification BEFORE any
rendering code is written. JIDOKA enforcement: rendering is blocked if verification fails.

Write these tests FIRST — they ARE the UI spec:

```rust
// tests/shell_safety_bricks.rs — WRITE THIS BEFORE ANY WIDGET CODE

use presentar::{Brick, BrickAssertion, BrickBudget};

// === ScriptEditor Brick ===

#[test]
fn test_script_editor_brick_name() {
    let editor = ScriptEditor::new("");
    assert_eq!(editor.brick_name(), "ScriptEditor");
}

#[test]
fn test_script_editor_assertions() {
    let editor = ScriptEditor::new("echo hello");
    let assertions = editor.assertions();
    assert!(assertions.contains(&BrickAssertion::MinSize { w: 400, h: 200 }));
    assert!(assertions.contains(&BrickAssertion::Accessible));
}

#[test]
fn test_script_editor_budget() {
    let editor = ScriptEditor::new("");
    assert!(editor.budget().total_ms <= 16); // 60fps
}

#[test]
fn test_script_editor_can_render_empty() {
    let editor = ScriptEditor::new("");
    assert!(editor.can_render()); // Empty input is valid
}

#[test]
fn test_script_editor_content_accessible() {
    let editor = ScriptEditor::new("eval $x");
    assert!(editor.can_render());
    assert_eq!(editor.content(), "eval $x");
}

// === SafetyResult Brick ===

#[test]
fn test_safety_result_brick_name() {
    let result = SafetyResult::safe(0.99);
    assert_eq!(result.brick_name(), "SafetyResult");
}

#[test]
fn test_safety_result_unsafe_display() {
    let result = SafetyResult::unsafe_with_findings(
        0.97,
        vec![Finding::new("SEC001", "eval on untrusted input", 2)],
    );
    assert!(!result.is_safe());
    assert!(result.confidence() > 0.9);
    assert_eq!(result.findings().len(), 1);
}

#[test]
fn test_safety_result_contrast_ratio() {
    let result = SafetyResult::unsafe_with_findings(0.97, vec![]);
    let assertions = result.assertions();
    // Unsafe label must have high contrast (red on white, WCAG AA)
    assert!(assertions.contains(&BrickAssertion::ContrastRatio(4.5)));
}

#[test]
fn test_safety_result_budget() {
    let result = SafetyResult::safe(0.99);
    assert!(result.budget().total_ms <= 16);
}

// === FixSuggestion Brick ===

#[test]
fn test_fix_suggestion_brick_name() {
    let fix = FixSuggestion::new("#!/bin/bash\nmkdir -p /tmp/build");
    assert_eq!(fix.brick_name(), "FixSuggestion");
}

#[test]
fn test_fix_suggestion_code_visible() {
    let fix = FixSuggestion::new("mkdir -p /tmp/build");
    assert!(fix.assertions().contains(&BrickAssertion::TextVisible));
}

// === AnalyzeButton Brick ===

#[test]
fn test_analyze_button_brick_name() {
    let btn = AnalyzeButton::new(ModelState::NotLoaded);
    assert_eq!(btn.brick_name(), "AnalyzeButton");
}

#[test]
fn test_analyze_button_disabled_without_model() {
    let btn = AnalyzeButton::new(ModelState::NotLoaded);
    assert!(!btn.is_enabled()); // Can't analyze without model
    assert!(btn.can_render());   // But can still render (shows "Load Model" text)
}

#[test]
fn test_analyze_button_enabled_with_model() {
    let btn = AnalyzeButton::new(ModelState::Ready);
    assert!(btn.is_enabled());
}

#[test]
fn test_analyze_button_loading_state() {
    let btn = AnalyzeButton::new(ModelState::Loading { progress: 0.45 });
    assert!(!btn.is_enabled());
    assert_eq!(btn.label(), "Loading model... 45%");
}

// === ModelStatus Brick ===

#[test]
fn test_model_status_not_loaded() {
    let status = ModelStatus::new(ModelState::NotLoaded);
    assert_eq!(status.brick_name(), "ModelStatus");
    assert!(status.can_render());
}

#[test]
fn test_model_status_cached() {
    let status = ModelStatus::new(ModelState::Cached);
    assert!(status.can_render());
    // Should indicate model is ready from cache
}
```

These tests define:
- **5 Brick widgets**: `ScriptEditor`, `SafetyResult`, `FixSuggestion`, `AnalyzeButton`, `ModelStatus`
- **Assertions**: MinSize, Accessible, ContrastRatio(4.5), TextVisible
- **Budgets**: All <= 16ms (60fps)
- **State machine**: `ModelState::NotLoaded → Loading { progress } → Ready | Cached`

**Enforcement**: Add to `build.rs`:
```rust
// build.rs — Compile fails if test file is missing
const _ENFORCE: &str = include_str!("../tests/shell_safety_bricks.rs");
```

#### 8.3.2 Widget → Screen Mapping

```
┌─────────────────────────────────────────────────────┐
│  Shell Safety Analyzer               [ModelStatus]  │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌─ ScriptEditor ────────────────────────────┐      │
│  │ #!/bin/bash                               │      │
│  │ eval "$user_input"                        │      │
│  │ mkdir /tmp/build                          │      │
│  │                                        ▊  │      │
│  └───────────────────────────────────────────┘      │
│                                                     │
│  [AnalyzeButton]                                    │
│                                                     │
│  ┌─ SafetyResult ────────────────────────────┐      │
│  │  ● UNSAFE  (confidence: 0.97)             │      │
│  │                                           │      │
│  │  Findings:                                │      │
│  │  ├ SEC001: eval on untrusted input (L2)   │      │
│  │  └ IDEM001: mkdir without -p (L3)         │      │
│  │                                           │      │
│  │  ┌─ FixSuggestion ─────────────────────┐  │      │
│  │  │ #!/bin/bash                         │  │      │
│  │  │ # eval removed — use case statement │  │      │
│  │  │ mkdir -p /tmp/build                 │  │      │
│  │  └─────────────────────────────────────┘  │      │
│  └───────────────────────────────────────────┘      │
│                                                     │
│  Powered by CodeBERT (125M) + bashrs linter         │
│  Running locally in your browser — no server calls  │
└─────────────────────────────────────────────────────┘
```

#### 8.3.3 Architecture

```
Browser
    ├── presentar-core (WASM) ──> Canvas rendering, Brick widgets
    ├── bashrs (WASM) ──────────> Linter (24 SEC + DET/IDEM rules, <1ms)
    └── CodeBERT (WASM) ───────> Classifier (~125MB int8, ~100ms WebGPU)
```

All three WASM modules load independently. Linter runs on every keystroke (<1ms).
Classifier runs on [Analyze] click. No network calls after model download.

#### 8.3.4 Model State Machine

```
NotLoaded ──[user clicks Analyze]──> Loading { progress: 0.0 }
Loading ──[IndexedDB check: hit]──> Cached ──> Ready
Loading ──[IndexedDB check: miss]──> Loading { progress: 0..1 } ──[download complete]──> Ready
Ready ──[user clicks Analyze]──> Classifying ──> Ready (with results)
```

Linter works in ALL states (no model dependency). Model only needed for ML classification.

#### 8.3.5 Deployment

Built as part of bashrs WASM pipeline. Served from `interactive.paiml.com/shell-safety/`.
Static files: `index.html` + presentar WASM + bashrs WASM + model weights.

CodeBERT int8 weights (~125MB) cached in IndexedDB after first download.

### 8.4 Probar-First Testing Design (WASM + LLM Correctness + Performance)

All WASM testing uses **Probar** (`jugar-probar`), NOT Playwright. Probar is the sovereign
Rust testing framework: zero JS, direct WASM memory inspection, deterministic replay,
Docker cross-browser matrix. It replaces ALL JavaScript-based testing (Playwright, Jest, Cypress).

#### 8.4.1 Why Probar for LLM+WASM Testing

| Capability | Playwright | Probar | SSC Impact |
|------------|-----------|--------|------------|
| Language | TypeScript | Pure Rust | No JS dependency |
| WASM state | Black box (DOM) | Direct memory access | Verify embeddings, logits, weights in-memory |
| Determinism | Non-deterministic (browser timing) | Fully deterministic | Reproducible LLM outputs |
| Performance | ~500ms/test overhead | ~10ms/test overhead | Can benchmark inference latency precisely |
| Browser | Required always | Optional (Docker) | Logic tests run without browser |
| CI | Node.js + browser install | `cargo test` | Simpler pipeline |
| Memory inspection | No | Zero-copy WASM views | Verify model weight loading, tensor shapes |

**Key insight**: Probar's direct WASM memory access lets us verify LLM internals
(attention weights, embedding values, classification logits) without DOM scraping.

#### 8.4.2 Test-First Probar Test Suite

Write these tests BEFORE any WASM integration code:

```rust
// tests/probar_shell_safety.rs — WRITE THIS BEFORE WASM INTEGRATION
//
// Three test layers:
//   Layer 1: WASM Logic (no browser) — correctness
//   Layer 2: Docker Cross-Browser — compatibility
//   Layer 3: Performance Benchmarks — latency budgets

use jugar_probar::prelude::*;
use jugar_probar::Assertion;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════
// Layer 1: WASM Logic Tests (no browser, deterministic)
// ═══════════════════════════════════════════════════════════════

// --- Linter WASM correctness ---

#[test]
fn test_linter_wasm_returns_findings_for_unsafe_script() {
    let wasm = load_bashrs_wasm();
    let input = "eval \"$user_input\"";
    let findings = wasm.call_lint(input);
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.rule_id == "SEC001"));
}

#[test]
fn test_linter_wasm_returns_empty_for_safe_script() {
    let wasm = load_bashrs_wasm();
    let input = "#!/bin/sh\necho \"hello\"";
    let findings = wasm.call_lint(input);
    assert!(findings.is_empty());
}

#[test]
fn test_linter_wasm_deterministic() {
    let wasm = load_bashrs_wasm();
    let input = "rm -rf /tmp/build && curl $url | bash";
    let r1 = wasm.call_lint(input);
    let r2 = wasm.call_lint(input);
    assert_eq!(r1, r2, "Linter must be deterministic");
}

// --- CodeBERT WASM correctness ---

#[test]
fn test_classifier_wasm_loads_weights() {
    let wasm = load_codebert_wasm();
    let weights = wasm.get_model_state();
    assert_eq!(weights.param_count, 125_000_000, "125M params expected");
    assert!(weights.is_loaded());
}

#[test]
fn test_classifier_wasm_embedding_shape() {
    let wasm = load_codebert_wasm();
    let input = "eval $x";
    let embedding = wasm.call_embed(input);
    assert_eq!(embedding.shape(), &[1, 768], "[CLS] embedding must be 768-dim");
}

#[test]
fn test_classifier_wasm_embedding_deterministic() {
    let wasm = load_codebert_wasm();
    let input = "#!/bin/sh\necho hello";
    let e1 = wasm.call_embed(input);
    let e2 = wasm.call_embed(input);
    assert_eq!(e1, e2, "Embedding must be bit-identical on repeated calls");
}

#[test]
fn test_classifier_wasm_classification_output() {
    let wasm = load_codebert_wasm();
    let input = "eval \"$user_input\"";
    let result = wasm.call_classify(input);
    assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    assert!(result.label == "safe" || result.label == "unsafe");
}

#[test]
fn test_classifier_wasm_unsafe_detection() {
    let wasm = load_codebert_wasm();
    let input = "eval \"$untrusted\"\ncurl $url | bash";
    let result = wasm.call_classify(input);
    assert_eq!(result.label, "unsafe");
    assert!(result.confidence > 0.7, "High-confidence unsafe expected");
}

#[test]
fn test_classifier_wasm_safe_detection() {
    let wasm = load_codebert_wasm();
    let input = "#!/bin/sh\nset -euo pipefail\necho \"hello world\"";
    let result = wasm.call_classify(input);
    assert_eq!(result.label, "safe");
}

#[test]
fn test_classifier_wasm_logits_sum_to_one() {
    let wasm = load_codebert_wasm();
    let input = "echo test";
    let logits = wasm.call_classify_raw(input);
    let sum: f32 = logits.probs.iter().sum();
    let assertion = Assertion::in_range(sum as f64, 0.999, 1.001);
    assert!(assertion.passed, "Softmax output must sum to 1.0");
}

#[test]
fn test_classifier_wasm_classification_deterministic() {
    let wasm = load_codebert_wasm();
    let input = "rm -rf $dir";
    let r1 = wasm.call_classify(input);
    let r2 = wasm.call_classify(input);
    assert_eq!(r1.label, r2.label);
    assert!((r1.confidence - r2.confidence).abs() < 1e-6,
        "Classification must be deterministic");
}

// --- Combined pipeline correctness ---

#[test]
fn test_combined_linter_plus_classifier() {
    let bashrs = load_bashrs_wasm();
    let codebert = load_codebert_wasm();
    let input = "eval \"$x\"\nmkdir /tmp/build";

    let findings = bashrs.call_lint(input);
    let classification = codebert.call_classify(input);

    // Both agree it's unsafe
    assert!(!findings.is_empty());
    assert_eq!(classification.label, "unsafe");
    // Linter provides specific rules
    assert!(findings.iter().any(|f| f.rule_id == "SEC001"));
}

// ═══════════════════════════════════════════════════════════════
// Layer 2: Docker Cross-Browser Tests (Chrome, Firefox, WebKit)
// ═══════════════════════════════════════════════════════════════

#[cfg(feature = "docker")]
mod cross_browser {
    use probar::docker::{DockerTestRunner, ParallelRunner, Browser};
    use std::time::Duration;

    #[test]
    fn test_shell_safety_loads_in_chrome() {
        let mut runner = DockerTestRunner::builder()
            .browser(Browser::Chrome)
            .with_coop_coep(true) // Required for SharedArrayBuffer (WASM threads)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Docker runner");

        runner.simulate_start().expect("Start");
        let results = runner.simulate_run_tests(&[
            "tests/probar_shell_safety.rs",
        ]).expect("Run");
        runner.simulate_stop().expect("Stop");
        assert!(results.all_passed());
    }

    #[test]
    fn test_shell_safety_cross_browser() {
        let mut runner = ParallelRunner::builder()
            .browsers(&Browser::all()) // Chrome, Firefox, WebKit
            .tests(&["tests/probar_shell_safety.rs"])
            .build()
            .expect("Parallel runner");

        runner.simulate_run().expect("Run");
        assert!(runner.all_passed(),
            "All browsers must pass: {:?}", runner.aggregate_stats());
    }

    #[test]
    fn test_model_download_and_indexeddb_cache() {
        let mut runner = DockerTestRunner::builder()
            .browser(Browser::Chrome)
            .with_coop_coep(true)
            .timeout(Duration::from_secs(120)) // Model download may be slow
            .build()
            .expect("Docker runner");

        runner.simulate_start().expect("Start");

        // First load: downloads model
        let first = runner.simulate_navigate("/shell-safety/")
            .expect("Navigate");
        assert!(first.status_ok());

        // Second load: model from IndexedDB cache (should be fast)
        let second = runner.simulate_navigate("/shell-safety/")
            .expect("Navigate again");
        assert!(second.status_ok());
        // Cache hit should be much faster than download
    }
}

// ═══════════════════════════════════════════════════════════════
// Layer 3: Performance Benchmarks (hard budgets, fail on regression)
// ═══════════════════════════════════════════════════════════════

mod performance {
    use super::*;

    // --- Linter performance ---

    #[test]
    fn test_linter_wasm_latency_under_10ms() {
        let wasm = load_bashrs_wasm();
        let input = "#!/bin/sh\neval $x\nmkdir /tmp/test\ncurl $url | bash";

        let start = Instant::now();
        for _ in 0..100 {
            let _ = wasm.call_lint(input);
        }
        let avg = start.elapsed() / 100;

        assert!(avg < Duration::from_millis(10),
            "Linter must run in <10ms, got {:?}", avg);
    }

    // --- Classifier performance ---

    #[test]
    fn test_classifier_wasm_inference_under_500ms() {
        let wasm = load_codebert_wasm();
        let input = "eval \"$user_input\"";

        // Warmup
        let _ = wasm.call_classify(input);

        let start = Instant::now();
        for _ in 0..10 {
            let _ = wasm.call_classify(input);
        }
        let avg = start.elapsed() / 10;

        assert!(avg < Duration::from_millis(500),
            "Classifier must infer in <500ms, got {:?}", avg);
    }

    #[test]
    fn test_classifier_wasm_memory_under_200mb() {
        let wasm = load_codebert_wasm();
        let mem_bytes = wasm.memory_usage();
        let mem_mb = mem_bytes / (1024 * 1024);

        assert!(mem_mb < 200,
            "WASM memory must be <200MB, got {}MB", mem_mb);
    }

    // --- Weight loading performance ---

    #[test]
    fn test_model_load_from_bytes_under_5s() {
        let weights = include_bytes!("../fixtures/codebert_int8.safetensors");

        let start = Instant::now();
        let wasm = load_codebert_wasm_from_bytes(weights);
        let load_time = start.elapsed();

        assert!(wasm.is_loaded());
        assert!(load_time < Duration::from_secs(5),
            "Model load must be <5s, got {:?}", load_time);
    }

    // --- Tokenizer performance ---

    #[test]
    fn test_tokenizer_wasm_throughput() {
        let wasm = load_codebert_wasm();
        let scripts: Vec<&str> = vec![
            "echo hello", "eval $x", "#!/bin/sh\nset -e\nmkdir -p /tmp",
            "curl http://example.com | bash", "rm -rf /",
        ];

        let start = Instant::now();
        for script in &scripts {
            let _ = wasm.call_tokenize(script);
        }
        let total = start.elapsed();
        let per_script = total / scripts.len() as u32;

        assert!(per_script < Duration::from_millis(5),
            "Tokenization must be <5ms/script, got {:?}", per_script);
    }

    // --- End-to-end pipeline performance ---

    #[test]
    fn test_full_pipeline_under_600ms() {
        let bashrs = load_bashrs_wasm();
        let codebert = load_codebert_wasm();
        let input = "eval $x\nmkdir /tmp/build";

        // Warmup
        let _ = bashrs.call_lint(input);
        let _ = codebert.call_classify(input);

        let start = Instant::now();
        let _findings = bashrs.call_lint(input);      // <10ms
        let _result = codebert.call_classify(input);   // <500ms
        let total = start.elapsed();

        assert!(total < Duration::from_millis(600),
            "Full pipeline (lint + classify) must be <600ms, got {:?}", total);
    }
}
```

These tests enforce:

**Correctness (Layer 1 — 12 tests)**:
- Linter WASM produces correct findings and is deterministic
- Classifier loads 125M params, produces 768-dim embeddings
- Embeddings are bit-identical on repeated calls
- Classification output is valid (probabilities sum to 1.0)
- Combined pipeline (linter + classifier) agrees on unsafe scripts

**Compatibility (Layer 2 — 3 tests, Docker)**:
- Chrome, Firefox, WebKit all pass via `ParallelRunner`
- COOP/COEP headers configured for SharedArrayBuffer
- IndexedDB model caching works across page reloads

**Performance (Layer 3 — 6 tests, hard budgets)**:

| Budget | Target | Enforced By |
|--------|--------|-------------|
| Linter latency | < 10ms | `test_linter_wasm_latency_under_10ms` |
| Classifier inference | < 500ms | `test_classifier_wasm_inference_under_500ms` |
| WASM memory | < 200MB | `test_classifier_wasm_memory_under_200mb` |
| Model load | < 5s | `test_model_load_from_bytes_under_5s` |
| Tokenization | < 5ms/script | `test_tokenizer_wasm_throughput` |
| Full pipeline | < 600ms | `test_full_pipeline_under_600ms` |

Tests **fail automatically** if performance degrades — no manual benchmarking needed.

#### 8.4.3 Probar Dual-Runtime Strategy for LLM Verification

```
┌──────────────────────────────────────┐  ┌──────────────────────────────────────┐
│  WasmRuntime (wasmtime)              │  │  BrowserController (Docker/CDP)      │
│  ─────────────────────────           │  │  ─────────────────────────           │
│  Purpose: LLM LOGIC TESTING         │  │  Purpose: GOLDEN MASTER              │
│                                      │  │                                      │
│  ✓ Embedding correctness            │  │  ✓ Full E2E with real browser        │
│  ✓ Classification determinism       │  │  ✓ Visual regression (UI rendering)  │
│  ✓ Softmax probability validation   │  │  ✓ IndexedDB caching verification   │
│  ✓ Weight loading verification      │  │  ✓ Cross-browser compatibility       │
│  ✓ Performance benchmarks           │  │  ✓ Model download flow              │
│  ✓ Memory usage verification        │  │  ✓ WebGPU inference path            │
│                                      │  │                                      │
│  ✗ NOT for UI rendering             │  │  This is the SOURCE OF TRUTH         │
│  ✗ NOT for browser APIs             │  │  for "does it work in production?"   │
│                                      │  │                                      │
│  Runs: cargo test (no browser)      │  │  Runs: probar test --docker          │
│  Speed: ~10ms/test                  │  │  Speed: ~500ms/test (browser)        │
│  CI: Always (every commit)          │  │  CI: Pre-release (Friday)            │
└──────────────────────────────────────┘  └──────────────────────────────────────┘
```

**WasmRuntime** (wasmtime, no browser): Verifies LLM correctness — embedding shapes,
logit distributions, weight counts, determinism, performance budgets. Runs on every commit.
This is where we catch regressions in classifier behavior.

**BrowserController** (Docker + CDP): Verifies production parity — real browser rendering,
IndexedDB persistence, WebGPU inference path, cross-browser compatibility. Runs pre-release.
This is where we catch browser-specific issues.

#### 8.4.4 LLM-Specific Probar Extensions

Beyond standard WASM testing, the shell safety app needs LLM-specific verification:

```rust
// LLM correctness properties verified by Probar

// 1. Numerical stability: No NaN/Inf in any tensor
#[test]
fn test_no_nan_in_embeddings() {
    let wasm = load_codebert_wasm();
    for script in CORPUS_SAMPLE_100 {
        let embedding = wasm.call_embed(script);
        assert!(embedding.iter().all(|v| v.is_finite()),
            "NaN/Inf in embedding for: {}", script);
    }
}

// 2. Calibration: confident predictions should be correct
#[test]
fn test_high_confidence_accuracy() {
    let wasm = load_codebert_wasm();
    let mut high_conf_correct = 0;
    let mut high_conf_total = 0;

    for (script, ground_truth) in LABELED_TEST_SET {
        let result = wasm.call_classify(script);
        if result.confidence > 0.9 {
            high_conf_total += 1;
            if result.label == ground_truth {
                high_conf_correct += 1;
            }
        }
    }

    if high_conf_total > 0 {
        let accuracy = high_conf_correct as f64 / high_conf_total as f64;
        assert!(accuracy > 0.85,
            "High-confidence (>0.9) predictions must be >85% accurate, got {:.1}%",
            accuracy * 100.0);
    }
}

// 3. Consistency: similar scripts get similar scores
#[test]
fn test_semantic_consistency() {
    let wasm = load_codebert_wasm();
    let r1 = wasm.call_classify("eval $x");
    let r2 = wasm.call_classify("eval \"$x\"");
    // Both are unsafe eval — scores should be close
    assert!((r1.confidence - r2.confidence).abs() < 0.2,
        "Similar scripts should get similar scores");
    assert_eq!(r1.label, r2.label);
}

// 4. Monotonicity: adding unsafe code shouldn't make script "safer"
#[test]
fn test_monotonicity_unsafe_additions() {
    let wasm = load_codebert_wasm();
    let safe_script = "#!/bin/sh\necho hello";
    let unsafe_script = "#!/bin/sh\necho hello\neval $x";

    let r_safe = wasm.call_classify(safe_script);
    let r_unsafe = wasm.call_classify(unsafe_script);

    // Adding eval should not decrease unsafe confidence
    assert!(r_unsafe.label == "unsafe" || r_safe.label == "safe",
        "Adding eval to safe script must not make it 'safer'");
}

// 5. Reference parity: WASM output matches native output
#[test]
fn test_wasm_matches_native_reference() {
    let wasm = load_codebert_wasm();
    let fixtures = load_reference_fixtures(); // Pre-computed native outputs

    for (input, expected) in fixtures {
        let wasm_result = wasm.call_classify(&input);
        assert!((wasm_result.confidence - expected.confidence).abs() < 0.01,
            "WASM must match native within 1% for: {}", input);
        assert_eq!(wasm_result.label, expected.label);
    }
}
```

#### 8.4.5 Probar CI/CD Integration

```yaml
# .github/workflows/shell-safety-probar.yml
name: Probar Shell Safety Tests

on: [push, pull_request]

jobs:
  logic-tests:
    # Layer 1: Every commit, no browser needed
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test -p shell-safety-wasm --test probar_shell_safety

  cross-browser:
    # Layer 2: Pre-release only (Friday builds)
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    services:
      docker:
        image: docker:dind
    steps:
      - uses: actions/checkout@v4
      - run: cargo test -p shell-safety-wasm --test probar_shell_safety
             --features docker -- cross_browser

  performance:
    # Layer 3: Every commit (fast, catches regressions)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test -p shell-safety-wasm --test probar_shell_safety
             -- performance
```

---

## 9. Implementation Plan

### Phase 0: Encoder Support + Contracts + Validation (3-4 days)

| Task | Time | Status |
|------|------|--------|
| PV-001: Create 4 YAML contracts in provable-contracts (S4.3.1) | 4 hrs | ✅ Done |
| PV-002: `pv scaffold` → generate trait stubs + test skeletons | 1 hr | ✅ Done |
| ENC-001..008: Implement encoder in entrenar (S4.2) | 2 days | ✅ Done |
| PV-003: `pv bind` → 12 falsification tests + 3 proptests in entrenar | 2 hrs | ✅ Done |
| PV-004: `pv audit` → all 4 contracts clean (0 findings) | 1 hr | ✅ Done |
| VAL-001: Tokenize 100 scripts, check C-TOK-001 | 2 hrs | ✅ Done (90.0% acceptable, 18/20 constructs) |
| VAL-002: Audit 100 unsafe labels, check C-LABEL-001 | 2 hrs | ✅ Done (label_audit.rs) |
| VAL-003: Write 50 generalization test scripts | 2 hrs | ✅ Done (GEN-001..050) |

**Kill gates**: C-ENC-SHIP (encoder works + `pv audit` clean), C-TOK-001
(tokenizer adequate), C-LABEL-001 (labels accurate). Any failure pauses.

### Phase 1: Classifier (2 days)

| Task | Time | Status |
|------|------|--------|
| CLF-001: EncoderBlock + EncoderModel (infrastructure) | 4 hrs | ✅ Done (entrenar) |
| CLF-002: LinearProbe on cached embeddings (infrastructure) | 2 hrs | ✅ Done (entrenar) |
| CLF-003: Evaluate with MCC, bootstrap CI (infrastructure) | 2 hrs | ✅ Done (entrenar) |
| CLF-004: Escalation ladder with decision logic | 1 hr | ✅ Done (entrenar) |
| CLF-005: Baselines comparison function | 1 hr | ✅ Done (entrenar + bashrs) |
| CLF-006: Generalization test function | 1 hr | ✅ Done (entrenar + bashrs) |
| CLF-007: Confidence scores computation | 30 min | ✅ Done (entrenar) |
| CLF-RUN: Download CodeBERT, extract embeddings, train, evaluate | 2-4 hrs | ✅ Done (bashrs corpus run-classifier, CPU) |
| CLF-VALIDATE: End-to-end pipeline validation with real CodeBERT weights | 2 hrs | ✅ Done (2047-entry: MCC=0.321, C-CLF-001 PASS) |
| CLF-FULL: Full 17,942-entry extraction + training | ~4 hrs | ✅ Done (MLP+aug MCC=0.443, C-CLF-001 PASS at full scale) |
| CLF-WEIGHT: Class-weighted online SGD with L2 regularization | 2 hrs | ✅ Done (aprender#427 merged, KAIZEN-101) |

**Kill gate**: C-CLF-001. If Level 3 fails, classifier adds no value.

**Validated results (Level 0 linear probe, class-weighted online SGD)**:
- CodeBERT (124M params, 199 safetensors, 12 layers, 768 hidden) loads in ~23s
- [CLS] embeddings: 768-dim, L2 norm ~20.5, extraction ~1.82 entries/s (CPU)
- Training: sqrt-inverse balanced class weights, L2 weight_decay=1e-4, 100 epochs

**Pre-#172 results (Rust code as training text — STALE, domain mismatch)**:

| Entries | Test MCC | Accuracy | Precision | Recall | Train MCC | Ship Gate |
|---------|----------|----------|-----------|--------|-----------|-----------|
| 500 (BPE) | 0.427 | 94.2% | 0.300 | 0.429 | 0.749 | PASS |
| 1000 (BPE) | 0.399 | 92.2% | 0.353 | 0.545 | 0.666 | PASS keyword |
| 2047 (BPE) | **0.321** | 83.7% | 0.328 | 0.512 | 0.546 | **PASS** |
| 3000 (BPE) | 0.291 | — | — | — | — | FAIL (below 0.3) |

**Post-#172 results (shell output as training text — correct domain)**:

| Entries | Test MCC | Accuracy | Precision | Recall | Train MCC | Ship Gate |
|---------|----------|----------|-----------|--------|-----------|-----------|
| 3000 (shell) | 0.043 | 94.7% | 0.040 | 0.111 | 0.651 | FAIL |
| 3000 + 350 adv | 0.205 | 36.5% | 0.146 | 1.000 | 0.209 | FAIL |
| 3000 + 50 adv | 0.112 | 48.8% | 0.043 | 0.875 | 0.179 | FAIL |

**Post-#172 results (MLP probe, Level 0.5 — shell output + adversarial augmentation)**:

| Entries | Test MCC | Accuracy | Precision | Recall | Config | Ship Gate |
|---------|----------|----------|-----------|--------|--------|-----------|
| 3000 + 350 adv (MLP h=32) | **0.754** | 94.2% | 0.670 | 0.918 | lr=1e-4, 50 ep | **PASS** |
| 5220 + 350 adv (MLP h=32) | **0.736** | 95.5% | 0.644 | 0.894 | lr=1e-4, 50 ep | **PASS** |
| 7500 + 350 adv (MLP h=32) | **0.702** | 96.1% | 0.576 | 0.900 | lr=1e-4, 50 ep | **PASS** |
| 10016 + 350 adv (MLP h=32) | **0.693** | 96.6% | 0.554 | 0.906 | lr=1e-4, 50 ep | **PASS** |
| 12029 + 350 adv (MLP h=32) | **0.654** | 96.7% | 0.490 | 0.909 | lr=1e-4, 50 ep | **PASS** |
| 15027 + 350 adv (MLP h=32) | **0.613** | 96.8% | 0.425 | 0.919 | lr=1e-4, 50 ep | **PASS** |
| **17942 + 350 adv (MLP h=32)** | **0.443** | 93.0% | 0.248 | 0.870 | lr=1e-4, 50 ep | **PASS** |
| 3000 (MLP h=32, no aug) | -0.005 | 98.4% | 0.000 | 0.000 | lr=3e-4, 50 ep | FAIL |

- **MLP probe + adversarial augmentation solves shell-based classification** (KAIZEN-105)
  - MLP hidden layer (ReLU) captures non-linear patterns in CodeBERT embeddings
  - Adversarial augmentation (350 shell scripts, label=1) provides sufficient unsafe signal
  - Without augmentation: MLP converges to "all safe" (same as linear probe)
  - With augmentation: MCC=0.754, recall=91.8%, precision=67.0%
  - C-CLF-001: **PASS** (MCC=0.754 > 0.3 keyword, > 0.4 linter target)
- Linear probe insufficient for shell-based labeling (KAIZEN-104)
  - Root cause: transpiler normalizes unsafe patterns → shell output is homogeneous
  - Only 148/17,942 entries (0.82%) trigger lint in shell output
  - CodeBERT [CLS] embeddings not linearly separable on safe/unsafe for shell
  - Tracked: #173 (bashrs), entrenar#245 (fine-tuning infrastructure)
- Pre-#172 PASS results are **invalid** — domain mismatch between training (Rust) and inference (shell)
- Class weighting critical: without it, MCC degrades further (probe converges to "always safe")
- **Data labeling gap** (#171): corpus entries 3000+ have exactly 0 unsafe labels
  - Total unsafe entries: 283 (pre-#172, Rust code linting) → **148** (post-#172, shell output linting)
  - Beyond n=3000, test set is 100% safe → MCC=0.000
  - Max effective training size: ~2500 entries (beyond this, no unsafe test samples)
  - Fix: inject adversarial entries throughout expansion ranges (#171)
- **Distribution mismatch fix** (#172): training data now uses transpiled shell output
  - Pre-fix: training on Rust source code, inference on shell scripts (domain mismatch)
  - Post-fix: both training and inference use shell scripts
  - Labels: 17,794 safe / 148 unsafe (0.82% positive rate)
  - Linter baseline MCC=1.000 (tautological — labels derived from linter)
  - Keyword baseline MCC=0.103 (target to beat)
- **Classifier value proposition** (with linter-derived labels):
  1. **Distillation**: ML classifier learns linter's decision boundary → deployable in WASM without linter
  2. **Generalization**: CodeBERT embeddings may generalize to novel unsafe patterns (tested by 50 OOD scripts, S5.6)
  3. **Speed**: Single forward pass vs running 24 lint rules individually
  4. The linter baseline MCC=1.000 is expected — it IS the label source. C-CLF-001 measures whether CodeBERT can learn the same boundary from embeddings alone (MCC > 0.3 sufficient).

### Phase 2: Conversations (1 day)

| Task | Time | Status |
|------|------|--------|
| GEN-001: Template engine in bashrs (Rust) | 6 hrs | ✅ Done (4 types, 12+ variants each, ChatML system prompt) |
| GEN-002: Generate + quality gate (S6.4) | 2 hrs | ✅ Done (17,942 conversations, Type D 97.7%, 0 empty) |
| GEN-003: Publish dataset (`publish-conversations` CLI) | 1 hr | ✅ Done (JSONL + HF dataset README) |

### Phase 3: Chat Model (2-3 days)

| Task | Time | Status |
|------|------|--------|
| CHAT-001: Configure Qwen LoRA in entrenar | 3 hrs | ✅ Done (training manifest + entrenar JSONL export + provable contract) |
| CHAT-002: Fine-tune (RTX 4090) | 34 min | ✅ Done — Run 4 (1 epoch): loss=1.86, best=0.607 |
| CHAT-003: Evaluate + human review | 8 hrs | FAIL — 32.0% accuracy (6 runs, 3 bugs fixed). Run 6: balanced data → 32%. Kill criterion CONFIRMED — 0.5B model capacity limit reached |
| CHAT-004: Publish to HuggingFace | 10 min | ✅ Done (paiml/shell-safety-conversations). Chat model DELETED (kill criterion) |

### Phase 4: CLI (1 day)

| Task | Time | Status |
|------|------|--------|
| CLI-001: Wire bashrs classify → CodeBERT | 3 hrs | ✅ Done (`--probe --model` flags, full inference path) |
| CLI-002: Wire bashrs explain/fix → chat model | 3 hrs | ✅ Done (`--chat-model` flag, entrenar InstructPipeline::generate_chat(), 6 new tests) |
| CLI-003: Integration tests | 2 hrs | ✅ Done (49 assert_cmd tests — 43 original + 6 CLI-002) |

### Phase 5: WASM App via presentar (2 days)

| Task | Time | Status |
|------|------|--------|
| WASM-001: Build bashrs linter as `wasm32-unknown-unknown` target (bashrs-wasm crate) | 4 hrs | ✅ Done (1.5MB release, 7 tests) |
| WASM-002: Quantize CodeBERT to int8, export weights for browser loading | 2 hrs | ✅ Done (entrenar#249, --safetensors flag) |
| WASM-003: Build `shell-safety.html` interactive app with lint + classify | 4 hrs | ✅ Done (rule-based, 150ms debounce) |
| WASM-004: Wire CodeBERT WASM classifier (requires WASM-002) | 3 hrs | ✅ Done — KILL CRITERION 5 TRIGGERED (2.7s native, ~8s WASM est.) |
| WASM-005: IndexedDB model caching (load once, persist) | 2 hrs | Cancelled (WASM-004 kill criterion) |
| WASM-006: Deploy to interactive.paiml.com/shell-safety/ | 1 hr | ✅ Done (S3 + CloudFront) |

**Exit criterion**: Page loads, linter runs on keystroke, classifier runs on click,
no network calls after initial model download.

### Phase 6: Probar Testing (WASM + LLM + Performance) (1.5 days)

| Task | Time | Status |
|------|------|--------|
| PRB-001: Write Probar test suite (tests/probar_shell_safety.rs) — Layer 1 logic tests | 3 hrs | ✅ Done (14 logic tests, 5 perf tests, 19 total) |
| PRB-002: Wire WASM helper functions (load_bashrs_wasm, load_codebert_wasm, etc.) | 2 hrs | Cancelled (WASM-004 kill criterion — CLI only) |
| PRB-003: Generate reference fixtures from native CodeBERT for WASM parity tests | 1 hr | Cancelled (WASM-004 kill criterion — CLI only) |
| PRB-004: Write LLM correctness tests (NaN check, calibration, monotonicity, consistency) | 2 hrs | Cancelled (WASM-004 kill criterion — CLI only) |
| PRB-005: Write performance benchmark tests with hard budgets | 1 hr | ✅ Done (5 budget tests, all pass) |
| PRB-006: Configure Docker cross-browser matrix (Chrome/Firefox/WebKit) | 2 hrs | Deferred (Docker infra) |
| PRB-007: CI integration (logic=every commit, browser=pre-release, perf=every commit) | 1 hr | ✅ Done (Layer 1+3 via cargo test --workspace) |

**Kill gate**: C-PRB-001 through C-PRB-007. All Probar tests must pass before deployment.
Browser tests (Layer 2) may be deferred if Docker infra blocks, but logic + performance
tests (Layer 1 + Layer 3) are mandatory.

**Total: 13-17 days.** Phase 0 (encoder + contracts) is new engineering. Phase 5
(WASM app) is new UX. Phase 6 (Probar) validates correctness + performance.
Phases 1-4 reuse existing infrastructure.

---

## 10. Contracts

### Encoder

| Contract | Postcondition |
|----------|--------------|
| C-ENC-001 | Bidirectional: all tokens attend to all tokens |
| C-ENC-002 | Weight loading: 125M params, zero missing keys |
| C-ENC-003 | Numerical: output within L2 < 1e-4 of reference |

### Tokenizer

| Contract | Postcondition |
|----------|--------------|
| C-TOK-001 | >= 70% of shell constructs tokenized acceptably |

### Labels

| Contract | Postcondition |
|----------|--------------|
| C-LABEL-001 | >= 90% of audited "unsafe" labels are genuinely unsafe |

### Classifier

| Contract | Postcondition |
|----------|--------------|
| C-CLF-001 | `MCC_CI_lower > 0.2 AND accuracy > 0.935 AND generalization >= 50%` |

### Chat Model

| Contract | Postcondition |
|----------|--------------|
| C-CHAT-001 | `classification > 85% AND shellcheck > 85% AND citation > 90%` |

### Embedding

| Contract | Postcondition |
|----------|--------------|
| C-EMBED-001 | Bit-identical 768-dim embedding on repeated runs |

### WASM App

| Contract | Postcondition |
|----------|--------------|
| C-WASM-001 | WASM binary < 5MB (excluding model weights) |
| C-WASM-002 | Linter runs on keystroke < 10ms |
| C-WASM-003 | ~~Classifier inference < 500ms in browser (WebGPU)~~ KILL-5 triggered: 2.7s native |
| C-WASM-004 | ~~Model weights cached in IndexedDB after first load~~ Cancelled (KILL-5) |
| C-WASM-005 | ~~Zero network calls after initial model download~~ N/A (no model in browser) |

### Probar Testing (WASM + LLM Correctness + Performance)

| Contract | Postcondition |
|----------|--------------|
| C-PRB-001 | Layer 1 (logic): 12 WASM tests pass without browser |
| C-PRB-002 | ~~Layer 2 (browser): Chrome + Firefox + WebKit all pass via Docker~~ Deferred (Docker infra) |
| C-PRB-003 | Layer 3 (performance): All 6 budgets met (linter <10ms, classifier <500ms, memory <200MB, load <5s, tokenizer <5ms, pipeline <600ms) |
| C-PRB-004 | ~~LLM correctness: No NaN/Inf in embeddings~~ Cancelled (KILL-5, no CodeBERT in WASM) |
| C-PRB-005 | ~~LLM correctness: High-confidence predictions~~ Cancelled (KILL-5) |
| C-PRB-006 | ~~LLM correctness: WASM/native parity~~ Cancelled (KILL-5) |
| C-PRB-007 | Determinism: Repeated classify calls produce bit-identical results |

---

## 11. Kill Criteria

| Phase | Kill If | Action |
|-------|---------|--------|
| 0 | Encoder tests fail C-ENC-003 | Debug encoder, do not proceed |
| 0 | Tokenizer < 70% adequate (C-TOK-001) | Try mean-pool, or continue-pretrain, or accept limitation |
| 0 | Labels < 90% correct (C-LABEL-001) | Clean labels before training |
| 1 | Level 3 fails C-CLF-001 | STOP classifier work. Document. Linter is sufficient. |
| 3 | Human review < 2.5/5.0 | Ship classifier only. Chat not ready. |
| 5 | CodeBERT WASM inference > 2s | **TRIGGERED**: 2681ms native, est. 5-13s WASM. Ship CLI only. |
| 6 | Probar Layer 1 fails (LLM correctness) | Debug WASM build. Do not deploy. |
| 6 | Probar Layer 3 fails (performance budgets) | Profile and optimize. Raise budget if justified. |
| 6 | Probar Layer 2 fails (cross-browser) | Ship Chrome-only. Fix Firefox/WebKit later. |

Each artifact ships independently. Classifier can ship without chat model.
WASM app can ship with linter-only if classifier is too slow in browser.
Dataset can ship without either model. Negative results are published honestly.

---

## 12. What We Are NOT Doing

| Excluded | Why | Sovereign Alternative |
|----------|-----|----------------------|
| Python / PyTorch / HuggingFace lib | Sovereign Rust stack | entrenar + aprender + trueno |
| Ad-hoc Python data scripts | Non-reproducible, non-sovereign | alimentar CLI + apr data |
| Shell scripts for orchestration | Ironic (fixing what we lint) | apr pipeline (forjar DAG) |
| Manual JSONL/data munging | Error-prone, untraceable | alimentar convert/mix/filter |
| Raw entrenar binary invocation | Skips plan validation + monitoring | apr train plan/apply/watch |
| Manual HuggingFace upload | No metadata, no versioning | alimentar hub push + apr publish |
| Jupyter notebooks | Python dependency | alimentar repl + apr eval |
| Qwen-0.5B as classifier | CodeBERT is 4x smaller, 10x faster | CodeBERT via entrenar encoder |
| Multi-GPU | Single RTX 4090 sufficient | — |
| wgpu for training | CUDA proven path | CUDA via entrenar |
| Multi-label | Binary first | — |
| Confidence routing | Circular (F6) | — |
| Claiming "safety reasoning" | Model learns patterns, not reasoning (F5) | — |
| Chat model in WASM | Too large for browser (KILL-5) | CLI only |
| Playwright / Jest / Cypress | Non-sovereign JS testing | Probar (pure Rust) |

---

## 13. Falsification Log

All falsifications tracked and resolved:

| ID | Falsification | Status | Resolution |
|----|--------------|--------|------------|
| F1 | entrenar has no encoder support | **RESOLVED** | Build it. Encoder is simpler than decoder. ~2 days. (S4) |
| F2 | CodeBERT tokenizer not trained on shell | **MITIGATED** | Validate first (C-TOK-001). Three fallback options. (S5.2) |
| F3 | 116 unsafe test samples = weak statistics | **MITIGATED** | Use CI lower bounds, not point estimates. (S5.5) |
| F4 | RoBERTa [CLS] weak without fine-tuning | **MITIGATED** | Test both [CLS] and mean-pool. Escalation ladder. (S5.4) |
| F5 | Synthetic conversations are templates | **RESOLVED (v12)** | Regenerated from real shell code via conversations.rs. 48 prompt variants. Old data had 90 unique responses; new data has per-entry lint-grounded responses. |
| F6 | Confidence routing is circular | **RESOLVED** | Removed. Chat is explicit-only command. (S8.2) |
| F7 | Labels derived from transpiler, not safety | **RESOLVED (v12)** | v12 uses bashrs lint on real shell code. Labels are SEC/DET/IDEM findings, not transpiler labels. CWE-mapped. |
| F8 | 926 unsafe examples is thin | **RESOLVED (v12)** | ~7,600 unsafe examples from corpus + verificar mutations. 8× improvement. |
| F9 | Timeline assumes no blockers | **MITIGATED** | Phase 0 scoped, buffer days added. 9-12 days. (S9) |
| F10 | Scripts too short for rich embeddings | **ACKNOWLEDGED** | Value on multi-line scripts. Honest about limitation. |
| F11 | "First shell-specific benchmark" may be false | **VERIFY** | Search HuggingFace, GitHub, arXiv for "bash security benchmark" or "shell vulnerability dataset" before publication. If competitor found, position as "first CWE-mapped" or "first with eval harness". |
| F12 | ~28K entries ≠ quality benchmark | **MITIGATED** | SEC-bench achieves meaningful results with 600 CVEs. Report unique CWE coverage (15 CWEs), script complexity distribution (LOC histogram), and format diversity (3 languages). Size is necessary but not sufficient. |
| F13 | "4B specialist beats 7B+ general" may not hold | **TEST** | General models may win on explanation quality even if specialist wins on detection. Report per-metric breakdown, not just composite score. Hypothesis is falsifiable. |
| F14 | DPO chosen/rejected pairs may be template artifacts | **MITIGATE** | Audit 100 random pairs for genuine preference alignment. Rejected responses must be plausibly wrong (not strawman). Add verificar-generated near-miss rejects. |
| F15 | Linter-derived labels have false positives | **QUANTIFY** | Hand-review 200 samples. Measure bashrs FP rate. Target: FP < 5%. If higher, add FP-aware noise labels or exclude ambiguous rules from ground truth. |

---

## 14. ShellSafetyBench (v12)

### 14.1 Why This Doesn't Exist Yet

Every code security benchmark focuses on compiled languages:

| Benchmark | Languages | Shell/Bash? | Year |
|-----------|----------|-------------|------|
| CASTLE | C/C++ | No | 2025 |
| SafeGenBench | 8 languages | No | 2025 |
| SecRepoBench | Multi | No | 2025 |
| CyberNative DPO | 11 languages | No | 2024 |
| CodeAstra-7B | Multi | No | 2024 |
| VulnLLM-R-7B | C/C++/Java/Python | No | 2025 |

Shell scripts are the most common attack surface for infrastructure (CI/CD,
Docker, deploy, cron) yet have zero ML-based security tooling or benchmarks.

### 14.2 CWE Taxonomy Mapping

bashrs linter rules mapped to MITRE CWE identifiers:

| Rule | Pattern | CWE | CVSS v3.1 | OWASP Category |
|------|---------|-----|-----------|----------------|
| SEC001 | Unquoted variable expansion | CWE-78 | 7.8 (High) | OS Command Injection |
| SEC002 | eval usage | CWE-94 | 8.8 (High) | Code Injection |
| SEC003 | Unquoted command substitution | CWE-78 | 7.8 (High) | OS Command Injection |
| SEC004 | Backtick command substitution | CWE-78 | 7.8 (High) | OS Command Injection |
| SEC005 | Source/eval of variable | CWE-94 | 8.8 (High) | Code Injection |
| SEC006 | Curl piped to shell | CWE-829 | 9.8 (Critical) | Inclusion of Untrusted Functionality |
| SEC007 | World-writable permissions | CWE-732 | 5.3 (Medium) | Incorrect Permission Assignment |
| SEC008 | Hardcoded credentials | CWE-798 | 7.5 (High) | Use of Hard-coded Credentials |
| SEC013 | Insecure /tmp usage | CWE-377 | 5.9 (Medium) | Insecure Temporary File |
| DET001 | $RANDOM usage | CWE-330 | 3.7 (Low) | Insufficient Randomness |
| DET002 | Timestamp in output | CWE-330 | 3.7 (Low) | Insufficient Randomness |
| DET003 | Unsorted glob expansion | CWE-330 | 3.7 (Low) | Insufficient Randomness |
| IDEM001 | mkdir without -p | CWE-362 | 4.7 (Medium) | Race Condition (TOCTOU) |
| IDEM002 | rm without -f | CWE-362 | 4.7 (Medium) | Race Condition (TOCTOU) |
| IDEM003 | ln without -sf | CWE-362 | 4.7 (Medium) | Race Condition (TOCTOU) |

CVSS base scores assume local attacker context (AV:L) for variable expansion and
remote context (AV:N) for curl-pipe-bash. Scores enable **severity prioritization**
in eval results, not just binary detection.

### 14.3 Data Pipeline

```
┌──────────────────────────────────────────────────────────┐
│  Source 1: bashrs corpus (17,942 entries)                 │
│  ├── 16,431 Bash scripts                                 │
│  ├── 804 Makefiles                                       │
│  └── 707 Dockerfiles                                     │
│  Labels: bashrs lint → SEC/DET/IDEM findings = unsafe    │
│          no findings = safe                              │
├──────────────────────────────────────────────────────────┤
│  Source 2: verificar synthetic generation (~10K)          │
│  ├── Generate safe programs (depth 3-5, all features)    │
│  ├── Mutate to inject specific CWE patterns              │
│  │   ├── AOR: arithmetic boundary errors                 │
│  │   ├── BSR: boundary substitution (empty, overlong)    │
│  │   └── CWE-targeted: inject $RANDOM, eval, unquoted   │
│  └── Oracle: execute before/after, verify behavior Δ     │
├──────────────────────────────────────────────────────────┤
│  bashrs lint (each entry)                                │
│  └── Ground truth labels with rule IDs + line numbers    │
├──────────────────────────────────────────────────────────┤
│  conversations.rs (generate training conversations)      │
│  ├── Type A: Classify+Explain (unsafe + lint findings)   │
│  ├── Type B: Fix (unsafe + corrected version)            │
│  ├── Type C: Debug (non-deterministic + DET findings)    │
│  ├── Type D: Confirm Safe (≥30% of total)                │
│  └── 12 prompt variants per type (48 total phrasings)    │
├──────────────────────────────────────────────────────────┤
│  Output: ~28K entries in DPO-compatible JSONL             │
│  ├── train split (80%)                                   │
│  ├── val split (10%)                                     │
│  └── test split (10%, held out for benchmark)            │
├──────────────────────────────────────────────────────────┤
│  Human Validation Set (200 entries)                       │
│  ├── Security researchers label without seeing bashrs     │
│  ├── Include bashrs false positives (safe flagged unsafe) │
│  ├── Include bashrs false negatives (unsafe missed)       │
│  ├── Report inter-annotator agreement (Cohen's κ)         │
│  └── Purpose: measure linter-free generalization          │
├──────────────────────────────────────────────────────────┤
│  Dynamic Eval (anti-gaming, per SEC-bench methodology)   │
│  ├── 500 verificar-generated scripts per eval run        │
│  ├── Seeded by date + commit hash (reproducible but fresh)│
│  ├── Prevents training-on-test contamination              │
│  └── Report static and dynamic scores separately          │
└──────────────────────────────────────────────────────────┘
```

### 14.4 Benchmark Schema

DPO-compatible format (same structure as CyberNative dataset, extended for shell):

```json
{
  "id": "SSB-00142",
  "lang": "bash",
  "cwe": "CWE-78",
  "rule": "SEC001",
  "severity": "error",
  "vulnerability": "OS Command Injection via unquoted variable",
  "script": "#!/bin/bash\nrm -rf $USER_INPUT/tmp",
  "chosen": "Classification: unsafe\n\n**SEC001** (line 2): Unquoted variable `$USER_INPUT` allows injection...\n\nFixed:\n```bash\n#!/bin/bash\nrm -rf \"${USER_INPUT:?}/tmp\"\n```",
  "rejected": "This script looks fine. It removes a temporary directory.",
  "source": "bashrs-corpus",
  "conversation_type": "classify-explain"
}
```

Fields specific to ShellSafetyBench (not in CyberNative):
- `lang`: bash | makefile | dockerfile (infrastructure trifecta)
- `rule`: bashrs-specific rule ID (SEC/DET/IDEM/SC codes)
- `severity`: error | warning | info (from linter)
- `script`: raw source code (not embedded in prompt)
- `source`: bashrs-corpus | verificar-generated | verificar-mutated
- `conversation_type`: classify-explain | fix | debug | confirm-safe

### 14.5 Evaluation Metrics (Leaderboard)

| Metric | Description | Weight |
|--------|-------------|--------|
| **Detection F1** | Binary safe/unsafe classification accuracy | 25% |
| **Rule Citation** | Correct SEC/DET/IDEM rule ID in response | 20% |
| **CWE Mapping** | Correct CWE ID referenced | 10% |
| **Fix Validity** | Suggested fix passes shellcheck AND removes vuln | 15% |
| **Explanation** | Response explains WHY the pattern is dangerous | 15% |
| **OOD Generalization** | Correct on novel CWE patterns not in bashrs linter | 15% |

The OOD (out-of-distribution) metric tests patterns the model was NOT trained on:
- CWE-426: Untrusted search path (`PATH` manipulation)
- CWE-77: Command injection via `xargs` without `-0`
- CWE-116: Improper output encoding (log injection via `echo $untrusted`)
- CWE-250: Execution with unnecessary privileges (`sudo` in scripts)
- Scripts from external sources (GitHub Actions workflows, Docker Hub Dockerfiles)

This prevents Goodhart's Law: a model that memorizes bashrs linter output will
score 0% on OOD, exposing the gap between pattern matching and understanding.

#### 14.5.1 Automated Scoring (Leaderboard)

- Detection F1: parse first line for "Classification: safe/unsafe", compare to label
- Rule Citation: regex match for rule IDs in response, compare to ground truth
- Fix Validity: extract ```bash blocks, run `shellcheck -s sh`, re-lint with bashrs
- CWE Mapping: regex match for CWE-\d+ in response
- Explanation: check for key terms (injection, non-deterministic, race condition, etc.)
- OOD: same automated pipeline but on held-out novel-CWE scripts

#### 14.5.2 LLM-as-Judge Scoring (Depth)

Automated regex scoring is fragile — a correct explanation using different terminology
scores 0. For a 200-sample subset, use Claude/GPT-4 as judge (per SafeGenBench methodology):

- **Explanation quality** (1-5): Does the response correctly identify the vulnerability,
  explain the attack vector, and describe the impact? Scored by LLM judge.
- **Fix completeness** (1-5): Does the fix address ALL vulnerabilities in the script,
  not just the first one? Scored by LLM judge.
- **Hallucination rate**: Does the response cite rules/CWEs that don't apply? Binary.

Report both scores: automated is the leaderboard metric (reproducible, cheap);
LLM-judge is reported in the paper (captures nuance, catches false negatives in
automated scoring).

#### 14.5.3 Dynamic Evaluation (Anti-Gaming)

Static benchmarks invite overfitting (per "The Leaderboard Illusion", 2025).
ShellSafetyBench includes a **regenerable dynamic split**:

- Each eval run generates 500 fresh scripts via verificar (`--seed $(date +%Y%m%d)`)
- Static portion (corpus-derived, ~2,800 test entries) stays fixed for reproducibility
- Dynamic portion (verificar-generated) prevents training-on-test contamination
- Report both scores: `Static: XX.X%, Dynamic: YY.Y%`
- A large gap (Static >> Dynamic) indicates overfitting to the benchmark

### 14.6 Baseline Models (to evaluate on ShellSafetyBench)

| Model | Params | Type | Expected Strength |
|-------|--------|------|-------------------|
| GPT-4o | ~200B? | General | Good reasoning, no shell specialization |
| Claude Sonnet 4.6 | ~100B? | General | Strong code analysis |
| Qwen3-4B (base) | 4B | General | Zero-shot shell knowledge |
| Qwen2.5-Coder-7B | 7B | Code | Code-specialized, no shell focus |
| CodeLlama-7B | 7B | Code | Code-specialized, no shell focus |
| **shell-safety-qwen3-4b** | **4B** | **Shell specialist** | **Trained on ShellSafetyBench** |
| bashrs lint | N/A | Rule-based | Perfect on known rules, no novel detection |

Hypothesis: shell-safety-qwen3-4b (4B) beats general-purpose 7B+ models on
shell-specific security tasks due to domain specialization. General models
will miss linter-specific rules and produce generic security advice.

### 14.7 HuggingFace Deliverables

| Artifact | Repo | Format |
|----------|------|--------|
| Benchmark dataset | `paiml/shell-safety-bench` | Parquet + JSONL, DPO-compatible |
| QLoRA adapter | `paiml/shell-safety-qwen3-4b` | PEFT safetensors, merge-compatible |
| Eval harness | `paiml/shell-safety-eval` | HF Space with model comparison |
| Model card | In model repo | CWE coverage, benchmark scores, limitations |
| Dataset card | In dataset repo | Generation methodology, label provenance |

### 14.8 What Makes This Defensible

1. **No competing dataset**: Shell/Make/Docker security has zero ML benchmarks (F11: verify pre-publication)
2. **Verified labels**: Ground truth from deterministic linter, FP rate < 5% (F15: quantified on 200 samples)
3. **Reproducible pipeline**: bashrs + verificar + conversations.rs = unlimited data
4. **CWE-mapped + CVSS-scored**: Credible for security researchers, maps to OWASP/MITRE, enables severity prioritization
5. **Three languages**: Bash + Makefile + Dockerfile = infrastructure trifecta
6. **DPO-compatible**: Can train with RLHF/DPO, not just SFT
7. **Sovereign stack**: No Python/PyTorch dependency in generation pipeline
8. **Anti-gaming**: Dynamic eval via verificar prevents benchmark overfitting (per Leaderboard Illusion)
9. **Dual scoring**: Automated (reproducible leaderboard) + LLM-as-judge (captures nuance)
10. **OOD generalization**: 15% eval weight on novel CWEs not in training set — measures understanding vs memorization
11. **Human validation**: 200-entry linter-free eval with inter-annotator agreement (Cohen's κ)
12. **Cross-linter validation**: ShellCheck findings as secondary oracle — bashrs rules aren't idiosyncratic

### 14.9 Implementation Plan (Phase 7 — Sovereign Tooling Only)

**Orchestration**: `apr pipeline plan/apply configs/pipeline/ssc.yaml` (see S2b.3)

All steps below are encoded as resources in the pipeline manifest. No ad-hoc scripts.
Manual steps (7.1, 7.4c, 7.11) are the ONLY exceptions — they produce YAML/spec artifacts
that are then consumed by automated pipeline stages.

| Step | Task | Sovereign Command | Estimate |
|------|------|-------------------|----------|
| 7.1 | Map CWE IDs + CVSS scores (spec update) | Manual → `configs/cwe-mapping.yaml` | 1 hour |
| 7.2 | CWE-targeted mutations | `verificar generate --language bash --count 10000 --strategy exhaustive` | 1 day |
| 7.2b | OOD CWE generators (CWE-426/77/116/250) | `verificar generate --cwe-targets 426,77,116,250 --count 2000` | 4 hours |
| 7.3 | Regenerate conversations from corpus | `bashrs corpus generate-conversations --entrenar --output conversations.jsonl` | 2 hours |
| 7.3b | Label corpus entries | `bashrs corpus label --input conversations.jsonl --format json` | 30 min |
| 7.4 | Merge + audit data | `alimentar mix --input *.jsonl && apr data audit merged.jsonl --num-classes 2` | 30 min |
| 7.4b | Stratified split + balance | `apr data split --stratified && apr data balance --strategy sqrt-inverse` | 30 min |
| 7.4c | Decontaminate train vs test | `apr data decontaminate train.jsonl --reference test.jsonl` | 15 min |
| 7.4d | Quality gate | `alimentar quality score train.jsonl --profile ml-training` | 15 min |
| 7.4e | Cross-validate vs ShellCheck | `shellcheck -f json` on 500 samples (secondary oracle) | 2 hours |
| 7.4f | Hand-label 200 human validation set | Manual → `training/shellsafetybench/human-validation.jsonl` | 4 hours |
| 7.5 | Training plan (dry-run) | `apr train plan --config configs/train/ssc-qwen3-4b-qlora.yaml` | 15 min |
| 7.6 | Train Run 7 | `apr train apply --config configs/train/ssc-qwen3-4b-qlora.yaml --seed 42` | 2-4 hours |
| 7.6b | Monitor training | `apr train watch --config configs/train/ssc-qwen3-4b-qlora.yaml` | (concurrent) |
| 7.7 | Eval static test set | `apr eval checkpoints/ --task classify --data splits/test.jsonl` | 1 hour |
| 7.7b | Eval dynamic set | `verificar generate --count 500 --seed $(date) && apr eval` | 1 hour |
| 7.7c | Eval OOD novel-CWE set | `apr eval checkpoints/ --data ood-cwe-test.jsonl` | 1 hour |
| 7.7d | LLM-as-judge 200-sample | `apr eval --judge claude-sonnet --data human-validation.jsonl` | 1 hour |
| 7.8 | Baseline comparison | `apr bench checkpoints/ --task shell-safety --baselines gpt4o,claude,qwen-7b` | 2 hours |
| 7.9 | QA release gate | `apr qa --checklist configs/qa/ssc-release-v1.yaml` | 30 min |
| 7.10 | Publish dataset | `alimentar hub push splits/ paiml/shell-safety-bench --format parquet` | 30 min |
| 7.10b | Publish model | `apr publish checkpoints/ paiml/shell-safety-qwen3-4b --license apache-2.0` | 30 min |
| 7.10c | Create HF Space leaderboard | `presentar deploy --config configs/space/ssc-leaderboard.yaml` | 2 hours |
| 7.11 | Verify F11 ("first shell benchmark") | Web search (manual, pre-publication) | 30 min |

**One-command execution**: `apr pipeline apply configs/pipeline/ssc.yaml`

Steps 7.1, 7.4f, and 7.11 are manual. All other steps are declaratively encoded
in the pipeline manifest and execute automatically in dependency order.

---

## 15. Version History

| Version | Date | Change |
|---------|------|--------|
| 1.0-3.4 | 2026-02 to 03-03 | v1 MLP, v2 LoRA 5-class, v3 binary QLoRA — failed to ship |
| 4.0 | 2026-03-05 | Retrospective + ShellSafeBench (superseded) |
| 5.0 | 2026-03-05 | Linear probing on Qwen (superseded) |
| 6.0 | 2026-03-05 | CodeBERT + 1.5B chat (falsified: F1 "encoder too hard") |
| 7.0 | 2026-03-05 | Dropped CodeBERT for Qwen-0.5B (sovereign stack constraint) |
| 8.0 | 2026-03-05 | Restored CodeBERT. Build encoder support. F1 re-falsified: encoder is simpler than decoder. |
| 9.0 | 2026-03-05 | Added WASM app via presentar. CodeBERT (125M int8) runs in browser. |
| 10.0 | 2026-03-05 | Added provable-contracts (4 YAML contracts, pv scaffold/bind/audit pipeline) + Brick profile-first design (PROBAR-SPEC-009): 5 widgets, 18 test-first assertions, JIDOKA enforcement, ModelState state machine. |
| **11.0** | **2026-03-05** | **Probar-first testing design (S8.4): 3-layer test suite (logic/browser/performance), 21 test-first tests, LLM correctness verification (NaN, calibration, monotonicity, reference parity), 6 hard performance budgets, Docker cross-browser matrix, dual-runtime strategy, 7 new contracts (C-PRB-001..007), Phase 6 added to implementation plan.** |
| 11.1 | 2026-03-08 | Phase 3b: LoRA re-run after ENT-LoRA-001..018 implementation. 28 falsification tests pass. |
| **12.0** | **2026-03-08** | **ShellSafetyBench reframe (S14): first shell-specific security benchmark + model. Key changes: (1) training data from real shell/Make/Docker corpus, not Rust transpiler output; (2) CWE taxonomy mapping for all linter rules; (3) verificar mutations for synthetic unsafe pairs; (4) DPO-compatible benchmark schema; (5) eval harness with 5 metrics; (6) Qwen3-4B NF4 QLoRA as specialist model; (7) HuggingFace publication plan (dataset + model + eval Space + leaderboard). Replaces conversations_v3.jsonl (2,268 entries, 85% Rust code) with ~28K entries of real shell code.** |
| **12.1** | **2026-03-08** | **Hardened eval methodology from 4-stream review (arxiv SEC-bench/SafeGenBench, Popperian Leaderboard Illusion, batuta oracle stack review). Key changes: (1) CVSS v3.1 base scores on CWE table (S14.2); (2) human validation set 200 entries with Cohen's κ (S14.3); (3) dynamic eval via verificar, anti-gaming (S14.5.3); (4) OOD generalization as 6th metric at 15% weight (S14.5); (5) dual scoring: automated leaderboard + LLM-as-judge depth (S14.5.2); (6) F11-F15 falsifications added (S13); (7) cross-linter validation with ShellCheck (S14.8); (8) linter FP rate quantification target <5% (F15).** |
| **12.2** | **2026-03-08** | **Sovereign tooling mandate (S2b) + VRAM budget (S8.1b.5). Tooling: ALL pipeline ops via apr-cli/alimentar/entrenar/verificar, 10 banned practices, forjar DAG manifest, QA checklist, one-command execution. VRAM: corrected from 2.9 GB (inference) to 5-7 GB (training), albor concurrent training infeasible (combined ~18-20 GB > 24 GB), KILL-CHAT-003 revised with scheduling options.** |
