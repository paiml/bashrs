# ShellSafetyBench Training Pipeline

ShellSafetyBench is the first shell-specific security benchmark. It combines bashrs's 17,942-entry transpilation corpus with CWE-mapped mutations from verificar into a unified training dataset for shell safety models. The full pipeline runs on sovereign Rust AI tooling (no Python, no PyTorch).

## Pipeline Overview

The pipeline has six stages, defined in `configs/pipeline/ssc.yaml` and orchestrated by the `apr pipeline apply` DAG engine:

```text
Stage 2a: Corpus Export          bashrs corpus generate-conversations
Stage 2b: Verificar Mutations    verificar mutate --cwe-targets all
Stage 2c: Label                  bashrs corpus label
Stage 2d: Merge + Audit          bashrs corpus merge-data
Stage 2e: Split + Decontaminate  bashrs corpus export-splits
Stage 3:  Training               entrenar (Qwen3-4B NF4 QLoRA)
Stage 4a: Batch Inference        bashrs corpus batch-eval
Stage 4b: Evaluation             bashrs corpus eval-benchmark
Stage 5:  QA Gate                apr qa --checklist
Stage 6:  Publish                alimentar hub push + apr publish
```

Stages 2a and 2b run in parallel. Everything else is sequential with dependency gates.

## Data Pipeline

### Stage 2a: Corpus Export

Export the bashrs transpilation corpus as ChatML conversations:

```text
bashrs corpus generate-conversations --entrenar \
  --output training/shellsafetybench/conversations.jsonl
```

This generates four conversation types from the 17,942-entry corpus:

| Type | Description | Minimum % |
|------|-------------|-----------|
| A: Classify+Explain | Unsafe scripts with SEC/DET/IDEM findings | — |
| B: Fix | Unsafe scripts with auto-applied corrections | — |
| C: Debug | Non-deterministic scripts with DET diagnostics | — |
| D: Confirm Safe | Safe scripts (no findings) | >= 30% |

Each type uses 12+ phrasing variants for diversity. Quality gates enforce Type D >= 30% and no single variant > 20%.

### Stage 2b: Verificar Mutations

Generate CWE-targeted safe/unsafe script pairs using verificar's mutation engine:

```text
verificar mutate --cwe-targets all --count 10000 --seed 42 \
  --output jsonl > training/shellsafetybench/verificar-mutations.jsonl
```

12 CWE patterns are supported:

| Category | CWEs |
|----------|------|
| In-distribution | 78, 94, 330, 362, 377, 732, 798, 829 |
| Out-of-distribution (eval only) | 426, 77, 116, 250 |

The OOD CWEs are intentionally excluded from training data to test generalization.

### Stage 2c: Label

Label both data sources using bashrs's linter as the oracle:

```text
# Label corpus conversations
bashrs corpus label \
  --input training/shellsafetybench/conversations.jsonl \
  --format json \
  --output training/shellsafetybench/labeled.jsonl

# Label verificar mutations
bashrs corpus label \
  --input training/shellsafetybench/verificar-mutations.jsonl \
  --format json \
  --output training/shellsafetybench/verificar-labeled.jsonl
```

Labels are binary: safe (0) or unsafe (1), derived from linting the transpiled shell output.

### Stage 2d: Merge

Combine all labeled sources into a single shuffled dataset:

```text
bashrs corpus merge-data \
  --input training/shellsafetybench/verificar-labeled.jsonl \
  -o training/shellsafetybench/merged-training.jsonl \
  --seed 42
```

The merge command:
- Auto-loads corpus conversations from `training/shellsafetybench/conversations.jsonl`
- Accepts multiple `--input` flags for additional JSONL sources
- Normalizes verificar entries to conversation format (adds instruction/response/system/text fields)
- Tags each entry with a `source` field (bashrs-corpus / verificar)
- Applies deterministic Fisher-Yates shuffle with `--seed`

The corpus alone has 99.2% safe / 0.8% unsafe (120:1 imbalance). Verificar augmentation rebalances to approximately 78.9% safe / 21.1% unsafe across the merged 27,842 entries.

### Stage 2e: Split

Export deterministic train/val/test splits:

```text
bashrs corpus export-splits \
  --input training/shellsafetybench/merged-training.jsonl \
  --output training/shellsafetybench/splits/
```

This produces three files:

| Split | Proportion | Entries |
|-------|-----------|---------|
| `train.jsonl` | ~80% | ~22,169 |
| `val.jsonl` | ~10% | ~2,738 |
| `test.jsonl` | ~10% | ~2,935 |

Splitting uses FNV-1a hash-based assignment (mod 10: 0-7 train, 8 val, 9 test), which is deterministic and stable as the corpus grows.

Without `--input`, the command exports splits from the raw bashrs corpus only (no verificar entries).

## Training Configuration

The Qwen3-4B NF4 QLoRA training config lives at `configs/train/ssc-qwen3-4b-qlora.yaml`:

