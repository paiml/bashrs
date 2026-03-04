# Shell Safety Classifier

bashrs includes a neural shell safety classifier that categorizes shell commands by safety risk. It combines a rule-based linter (instant, zero dependencies) with an optional transformer-based classifier (Qwen2.5-Coder + LoRA) for higher accuracy on novel patterns.

## Safety Classes

### v3 Binary Classification (Current)

The v3 classifier uses binary classification derived from the transpilation corpus:

| Class | Label | Index | Derivation |
|-------|-------|-------|------------|
| **Safe** | `safe` | 0 | Transpiled AND lint-clean AND deterministic |
| **Unsafe** | `unsafe` | 1 | Otherwise (failed transpilation, lint errors, or non-deterministic) |

This replaced the earlier 5-class taxonomy (safe, needs-quoting, non-deterministic, non-idempotent, unsafe) which failed to converge because only 3 of 5 classes were populated in the corpus data.

### Rule-Based Classes (5-class, for linter output)

The rule-based classifier still reports 5 severity levels for detailed diagnostics:

| Class | Index | Risk Level | Description | Example |
|-------|-------|------------|-------------|---------|
| **Safe** | 0 | None | Deterministic, idempotent, properly quoted | `echo "hello world"` |
| **Needs Quoting** | 1 | Low | Unquoted variable expansion risks word splitting | `echo $HOME` |
| **Non-Deterministic** | 2 | Medium | Output varies between runs | `echo $RANDOM`, `date +%s` |
| **Non-Idempotent** | 3 | Medium | Unsafe to re-run (missing safety flags) | `mkdir /tmp/build` |
| **Unsafe** | 4 | High | Security violations (injection, privilege escalation) | `eval "$user_input"` |

When a command exhibits multiple issues, the highest-severity class wins:

```text
unsafe > non-deterministic > non-idempotent > needs-quoting > safe
```

## Quick Start

### Rule-Based Classification (Built-in)

The rule-based classifier requires no model weights and runs in under 1ms:

```bash
# Classify a single script
bashrs classify script.sh
# Output: safe (confidence: 95.0%)

# JSON output with per-class scores
bashrs classify --json script.sh
# {"label":"safe","index":0,"confidence":0.95,"scores":[0.95,0.01,0.01,0.02,0.01]}

# Multi-label mode (detects ALL applicable issues)
bashrs classify --multi-label script.sh
# Output: non-deterministic + needs-quoting

# Classify a Makefile or Dockerfile
bashrs classify Makefile
bashrs classify Dockerfile
```

### Neural Classification (LoRA Model)

The neural classifier uses a fine-tuned Qwen2.5-Coder-0.5B model for higher accuracy on novel patterns:

```bash
# Classify using the trained model
bashrs classify --model /path/to/ssc-checkpoints/ script.sh

# The model is a small LoRA adapter (~4MB) loaded on top of
# the frozen Qwen2.5-Coder-0.5B base model (~1GB)
```

## How It Works

### Rule-Based Pipeline

The built-in classifier uses bashrs's linter rules as features:

```text
script.sh
    |
    v
lint_shell()  -->  SEC001-008 (security)
                   DET001-006 (determinism)
                   IDEM001+   (idempotency)
                   SC1xxx     (shellcheck)
    |
    v
derive_safety_label()  -->  Priority decision tree
    |
    v
SafetyClass (0-4) + confidence score
```

The decision tree applies cascading priority:

1. **Lint failure or SEC rule violation** -> Unsafe (4)
2. **DET rule violation** ($RANDOM, $$, timestamps) -> Non-Deterministic (2)
3. **Missing safety flags** (mkdir without -p, rm without -f) -> Non-Idempotent (3)
4. **Unquoted variable expansion** -> Needs Quoting (1)
5. **All checks pass** -> Safe (0)

### Neural Pipeline

The neural classifier leverages pretrained code understanding from Qwen2.5-Coder:

