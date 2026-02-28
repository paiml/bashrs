# Shell Safety Classifier

bashrs includes a neural shell safety classifier that categorizes shell commands into 5 risk levels. It combines a rule-based linter (instant, zero dependencies) with an optional transformer-based classifier (Qwen2.5-Coder-0.5B + LoRA) for higher accuracy on novel patterns.

## Safety Classes

Every shell command falls into one of 5 safety categories:

| Class | Index | Risk Level | Description | Example |
|-------|-------|------------|-------------|---------|
| **Safe** | 0 | None | Deterministic, idempotent, properly quoted | `echo "hello world"` |
| **Needs Quoting** | 1 | Low | Unquoted variable expansion risks word splitting | `echo $HOME` |
| **Non-Deterministic** | 2 | Medium | Output varies between runs | `echo $RANDOM`, `date +%s` |
| **Non-Idempotent** | 3 | Medium | Unsafe to re-run (missing safety flags) | `mkdir /tmp/build` |
| **Unsafe** | 4 | High | Security violations (injection, privilege escalation) | `eval "$user_input"` |

### Priority Order

When a command exhibits multiple issues, the highest-severity class wins:

```
unsafe > non-deterministic > non-idempotent > needs-quoting > safe
```

For example, `eval $RANDOM` is classified as **unsafe** (class 4), not non-deterministic, because the `eval` injection risk is more severe.

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

```
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

The neural classifier leverages pretrained code understanding from Qwen2.5-Coder-0.5B:

```
script.sh
    |
    v
BPE Tokenizer (151,665 tokens)
    |
    v
Qwen2.5-Coder-0.5B (frozen, 494M params)
  + LoRA adapters (Q/V projections, ~1.1M trainable params)
    |
    v
Mean pooling over sequence
    |
    v
Classification head: Linear(896 -> 5)
    |
    v
Softmax -> 5-class probability distribution
```

The model was fine-tuned on bashrs's 29,307-entry corpus using LoRA (Low-Rank Adaptation) on the query and value attention projections. Only ~0.2% of parameters are trained, keeping the base model's code understanding intact while learning shell safety patterns.

## Training Data

The classifier is trained on bashrs's transpilation corpus, which contains 29,307 shell script entries across three formats:

| Format | Entries | Source |
|--------|---------|--------|
| Bash | ~16,431 | Core transpilation corpus |
| Makefile | ~804 | Makefile purification corpus |
| Dockerfile | ~707 | Dockerfile purification corpus |
| Adversarial | ~10,000+ | Generated adversarial samples (balanced across classes) |

### Class Distribution (Training Set)

| Class | Label | Count | Source |
|-------|-------|-------|--------|
| 0 | safe | ~17,252 | Transpiler-verified clean output |
| 1 | needs-quoting | ~2,402 | Unquoted variable patterns |
| 2 | non-deterministic | ~2,858 | $RANDOM, timestamps, PIDs |
| 3 | non-idempotent | ~2,875 | Missing -p/-f flags |
| 4 | unsafe | ~3,920 | SEC rule violations, eval, injection |

Labels are derived automatically from bashrs's linter analysis of each corpus entry. The adversarial generator (`bashrs generate-adversarial`) produces balanced samples for underrepresented classes.

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
| Output dimension | 5 (safety classes) |
| Parameters | 4,485 |
| Activation | None (logits -> softmax at inference) |

### Training Configuration

| Parameter | Value |
|-----------|-------|
| Optimizer | AdamW |
| Learning rate | 1e-4 (with warmup) |
| Epochs | 3 |
| Batch size | 40 |
| Max sequence length | 512 |
| Gradient clipping | max_norm=1.0 |
| Loss function | CrossEntropyLoss |
| Device | NVIDIA RTX 4090 (25.2 GB VRAM) |

## Inference

### Performance

| Mode | Latency | Dependencies |
|------|---------|-------------|
| Rule-based | <1ms | None (built into bashrs) |
| Neural (CPU) | ~50ms | Base model + LoRA adapter |
| Neural (GPU) | ~5ms | CUDA-capable GPU |

The rule-based classifier is always available and fast enough for real-time use (linting, CI/CD pipelines). The neural classifier provides higher accuracy on novel patterns at the cost of model loading overhead.

### Deployment

The fine-tuned model is published to HuggingFace as [`paiml/shell-safety-classifier`](https://huggingface.co/paiml/shell-safety-classifier). It ships as a LoRA adapter (~4MB) on top of the public Qwen2.5-Coder-0.5B base model:

```
paiml/shell-safety-classifier/
  adapter.safetensors              # LoRA adapter weights (~4MB)
  classifier_head.safetensors      # Linear(896->5) classification head
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

The corpus JSONL format is simple:

```json
{"input": "#!/bin/sh\necho \"hello\"\n", "label": 0}
{"input": "#!/bin/sh\necho $HOME\n", "label": 1}
{"input": "#!/bin/sh\necho $RANDOM\n", "label": 2}
{"input": "#!/bin/sh\nmkdir /tmp/build\n", "label": 3}
{"input": "#!/bin/sh\neval \"$user_input\"\n", "label": 4}
```

### Step 3: Fine-Tune with LoRA

```bash
apr finetune --task classify --model-size 0.5B \
    ./models/qwen2.5-coder-0.5b \
    --data corpus.jsonl \
    --epochs 3 \
    --learning-rate 0.0001 \
    --num-classes 5 \
    -o ./ssc-checkpoints/
```

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

### Step 6: Publish to HuggingFace

After training completes, publish the adapter to HuggingFace Hub:

```bash
# Set HuggingFace token
export HF_TOKEN=hf_xxxxxxxxxxxxx

# Publish adapter + classifier head + config
apr publish ./ssc-checkpoints/ paiml/shell-safety-classifier

# Or manual upload
huggingface-cli upload paiml/shell-safety-classifier ./ssc-checkpoints/
```

This uploads only the LoRA adapter and classifier head — not a full copy of the base model. Users load `Qwen/Qwen2.5-Coder-0.5B` (already public) and apply the adapter on top.

The published model card includes:
- Training data description (29,307 bashrs corpus entries)
- Per-class accuracy and F1 scores
- Usage examples with bashrs CLI
- Architecture details (LoRA rank, alpha, target modules)
- License (MIT)

## Sovereign Stack

The entire training and inference pipeline runs without Python or PyTorch:

| Layer | Crate | Role |
|-------|-------|------|
| **trueno** | SIMD + GPU compute | Tensor operations (5 SIMD backends + wgpu) |
| **aprender** | ML framework | Autograd, optimizers, loss functions, SafeTensors I/O |
| **entrenar** | Training engine | Transformer, LoRA/QLoRA, AdamW, ClassifyPipeline |
| **realizar** | Inference engine | CUDA-accelerated model serving |
| **bashrs** | Training data | 29,307-entry corpus + linter-based label derivation |

Model checkpoints are saved in dual format:
- **APR** (native): Used by `realizar` for inference. Proves the stack is self-sufficient.
- **SafeTensors** (interop): HuggingFace-compatible. Anyone can load without our tooling.

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