```yaml
model:
  path: "/home/noah/src/models/qwen3-4b/"
  mode: transformer
  config: "/home/noah/src/models/qwen3-4b/config.json"

data:
  train: "/home/noah/src/bashrs/training/shellsafetybench/splits/train.jsonl"
  val: "/home/noah/src/bashrs/training/shellsafetybench/splits/val.jsonl"
  tokenizer: "/home/noah/src/models/qwen3-4b/tokenizer.json"
  seq_len: 512
  batch_size: 4
  input_column: "input"

optimizer:
  name: "adamw"
  lr: 5.0e-5
  weight_decay: 0.01

training:
  mode: "causal_lm"
  epochs: 1
  gradient_accumulation: 4
  warmup_steps: 100
  lr_scheduler: "cosine"
  output_dir: "/home/noah/src/bashrs/training/checkpoints/ssc-chat-v7-qwen3-4b"
  save_interval: 500
  eval_interval: 250
  patience: 5

lora:
  enabled: true
  rank: 16
  alpha: 32.0
  target_modules: [q_proj, v_proj, o_proj, gate_proj]
  quantize_base: true
  quantize_bits: 4
  quant_type: nf4
```

Key design choices:

- **NF4 QLoRA**: The 4B base model is quantized to NF4 (~2GB), with LoRA adapters (rank 16, ~5.9M trainable params) on GPU. Total VRAM: ~9.9 GB on RTX 4090.
- **1 epoch**: Full fine-tuning beyond 1 epoch causes catastrophic forgetting (validated in runs 3-6 with the 0.5B model). LoRA preserves base instruction-following.
- **4 LoRA targets**: Q, V, O, and Gate projections. Follows the proven albor finetune-lora pattern.
- **Cosine schedule with warmup**: 100 warmup steps, then cosine decay over the single epoch.
- **Eval every 250 steps**: Decoupled from checkpoint saving (every 500 steps). Early stopping with patience=5.

## CLI Commands Reference

All commands are subcommands of `bashrs corpus`.

### generate-conversations

```text
# Generate ChatML conversations from corpus
bashrs corpus generate-conversations --output conversations.jsonl

# Entrenar-compatible format (adds text field for causal LM training)
bashrs corpus generate-conversations --entrenar --output conversations.jsonl

# Limit to first 100 entries with custom seed
bashrs corpus generate-conversations --limit 100 --seed 123 --output sample.jsonl
```

### label

```text
# Label entries with safe/unsafe using bashrs linter
bashrs corpus label --input data.jsonl --format json --output labeled.jsonl
```

Accepts both corpus conversation format and verificar mutation format. Verificar entries use the `unsafe_script` field for labeling.

### merge-data

```text
# Merge corpus + verificar into unified dataset
bashrs corpus merge-data \
  --input verificar-labeled.jsonl \
  -o merged.jsonl \
  --seed 42
```

### export-splits

```text
# Export splits from merged data
bashrs corpus export-splits \
  --input merged.jsonl \
  --output ./splits/

# Export splits from raw corpus only (no --input)
bashrs corpus export-splits --output ./splits/

# Show split statistics without writing files
bashrs corpus export-splits
```

### export-benchmark

```text
# Export DPO-compatible benchmark JSONL
bashrs corpus export-benchmark --output benchmark.jsonl

# Export first 100 entries
bashrs corpus export-benchmark --limit 100
```

Each entry contains: `id`, `lang`, `cwe`, `rule`, `severity`, `script`, `chosen`, `rejected`, `source`, `conversation_type`.

### batch-eval

Run batch inference on a test split using a trained model checkpoint.
Produces predictions JSONL compatible with `eval-benchmark`.

```text
# Run model inference on test split (requires --features ml)
bashrs corpus batch-eval \
  --model /path/to/model \
  --test-data training/shellsafetybench/splits/test.jsonl \
  --output /tmp/predictions.jsonl \
  --max-tokens 128
```

The model loads once, then iterates through test entries with progress reporting.
Output is `EvalPrediction`-compatible JSONL ready for `eval-benchmark`.

### eval-benchmark

```text
# Evaluate model predictions against benchmark
bashrs corpus eval-benchmark --predictions predictions.jsonl

# JSON output for CI
bashrs corpus eval-benchmark --predictions predictions.jsonl --json
```

Predictions JSONL format:

```text
{"id":"SSB-00001","classification":"unsafe","label":1,"cited_rules":["SEC001"],"cited_cwes":["CWE-78"],"explanation":"...","ground_truth_rules":["SEC001"],"ground_truth_cwes":["CWE-78"]}
```

Six weighted metrics:

| Metric | Weight |
|--------|--------|
| Detection F1 | 25% |
| Rule Citation | 20% |
| CWE Mapping | 10% |
| Fix Validity | 15% |
| Explanation | 15% |
| OOD Generalization | 15% |

### pipeline-check

```text
# Preflight: verify all tools and configs are available
bashrs corpus pipeline-check

# JSON output for CI
bashrs corpus pipeline-check --json
```

Checks: required tools (bashrs, verificar, alimentar, shellcheck), optional tools (entrenar, apr-cli), config files, and data artifacts.

### ssc-report

```text
# Full readiness report
bashrs corpus ssc-report

# CI gate mode: exit 1 if any section fails
bashrs corpus ssc-report --gate

# JSON output
bashrs corpus ssc-report --json
```