```text
script.sh (preamble stripped)
    |
    v
BPE Tokenizer (151,665 tokens)
    |
    v
Qwen2.5-Coder (frozen, 494M+ params)
  + LoRA adapters (Q/V projections, ~1.1M trainable params)
    |                              ┌──────────────────────┐
    ├─ FFN on GPU (wgpu/CUDA) ────│ GpuCommandBatch      │
    ├─ Attention on CPU ──────────│ (batched matmul)      │
    |                              └──────────────────────┘
    v
Mean pooling over sequence
    |
    v
Classification head: Linear(896 -> 2)
    |
    v
Softmax -> binary probability (safe vs unsafe)
```

The model is fine-tuned on bashrs's 17,942-entry corpus (v3, binary classification after preamble stripping) using LoRA (Low-Rank Adaptation) on the query and value attention projections. Only ~0.2% of parameters are trained, keeping the base model's code understanding intact while learning shell safety patterns.

## Training Data

The classifier is trained on bashrs's transpilation corpus with binary labels (v3):

| Format | Entries | Source |
|--------|---------|--------|
| Bash | ~16,431 | Core transpilation corpus |
| Makefile | ~804 | Makefile purification corpus |
| Dockerfile | ~707 | Dockerfile purification corpus |
| **Total** | **17,942** | All formats combined |

### Class Distribution (v3 Binary)

| Class | Label | Count | Percentage |
|-------|-------|-------|------------|
| 0 | safe | 16,784 | 93.5% |
| 1 | unsafe | 1,158 | 6.5% |

Labels are derived automatically by `classify_single()` — scripts that transpile successfully, pass linting, and are deterministic are `safe`; everything else is `unsafe`. The imbalance ratio is 14.5:1; entrenar auto-applies sqrt-inverse class weights (safe=0.534, unsafe=7.747).

### Data Pipeline (v3)

Training data flows through three tools with strict separation of concerns:

```text
bashrs (export) → alimentar (split) → entrenar (train)
```

1. **bashrs** exports `corpus.jsonl` via `fast_classify_export` with `validate_export()` DataOps gate
2. **alimentar** handles stratified 80/10/10 splitting (train/test/val), preserving class distribution
3. **entrenar** trains on the split data

### Data Preprocessing

Training data undergoes automatic preamble stripping before export. The transpiler's boilerplate preamble (`set -euf`, `trap '... $$'`, etc.) is removed because:

- The `trap '... $$'` pattern contains a non-deterministic PID reference, causing the classifier to confuse safe commands with non-deterministic ones
- `set -euf` and shebang lines add no safety-relevant signal
- Stripping focuses the model on the actual command semantics

The `strip_shell_preamble()` function (shared with corpus B2 scoring) canonically identifies and removes these lines.

## Model Architecture

### Base Model: Qwen2.5-Coder-0.5B

| Parameter | Value |
|-----------|-------|
| Hidden size | 896 |
| Layers | 24 |
| Attention heads | 14 (+ 2 KV heads, GQA) |
| Intermediate size | 4,864 |
| Vocabulary | 151,665 BPE tokens |
| Total parameters | ~494M (frozen) |

### LoRA Configuration

| Parameter | Value |
|-----------|-------|
| Rank | 16 |
| Alpha | 16.0 |
| Target modules | Q and V projections |
| Adapters | 48 (2 per layer x 24 layers) |
| Trainable parameters | 1,085,829 (~0.2% of total) |

### Classification Head

| Parameter | Value |
|-----------|-------|
| Input dimension | 896 (hidden size) |
| Output dimension | 2 (binary: safe vs unsafe) |
| Parameters | 1,794 |
| Activation | None (logits -> softmax at inference) |

### Training Configuration

| Parameter | Value |
|-----------|-------|
| Optimizer | AdamW |
| Learning rate | 2e-4 (C-HP-001) |
| Epochs | 3 |
| Batch size | 4, grad_accum=4 (effective 16, C-HP-002) |
| LoRA alpha | 32 (2x rank, C-HP-003) |
| Max sequence length | 256 (from data p99, C-HP-004) |
| Warmup | 6% of steps (C-HP-005) |
| Gradient clipping | max_norm=1.0 (C-HP-006) |
| Loss function | CrossEntropyLoss |
| Class weights | sqrt-inverse (safe=0.534, unsafe=7.747) |
| Device | CUDA (NVIDIA), wgpu (AMD/Intel), or CPU |

## Multi-GPU Training

Training supports multiple compute backends and data parallelism across GPUs.

### Compute Backend Priority

The training pipeline automatically selects the best available backend:

```text
CUDA (NVIDIA) → wgpu (AMD/Intel/NVIDIA) → CPU
```

| Backend | Forward Pass | Backward Pass | Multi-GPU |
|---------|-------------|---------------|-----------|
| **CUDA** | Full GPU (all ops) | Full GPU (LoRA + frozen) | Yes (via DataParallelCoordinator) |
| **wgpu** | FFN on GPU, attention on CPU | CPU-only (LoRA adapters) | Yes (via DataParallelCoordinator) |
| **CPU** | All CPU (trueno SIMD) | CPU-only (LoRA adapters) | N/A |

### Data Parallelism

For multi-GPU systems, `DataParallelCoordinator` replicates the model across GPUs:

1. Split mini-batch into N shards (one per GPU)
2. Each GPU processes its shard independently
3. Average LoRA gradients on CPU (~22MB for rank-16, <2ms over PCIe)
4. Broadcast primary's weights to all replicas

```bash
# Train on two GPUs
apr finetune --task classify --model-size 4B --gpus 0,1 \
    ./models/qwen3-4b --data train.jsonl \
    --num-classes 2 --epochs 3 -o ./checkpoints/
```

### Memory Budget (Qwen3-4B, 8GB VRAM per GPU)

| Component | fp32 | NF4 (QLoRA) |
|-----------|------|-------------|
| Model weights | 1,007 MB | 126 MB |
| LoRA adapters | 5.5 MB | 5.5 MB |
| Activations (seq128) | ~28 MB | ~28 MB |
| Optimizer state | ~11 MB | ~11 MB |
| **Total** | **~1,052 MB** | **~171 MB** |

## Inference

### Performance

| Mode | Latency | Dependencies |
|------|---------|-------------|
| Rule-based | <1ms | None (built into bashrs) |
| Neural (CPU) | ~50ms | Base model + LoRA adapter |
| Neural (GPU) | ~5ms | CUDA or wgpu-capable GPU |

The rule-based classifier is always available and fast enough for real-time use (linting, CI/CD pipelines). The neural classifier provides higher accuracy on novel patterns at the cost of model loading overhead.

### Deployment