### shellcheck-validate

```text
# Cross-validate bashrs labels against ShellCheck
bashrs corpus shellcheck-validate --samples 500

# JSON output
bashrs corpus shellcheck-validate --samples 100 --json
```

## Quality Gates and Provable Contracts

### Pre-Training Contracts

Run all validation contracts before training:

```text
bashrs corpus validate-contracts
```

| Contract | What It Checks | Threshold |
|----------|---------------|-----------|
| C-TOK-001 | Tokenizer quality on shell constructs | >= 70% acceptable |
| C-LABEL-001 | Unsafe label accuracy | >= 90% genuine |
| C-CLF-001 | Baseline MCC scores | Reference values |
| C-CLF-001-GEN | Generalization on 50 OOD scripts | >= 50% caught |
| C-DATA-001 | Dataset split proportions | 80/10/10 |

### ShellSafetyBench Contract

The `shellsafetybench-v1.yaml` contract (11 falsification tests) validates:

| Test ID | What It Proves |
|---------|---------------|
| FALSIFY-SSB-001 | CWE mapping covers all 14 linter rules |
| FALSIFY-SSB-002 | Conversations contain shell code, not Rust |
| FALSIFY-SSB-003 | Benchmark export has DPO schema |
| FALSIFY-SSB-004 | OOD CWEs disjoint from linter CWEs |
| FALSIFY-SSB-005 | CVSS scores valid (0.0-10.0) |
| FALSIFY-SSB-006 | Eval harness weights sum to 1.0 |
| FALSIFY-SSB-007 | Zero exact duplicates between train/test |
| FALSIFY-SSB-008 | Eval harness produces valid results |
| FALSIFY-SSB-009 | Pipeline preflight validates all tools |
| FALSIFY-SSB-010 | Verificar mutations label correctly |
| FALSIFY-SSB-011 | Merge normalizes verificar entries to conversation format |

Contract file: `provable-contracts/contracts/shellsafetybench-v1.yaml`

### Pipeline QA Gate

The final QA gate (`configs/qa/ssc-release-v1.yaml`) blocks publishing unless:
- Static eval on test split passes accuracy thresholds
- Dynamic eval on fresh OOD mutations passes generalization thresholds
- Baselines are computed for comparison

## Running the Full Pipeline

### One-Command Execution

```text
apr pipeline apply configs/pipeline/ssc.yaml
```

This runs all stages in dependency order with automatic failure handling.

### Manual Step-by-Step

```text
# 1. Export corpus conversations
bashrs corpus generate-conversations --entrenar \
  -o training/shellsafetybench/conversations.jsonl

# 2. Generate verificar mutations (can run in parallel with step 1)
verificar mutate --cwe-targets all --count 10000 --seed 42 \
  --output jsonl > training/shellsafetybench/verificar-mutations.jsonl

# 3. Label both sources
bashrs corpus label \
  --input training/shellsafetybench/conversations.jsonl \
  --format json -o training/shellsafetybench/labeled.jsonl

bashrs corpus label \
  --input training/shellsafetybench/verificar-mutations.jsonl \
  --format json -o training/shellsafetybench/verificar-labeled.jsonl

# 4. Merge into unified dataset
bashrs corpus merge-data \
  --input training/shellsafetybench/verificar-labeled.jsonl \
  -o training/shellsafetybench/merged-training.jsonl \
  --seed 42

# 5. Export train/val/test splits
bashrs corpus export-splits \
  --input training/shellsafetybench/merged-training.jsonl \
  -o training/shellsafetybench/splits/

# 6. Validate contracts
bashrs corpus validate-contracts

# 7. Run SSC readiness report
bashrs corpus ssc-report --gate

# 8. Train Qwen3-4B NF4 QLoRA
apr train apply --task pretrain \
  --config configs/train/ssc-qwen3-4b-qlora.yaml \
  --seed 42 --deterministic

# 9. Evaluate
apr eval training/checkpoints/ssc-chat-v7-qwen3-4b/ \
  --task classify \
  --data training/shellsafetybench/splits/test.jsonl \
  --device cuda \
  --output training/shellsafetybench/eval/static-results.json

# 10. QA gate
apr qa --checklist configs/qa/ssc-release-v1.yaml
```

## Sovereign Stack

The entire pipeline avoids Python and PyTorch:

| Crate | Role in Pipeline |
|-------|-----------------|
| **bashrs** | Corpus export, labeling, merging, splitting, eval harness |
| **verificar** | CWE-targeted mutation generation (9,900 entries) |
| **entrenar** | Transformer training (Qwen3-4B NF4 QLoRA) |
| **alimentar** | Data quality audit + HuggingFace publishing |
| **trueno** | SIMD + GPU tensor operations |
| **aprender** | ML framework (autograd, optimizers, class weights) |

## See Also

- [Shell Safety Classifier](./shell-safety-classifier.md) -- full SSC architecture and usage
- [Corpus Testing](./corpus-testing.md) -- corpus scoring and validation
- [Probar Testing](./probar-testing.md) -- WASM + contract testing