The fine-tuned model is published to HuggingFace as [`paiml/shell-safety-classifier`](https://huggingface.co/paiml/shell-safety-classifier). It ships as a LoRA adapter (~4MB) on top of the public Qwen2.5-Coder-0.5B base model:

```text
paiml/shell-safety-classifier/
  adapter.safetensors              # LoRA adapter weights (~4MB)
  classifier_head.safetensors      # Linear(896->2) classification head
  shell-safety-classifier.apr      # APR format (sovereign stack native)
  config.json                      # Model architecture config
  tokenizer.json                   # Qwen2 BPE tokenizer (151,665 tokens)
  README.md                        # Model card (Mitchell et al. 2019)
```

Loading at inference:
1. Download base model once: `Qwen/Qwen2.5-Coder-0.5B` (~1GB)
2. Download adapter: `paiml/shell-safety-classifier` (~4MB)
3. `realizar` merges adapter into base model and serves classification

The adapter can also be:
- Bundled with bashrs releases
- Downloaded on first use (`bashrs classify --model auto`)
- Loaded from a custom path (`bashrs classify --model /path/to/checkpoint`)

No GPU is required for inference — CPU is sufficient for the 0.5B model.

## Cross-Format Support

The classifier supports all three formats that bashrs processes:

### Bash Scripts

```bash
bashrs classify script.sh
# Uses: SEC001-008, DET001-006, IDEM001+ rules
```

### Makefiles

```bash
bashrs classify Makefile
# Uses: MAKE001 (unsorted wildcard → DET)
#       MAKE002 (missing .PHONY → IDEM)
#       MAKE003 (shell injection → SEC)
```

### Dockerfiles

```bash
bashrs classify Dockerfile
# Uses: DOCKER001 (root user → SEC)
#       DOCKER002 (unpinned tag → DET)
#       DOCKER006 (ADD vs COPY → SEC)
```

Format is auto-detected from the file extension. Use `--format` to override:

```bash
bashrs classify config.txt --format makefile
```

## Multi-Label Classification

By default, bashrs assigns the single highest-priority label. With `--multi-label`, it reports ALL applicable safety issues:

```bash
# Single-label (default): highest priority wins
bashrs classify script.sh
# Output: non-deterministic (confidence: 87.3%)

# Multi-label: all issues reported
bashrs classify --multi-label script.sh
# Output: non-deterministic + needs-quoting
```

Multi-label mode uses `BCEWithLogitsLoss` (binary cross-entropy) instead of `CrossEntropyLoss`, treating each class as an independent binary decision. This is useful for detailed safety audits where you want to know all issues, not just the most severe one.

### Multi-Label Corpus Export

```bash
# Export multi-label training data
bashrs corpus export-dataset --format multi-label-classification -o corpus.jsonl
# Output: {"input":"echo $RANDOM","labels":[0.0,1.0,1.0,0.0,0.0]}
```

## Training Your Own Model

### Prerequisites

- bashrs >= 6.65.0
- aprender >= 0.26.3 (ML framework)
- entrenar >= 1.0 (training engine)
- Qwen2.5-Coder-0.5B weights from HuggingFace

### Step 1: Download Base Model

```bash
python3 -c "
from huggingface_hub import snapshot_download
snapshot_download('Qwen/Qwen2.5-Coder-0.5B',
                  local_dir='./models/qwen2.5-coder-0.5b',
                  allow_patterns=['*.safetensors', 'tokenizer.json', 'config.json',
                                  'tokenizer_config.json', 'vocab.json', 'merges.txt'])
"
```

### Step 2: Export Training Corpus

```bash
# Classification-only JSONL for fine-tuning
bashrs corpus export-dataset --format classification -o corpus.jsonl

# Fast export (skips B3/cross-shell, transpile+lint+label only)
cargo run -p bashrs --release --example fast_classify_export /tmp/ssc-corpus.jsonl

# Or generate balanced adversarial data
bashrs generate-adversarial --count 10000 -o adversarial.jsonl
```

The corpus JSONL format is simple. Note that the shell preamble (shebang, `set -euf`, `trap` cleanup) is **automatically stripped** during export, so the model sees only the safety-relevant command:

```json
{"input": "echo \"hello\"\n", "label": 0}
{"input": "mkdir -p \"$HOME/tmp\"\n", "label": 0}
{"input": "eval \"$user_input\"\n", "label": 1}
{"input": "echo $RANDOM\n", "label": 1}
```

### Step 3: Fine-Tune with LoRA

```bash
# Single GPU (auto-detected backend: CUDA > wgpu > CPU)
apr finetune --task classify --model-size 0.5B \
    ./models/qwen2.5-coder-0.5b \
    --data train.jsonl \
    --epochs 3 \
    --learning-rate 0.0002 \
    --num-classes 2 \
    -o ./ssc-checkpoints/

# Specify GPU backend (for AMD/Intel GPUs)
apr finetune --task classify --model-size 4B --gpu-backend wgpu \
    ./models/qwen3-4b \
    --data train.jsonl \
    --num-classes 2 --epochs 3 -o ./ssc-checkpoints/

# Multi-GPU data parallelism
apr finetune --task classify --model-size 4B --gpus 0,1 \
    ./models/qwen3-4b \
    --data train.jsonl \
    --num-classes 2 --epochs 3 -o ./ssc-checkpoints/
```

> **Auto-class-balancing**: entrenar automatically detects class imbalance (ratio >2:1) and applies sqrt-inverse weights when no explicit weights are configured. For the SSC v3 corpus (14.5:1 safe vs unsafe), this applies weight=7.747 to the minority class.

### Step 3.5: Hyperparameter Tuning (Optional)

Before committing to a full training run, use `apr tune` to search for optimal hyperparameters:

```bash
# Scout: quick 1-epoch sweep to find a good HP region
apr tune --task classify --budget 5 --scout --data corpus.jsonl --json

# Full: multi-epoch search with ASHA early stopping
apr tune --task classify --budget 10 --data corpus.jsonl \
    --strategy tpe --scheduler asha
```

The tuner searches over 9 parameters (learning rate, LoRA rank, alpha ratio, batch size, warmup fraction, gradient clip norm, class weights, target modules, LR min ratio) using TPE (Tree-structured Parzen Estimators). Scout mode runs 1 epoch per trial for fast exploration; full mode uses ASHA to prune unpromising trials early.

See the [entrenar classify-tune example](https://github.com/paiml/entrenar/blob/main/examples/classify_tune_demo.rs) for the programmatic API.

### Step 4: Monitor Training

```bash
# Live monitoring
apr monitor ./ssc-checkpoints/

# Or check state manually
cat ./ssc-checkpoints/training_state.json | python3 -m json.tool
```

### Step 5: Use the Trained Model

```bash
# Classify with your custom model
bashrs classify --model ./ssc-checkpoints/ script.sh
```

### Step 6: Evaluate the Checkpoint

Before publishing, evaluate the trained model against the held-out test set (produced by alimentar):

```bash
# Evaluate: text report (sklearn-style)
apr eval /tmp/ssc-checkpoints/best/ --task classify \
    --data /tmp/ssc-export/test.jsonl --model-size 0.5B --num-classes 2

# Evaluate: JSON output (machine-readable)
apr eval /tmp/ssc-checkpoints/best/ --task classify \
    --data /tmp/ssc-export/test.jsonl --model-size 0.5B --num-classes 2 --json

# Evaluate + generate HuggingFace model card
apr eval /tmp/ssc-checkpoints/best/ --task classify \
    --data /tmp/ssc-export/test.jsonl --model-size 0.5B --num-classes 2 --generate-card
```

The `--generate-card` flag writes a publication-quality `README.md` to the checkpoint directory with YAML front matter, metrics tables, confusion matrix, calibration curve, and error analysis — ready for HuggingFace upload.

#### Evaluation Metrics

The evaluation harness computes 13 metrics across 4 categories:

**Accuracy & Agreement**

| Metric | Description |
|--------|-------------|
| Accuracy | Overall correct classification rate |
| Top-2 Accuracy | Correct class in top 2 predictions |
| Cohen's Kappa | Chance-corrected agreement (>0.6 = substantial) |
| MCC | Matthews Correlation Coefficient (balanced even with skewed classes) |

**Per-Class Performance**

| Metric | Description |
|--------|-------------|
| Precision | Of predictions for this class, how many were correct |
| Recall | Of actual instances, how many were found |
| F1 | Harmonic mean of precision and recall |
| Support | Number of true instances per class |

**Proper Scoring Rules**

| Metric | Description |
|--------|-------------|
| Brier Score | Mean squared error of probability estimates (lower = better) |
| Log Loss | Negative log-likelihood of correct class (lower = better) |

**Calibration & Confidence**

| Metric | Description |
|--------|-------------|
| ECE | Expected Calibration Error (|confidence - accuracy| per bin) |
| Mean Confidence | Average predicted probability for chosen class |
| Confidence Gap | Difference between correct and incorrect prediction confidence |

All accuracy-type metrics include bootstrap 95% confidence intervals (1,000 resamples with deterministic LCG PRNG). Baselines are reported for context: random (1/K) and majority-class accuracy.

#### Example Output (Text Report)

```text
=== Classification Report ===

                precision    recall  f1-score   support
              safe    0.8022    0.5840    0.6759       125
     needs-quoting    0.5000    0.0526    0.0952        38
 non-deterministic    0.5423    0.7624    0.6337       101
    non-idempotent    0.5389    0.8333    0.6545       108
            unsafe    0.7188    0.5391    0.6161       128
 ----------------------------------------------------------
         macro avg    0.6204    0.5543    0.5351       500
      weighted avg    0.6329    0.6220    0.6033       500

Accuracy: 62.20% [57.80%, 66.80%]
Cohen's kappa: 0.5124 (moderate)
MCC: 0.5241 [0.4701, 0.5793]
Macro F1: 0.5351 [0.4901, 0.5791]

Brier Score: 0.6077 (lower is better)
Log Loss: 1.8209 (lower is better)

Baselines: random=20.0%, majority=25.6%, model=62.2% (2.4x lift over majority)

Top confused pairs:
  safe → non-deterministic: 28
  unsafe → non-idempotent: 24
  safe → non-idempotent: 22
```

### Step 7: Publish to HuggingFace

After evaluation, publish the adapter to HuggingFace Hub:

```bash
# Set HuggingFace token
export HF_TOKEN=hf_xxxxxxxxxxxxx

# Publish adapter + classifier head + config + model card
apr publish ./ssc-checkpoints/ paiml/shell-safety-classifier

# Or manual upload
huggingface-cli upload paiml/shell-safety-classifier ./ssc-checkpoints/
```

This uploads only the LoRA adapter and classifier head — not a full copy of the base model. Users load `Qwen/Qwen2.5-Coder-0.5B` (already public) and apply the adapter on top.

The auto-generated model card (from `--generate-card`) includes:
- YAML front matter with `model-index` metrics (accuracy, F1, MCC, kappa)
- Summary table with 95% confidence intervals
- Per-class precision/recall/F1 table
- Confusion matrix (raw counts + normalized percentages)
- Error analysis (top-5 most confused class pairs)
- Calibration curve (reliability diagram with 8 bins)
- Baseline comparisons (random, majority, model lift)
- Training details (framework, method, base model)
- Intended use, limitations, and ethical considerations
- License (Apache-2.0)

## Sovereign Stack

The entire training and inference pipeline runs without Python or PyTorch:

| Layer | Crate | Role |
|-------|-------|------|
| **trueno** | SIMD + GPU compute | Tensor operations (5 SIMD backends + wgpu + CUDA), `GpuCommandBatch` batched matmul, `GpuDevicePool` multi-GPU |
| **aprender** | ML framework | Autograd, optimizers, loss functions, SafeTensors I/O |
| **entrenar** | Training engine | Transformer, LoRA/QLoRA, AdamW, `ClassifyPipeline`, `WgpuForwardPass`, `DataParallelCoordinator` |
| **alimentar** | Data pipeline | Stratified train/test/val splitting, data loading |
| **realizar** | Inference engine | CUDA-accelerated model serving |
| **bashrs** | Training data | 17,942-entry corpus (v3) + binary label derivation via `classify_single()` |

Model checkpoints are saved in APR format with full lifecycle support:
- **`.adapter.apr`** (deploy): LoRA + classifier head, no optimizer state. Used by `realizar` for inference.
- **`.ckpt.apr`** (resume): Full training state including optimizer moments. Enables `--resume`.
- **SafeTensors** (interop): HuggingFace-compatible export. Anyone can load without our tooling.

The checkpoint system implements 18 provable contracts (F-CKPT-001..018) covering atomic writes, NaN guards, shape validation, canonical ordering, filtered reader for inference, and round-trip bit-identity. See the [APR Checkpoint Specification](https://github.com/paiml/aprender/blob/main/docs/specifications/apr-checkpoints.md) for details.

## Comparison: Rule-Based vs Neural

| Aspect | Rule-Based | Neural (LoRA) |
|--------|-----------|---------------|
| **Latency** | <1ms | ~50ms (CPU) |
| **Accuracy (known patterns)** | ~95% | ~95% |
| **Accuracy (novel patterns)** | Low (misses unseen patterns) | High (generalizes from training) |
| **Dependencies** | None | Base model (~1GB) + adapter (~4MB) |
| **Interpretability** | Full (maps to specific lint rules) | Limited (neural attention weights) |
| **Offline** | Always | Always (local inference) |

**Recommendation**: Use rule-based for CI/CD and real-time feedback. Use neural for comprehensive security audits and classifying scripts with unusual patterns not covered by existing lint rules.

## See Also

- [Security Rules (SEC001-SEC008)](../linting/security.md)
- [Determinism Rules (DET001-DET003)](../linting/determinism.md)
- [Idempotency Rules (IDEM001-IDEM003)](../linting/idempotency.md)
- [ML Error Classification](../quality/ml-classification.md)
- [Corpus Testing](./corpus-testing.md)
