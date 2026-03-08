# Shell Safety Classifier

bashrs includes a neural shell safety classifier that categorizes shell commands by safety risk. It combines a rule-based linter (instant, zero dependencies) with an optional transformer-based classifier for higher accuracy on novel patterns.

## SSC v11 Architecture

The Shell Safety Classifier (SSC) v11 uses a three-stage pipeline, built entirely on the sovereign Rust AI stack (no Python):

```text
Stage 0: Rule-based linter           <1ms, built-in, zero dependencies
Stage 1: CodeBERT classifier (125M)  ~20ms CPU, WASM-deployable
Stage 2: Qwen-1.5B chat model        ~2s, explains + suggests fixes
```

Each stage ships independently. The linter is always available. The CodeBERT classifier adds ML-based detection. The chat model adds natural language explanations.

### Why Three Stages?

| Stage | Speed | Accuracy (known) | Accuracy (novel) | Dependencies |
|-------|-------|-------------------|-------------------|-------------|
| 0: Linter | <1ms | ~95% | Low | None |
| 1: CodeBERT | ~20ms | ~95% | High | 125M model |
| 2: Chat | ~2s | ~95% | High + explains | 1.5B model |

The linter catches known patterns fast. CodeBERT generalizes to unseen patterns. The chat model explains why and suggests fixes.

## Safety Classes

### Binary Classification (v3, Current)

The classifier uses binary labels derived from the transpilation corpus:

| Class | Label | Index | Derivation |
|-------|-------|-------|------------|
| **Safe** | `safe` | 0 | Transpiled AND lint-clean AND deterministic |
| **Unsafe** | `unsafe` | 1 | Otherwise (failed transpilation, lint errors, or non-deterministic) |

### Rule-Based Classes (5-class, for linter output)

The rule-based classifier reports 5 severity levels for detailed diagnostics:

| Class | Index | Risk Level | Example |
|-------|-------|------------|---------|
| **Safe** | 0 | None | `echo "hello world"` |
| **Needs Quoting** | 1 | Low | `echo $HOME` |
| **Non-Deterministic** | 2 | Medium | `echo $RANDOM` |
| **Non-Idempotent** | 3 | Medium | `mkdir /tmp/build` |
| **Unsafe** | 4 | High | `eval "$user_input"` |

Priority: unsafe > non-deterministic > non-idempotent > needs-quoting > safe.

## Quick Start

### Combined Safety Check (Lint + Classify)

```bash
# Combined check: lint findings + classification in one pass
bashrs safety-check script.sh
# Output: label, confidence, and all findings

# JSON output for CI/CD integration
bashrs safety-check --json script.sh
```

### Natural-Language Explanation (SSC v11 S8.1)

```bash
# Get a full safety explanation with risk level, findings, and fix suggestions
bashrs explain script.sh

# JSON output for programmatic consumption
bashrs explain --json script.sh

# Force format detection
bashrs explain --format makefile Makefile
```

The `explain` command categorizes findings into Security, Determinism, Idempotency, and Style, then generates human-readable explanations for each issue including what the issue is, why it matters, and how to fix it. Risk levels: SAFE, LOW, MEDIUM, HIGH, CRITICAL.

```bash
cargo run -p bashrs --example explain_demo
```

### Auto-Fix (SSC v11 S8.1, Linter Spec S9)

```bash
# Apply all SAFE auto-fixes in-place (creates .bak backup)
bashrs fix script.sh

# Preview what would change without modifying
bashrs fix --dry-run script.sh

# Include SAFE-WITH-ASSUMPTIONS fixes (e.g., mkdir -p)
bashrs fix --assumptions script.sh

# Write fixed output to a different file
bashrs fix --output fixed.sh script.sh
```

### Rule-Based Classification (Built-in, Stage 0)

```bash
# Classify a single script
bashrs classify script.sh
# Output: safe (confidence: 95.0%)

# JSON output with per-class scores
bashrs classify --json script.sh

# Multi-label mode (detects ALL applicable issues)
bashrs classify --multi-label script.sh

# Classify Makefile or Dockerfile
bashrs classify Makefile
bashrs classify Dockerfile
```

### Example: Rule-Based Pipeline

Run the built-in example to see classification in action:

```bash
cargo run -p bashrs --example shell_safety_classifier
```

Output:

```text
=== Shell Safety Classifier (Rule-Based) ===

  [safe]
    Label: safe (class 0)
    Diagnostics: 1

  [needs-quoting]
    Label: needs-quoting (class 1)
    Diagnostics: 5

  [non-deterministic]
    Label: non-deterministic (class 2)
    Diagnostics: 4

  [non-idempotent]
    Label: non-idempotent (class 3)
    Diagnostics: 3

  [unsafe-eval]
    Label: unsafe (class 4)
    Diagnostics: 2
```

### Neural Classification (Stage 1: CodeBERT)

The CodeBERT classifier (125M encoder) provides ML-based classification:

```bash
# Classify with MLP probe (recommended, MCC=0.754)
bashrs classify --mlp-probe mlp_probe.json --model /path/to/codebert script.sh

# Classify with linear probe (simpler, lower accuracy)
bashrs classify --probe probe.json --model /path/to/codebert script.sh
```

CodeBERT is an encoder-only model (RoBERTa architecture) pretrained on 6 programming languages. It sees the entire input bidirectionally, making it naturally suited for classification tasks. At 125M parameters, it runs in ~20ms on CPU and fits in a browser via WASM (~125MB int8).

### Tokenizer Validation (C-TOK-001)

Before training, the CodeBERT tokenizer is validated on 20 shell constructs to ensure it handles shell syntax adequately:

```bash
# Download CodeBERT tokenizer files
huggingface-cli download microsoft/codebert-base vocab.json merges.txt config.json \
    --local-dir /tmp/codebert-base

# Run tokenizer validation (requires --features ml)
cargo run -p bashrs --features ml --example tokenizer_validation
```

**Result: 90.0% acceptable (18/20 constructs) — C-TOK-001 PASSED** (threshold: >= 70%).

The tokenizer correctly preserves: command substitution (`$(command)`), variables (`$RANDOM`), pipes (`|`), operators (`&&`, `||`), parameter expansion (`${var:-default}`), and shell keywords (`eval`, `trap`, `for`, `case`).

Two constructs fail: `2>&1` (fragmented into 4 single-char tokens) and `#!/bin/bash` (5 tokens for 1 logical unit). These are acceptable losses — the classifier sees sufficient semantic signal.

Contract: `provable-contracts/contracts/codebert-tokenizer-validation-v1.yaml`.

### Synthetic Conversation Dataset (S6)

bashrs generates ChatML instruction conversations for fine-tuning the Qwen-1.5B chat model. Four conversation types are produced:

| Type | Description | Source |
|------|-------------|--------|
| A: Classify+Explain | Unsafe scripts with findings | SEC/DET/IDEM diagnostics |
| B: Fix | Unsafe scripts with corrected version | Auto-applied safety fixes |
| C: Debug | Non-deterministic scripts | DET diagnostics |
| D: Confirm Safe | Safe scripts (>=30% of total) | No findings |

Each type uses 12+ phrasing variants for diversity. All conversations include a system prompt defining the assistant's role and limitations.

```bash
# Generate conversations from full corpus
bashrs corpus generate-conversations --output conversations.jsonl

# Publish HuggingFace-ready dataset directory
bashrs corpus publish-conversations --output /tmp/shell-safety-conversations

# View conversation example
cargo run -p bashrs --example conversation_generator
```

Quality gates (S6.4): Type D >= 30%, no empty responses, variant distribution balanced (no single variant > 20%).

## How It Works

### Stage 0: Rule-Based Pipeline

```text
script.sh
    |
    v
lint_shell()  -->  SEC001-024 (security)
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

### Stage 1: CodeBERT Classifier

```text
Input script --> RoBERTa BPE tokenizer --> CodeBERT (125M, frozen or fine-tuned)
                                                |
                                           768-dim [CLS] embedding
                                                |
                                           Linear(768, 2) --> [p_safe, p_unsafe]
```

The classifier uses an escalation ladder (cheapest first):

| Level | Approach | Params Trained | Time |
|-------|----------|---------------|------|
| 0 | Linear probe (frozen CodeBERT) | 769 | Seconds |
| 0.5 | **MLP probe (frozen CodeBERT)** | **25,345** | **Seconds** |
| 1 | Fine-tune top-2 layers + head | ~15M | ~30 min |
| 2 | Full fine-tune all layers | 125M | ~1 hr |
| 3 | Continue-pretrain on shell + fine-tune | 125M | ~4 hrs |

Level 0.5 (MLP probe) is the recommended starting point for shell-based labels. The 2-layer MLP with ReLU captures non-linear patterns in CodeBERT embeddings that a linear probe misses. With adversarial augmentation, it achieves MCC=0.754.

### Stage 2: Chat Model

Qwen2.5-Coder-1.5B-Instruct + LoRA, trained on synthetic conversations derived from Stage 1 confidence scores + bashrs linter findings:

```bash
# Rule-based analysis (always available, <1ms)
bashrs explain script.sh

# ML-powered analysis with chat model (requires trained model + ml feature)
bashrs explain script.sh --chat-model /path/to/shell-safety-chat/

# Rule-based autofix (always available)
bashrs fix script.sh

# ML-powered fix suggestions with chat model
bashrs fix script.sh --chat-model /path/to/shell-safety-chat/
```

The `--chat-model` flag loads a Qwen-1.5B + LoRA model via entrenar's `InstructPipeline`. It formats the script + linter findings as a ChatML prompt and generates a natural-language response. Without `--chat-model`, the commands fall back to rule-based analysis (Stage 0).

Requires the `ml` feature: `cargo install bashrs --features ml`

## Training Data

The classifier trains on bashrs's 17,942-entry transpilation corpus:

| Format | Entries |
|--------|---------|
| Bash | ~16,431 |
| Makefile | ~804 |
| Dockerfile | ~707 |
| **Total** | **17,942** |

Class distribution: 99.2% safe, 0.8% unsafe (120:1 imbalance). Labels are derived from linting the **transpiled shell output** (not the Rust source code), so training distribution matches inference (#172). Class weights auto-applied via sqrt-inverse.

### Data Pipeline

```text
bashrs (export) --> alimentar (split 80/10/10) --> entrenar (train)
```

### Synthetic Conversations (Stage 2 Training Data)

The `corpus::conversations` module generates chat training data from corpus entries:

```bash
# Generate conversations from full corpus
bashrs corpus generate-conversations --output conversations.jsonl

# Generate from first 100 entries with custom seed
bashrs corpus generate-conversations --limit 100 --seed 123 --output sample.jsonl
```

Four conversation types, 10+ phrasing variants each:

| Type | Purpose | Template |
|------|---------|----------|
| A: Classify+Explain | Unsafe scripts with findings | "Is this script safe?" -> findings list |
| B: Fix | Unsafe scripts with correction | "Fix this script" -> corrected version |
| C: Debug | Non-deterministic scripts | "Why different results?" -> DET findings |
| D: Confirm Safe | Safe scripts (>=30% required) | "Check this script" -> confirmation |

Quality gates (SSC v11 Section 6.4): rule citation accuracy 100%, Type D >= 30%, no variant > 20%.

### Baseline Classifiers (Section 5.5)

Three baseline classifiers that any ML classifier must beat:

```bash
# Run all three baselines
bashrs corpus baselines

# Run the baselines example
cargo run -p bashrs --example baselines
```

| Baseline | Strategy | MCC |
|----------|----------|-----|
| Majority class | Always predict "safe" | 0.000 |
| Keyword regex | Pattern match on 17 unsafe keywords | 0.103 |
| bashrs linter | Use 24 SEC + DET/IDEM rules as classifier | 1.000 (tautological) |

The linter baseline achieves MCC=1.000 because labels are derived from linter findings (tautological). The keyword baseline (MCC=0.103) is the realistic target to beat. Contract C-CLF-001 requires MCC > 0.3 on test set.

### Label Audit (Section 5.3, C-LABEL-001)

Validates that "unsafe" labels are genuinely unsafe (not transpiler-limitation false positives):

```bash
# Audit first 100 unsafe labels
bashrs corpus label-audit

# Audit with custom limit
bashrs corpus label-audit -n 200

# Run the label audit example
cargo run -p bashrs --example label_audit
```

Multi-signal validation checks: linter findings, 24 known unsafe patterns (SEC001-SEC024), structural checks (non-idempotent, unquoted variables). Target: >= 90% accuracy (C-LABEL-001).

### Generalization Tests (Section 5.6)

50 hand-written OOD (out-of-distribution) scripts with no lexical overlap with training data:

```bash
# Run generalization tests
bashrs corpus generalization-tests

# Run the example
cargo run -p bashrs --example generalization_tests
```

6 categories: injection (10), non-determinism (10), race-condition (10), privilege (10), exfiltration (5), destructive (5). Target: linter catches >= 50%. Current: **100% (50/50)** with SEC020-SEC024 extended rules + DET004 system state detection.

### Contract Validation (Pre-Training Gate)

Run all SSC contracts in a single pass before proceeding to classifier training:

```bash
# Run all contracts
bashrs corpus validate-contracts

# Run the example
cargo run -p bashrs --example contract_validation
```

| Contract | What It Checks | Threshold |
|----------|---------------|-----------|
| C-TOK-001 | Tokenizer quality on shell constructs | >= 70% acceptable |
| C-LABEL-001 | Unsafe label accuracy | >= 90% genuine |
| C-CLF-001 | Baseline MCC scores (majority, keyword, linter) | Reference |
| C-CLF-001-GEN | Generalization on 50 OOD scripts | >= 50% caught |
| C-DATA-001 | Dataset split proportions (80/10/10) | Valid |

### Dataset Export with Splits

Export the corpus as train/val/test JSONL files for ML training:

```bash
# Show split statistics
bashrs corpus export-splits

# Write split files to directory
bashrs corpus export-splits --output ./data/splits/
```

Uses FNV-1a hash-based deterministic splitting (80/10/10, seed-stable across corpus growth).

### Model Card Generation (S6.5)

Generate a HuggingFace-compatible model card with YAML front matter, dataset statistics, class weights, baseline performance, and honesty requirements:

```bash
# Print model card to stdout
bashrs corpus model-card

# Write to file
bashrs corpus model-card --output README.md
```

The model card includes:
- YAML front matter (HuggingFace dataset card schema)
- Live dataset statistics (entries, class distribution, splits)
- Computed class weights (sqrt-inverse for imbalanced training)
- Baseline classifier performance (majority, keyword, linter)
- Honesty requirements per SSC v11 S6.5:
  - Trained on synthetic data derived from rule-based linter output
  - Explains known patterns, not novel safety reasoning
  - Not a replacement for security audit

### Training Configuration (S9, CLF-001)

Export an entrenar-compatible training configuration:

```bash
# YAML format (default)
bashrs corpus training-config

# JSON format
bashrs corpus training-config --json

# Write to file
bashrs corpus training-config --output config.yaml
```

The config includes model architecture (CodeBERT encoder, 768-dim, 12 layers), training hyperparameters (epochs, batch size, learning rate, class weights), data statistics, and evaluation targets (MCC CI lower > 0.2, accuracy > 93.5%).

### Publish Dataset (S9)

Generate a complete HuggingFace-ready dataset directory with all artifacts:

```bash
# Create dataset directory
bashrs corpus publish-dataset --output ./ssc-dataset/
```

This creates:
- `README.md` — HuggingFace model card with YAML front matter
- `train.jsonl` — Training split (~80%)
- `val.jsonl` — Validation split (~10%)
- `test.jsonl` — Test split (~10%)
- `training_config.yaml` — entrenar-compatible training configuration

Upload to HuggingFace Hub:

```bash
huggingface-cli upload paiml/shell-safety-classifier ./ssc-dataset/
```

### Data Pipeline Example

Run the complete data pipeline example:

```bash
cargo run -p bashrs --example ssc_data_pipeline
```

This demonstrates: dataset overview -> train/val/test split -> training config -> model card generation.

### SSC Status Report

Generate a comprehensive readiness report covering all SSC validation sections:

```bash
# Full SSC readiness report
bashrs corpus ssc-report

# CI gate mode: exit 1 if any section fails
bashrs corpus ssc-report --gate

# JSON output for programmatic consumption
bashrs corpus ssc-report --json

# Run the example
cargo run -p bashrs --example ssc_report
```

The report covers 8 sections:

| Section | Spec Ref | What It Checks |
|---------|----------|---------------|
| Corpus | S5.3 | Entry count >= 17,000, format distribution |
| Tokenizer (C-TOK-001) | S5.2 | >= 70% constructs acceptable |
| Label Audit (C-LABEL-001) | S5.3 | >= 90% accuracy, false positives <= 10% |
| Baselines (C-CLF-001) | S5.5 | MCC + accuracy + recall per baseline, ML targets |
| Generalization (OOD) | S5.6 | >= 50% of 50 OOD scripts caught |
| Dataset Splits | S5.3 | 80/10/10 train/val/test proportions |
| Conversations (S6) | S6 | Type A/B/C/D breakdown, >=30% Type D, variant balance |
| Data Pipeline (S9) | S9 | Model card, honesty, class weights, config |

Overall readiness: YES when no section has FAIL status.

### Classifier Pipeline (CLF-RUN)

The CLF-RUN pipeline trains a linear probe on frozen CodeBERT embeddings. Three steps:

**Step 1: Extract [CLS] embeddings** (requires `--features ml`):

```bash
# Extract embeddings from CodeBERT for all corpus entries
bashrs corpus extract-embeddings \
  --model /path/to/codebert-base/ \
  --output embeddings.jsonl
```

This loads CodeBERT (124M params, 199 safetensors), tokenizes each corpus entry, runs a forward pass, and saves the 768-dimensional [CLS] embedding per entry. Use `--limit N` to test with a subset first.

**Validated performance** (release build, CPU):
- Model loading: ~23s (476MB safetensors)
- Extraction rate: ~1.82 entries/s
- Full corpus (17,942 entries): ~4 hours

**Step 2: Train linear probe + evaluate**:

```bash
# Train on cached embeddings (no GPU needed)
bashrs corpus train-classifier \
  --embeddings embeddings.jsonl \
  --output ./classifier/ \
  --epochs 100 \
  --seed 42

# MLP probe with adversarial augmentation (recommended for shell-based labels)
bashrs corpus train-classifier \
  --embeddings embeddings.jsonl \
  --augment adversarial.jsonl \
  --output ./classifier/ \
  --epochs 50 \
  --learning-rate 0.0001 \
  --mlp --mlp-hidden 32
```

Outputs:
- `probe.json` — Trained weights (769 parameters: 768 weights + 1 bias)
- `evaluation.json` — Test set MCC, accuracy, precision, recall, F1

**Full pipeline** (extract + train + evaluate in one command):

```bash
bashrs corpus run-classifier \
  --model /path/to/codebert-base/ \
  --output ./classifier/ \
  --epochs 30 \
  --seed 42
```

**Ship Gate C-CLF-001**: The classifier must beat the keyword baseline:
- MCC > 0.3 (beats keyword regex baseline MCC=0.103)
- Note: linter baseline MCC=1.000 is tautological (labels derived from linter)

**Validated results** (Level 0 linear probe, RoBERTa BPE tokenizer, class-weighted online SGD):

Shell-based labels (#172 fix — correct domain):

| Entries | Probe | Test MCC | Accuracy | Precision | Recall | Ship Gate |
|---------|-------|----------|----------|-----------|--------|-----------|
| 3000 (shell) | Linear | 0.043 | 94.7% | 0.040 | 0.111 | FAIL |
| 3000 + 350 adv | Linear | 0.205 | 36.5% | 0.146 | 1.000 | FAIL |
| 3000 + 350 adv | **MLP h=32** | **0.754** | 94.2% | 0.670 | 0.918 | **PASS** |
| **17942 + 350 adv** | **MLP h=32** | **0.443** | 93.0% | 0.248 | 0.870 | **PASS** |

The MLP probe (Level 0.5) with adversarial augmentation is the recommended configuration. Use `--mlp --mlp-hidden 32 --augment adversarial.jsonl`. At full corpus scale (17,942 entries), MCC=0.443 with 87% recall — the classifier catches most unsafe scripts while maintaining acceptable precision.

Training uses sqrt-inverse balanced class weights (aprender `ClassWeight::Balanced`)
and L2 regularization (weight_decay=1e-4) to handle 99.2/0.8% safe/unsafe imbalance.

**Known limitation**: entries 3000+ have zero unsafe labels (data labeling gap, [#171](https://github.com/paiml/bashrs/issues/171)). Use `--max-entries 2500` when training on full extraction to maintain MCC > 0.3. The #172 fix ensures training data uses transpiled shell output (matching inference input), not Rust source code.

## WASM Deployment

The `bashrs-wasm` crate provides browser-native shell safety analysis. The WASM binary
is 1.5MB in release mode and runs the full linter in <10ms per analysis.

**Live**: [interactive.paiml.com/shell-safety/](https://interactive.paiml.com/shell-safety/)

### shell-safety.html

The interactive app provides:

- **ScriptEditor**: Split-pane editor with 150ms debounce analysis
- **Format selector**: Bash, Makefile, or Dockerfile linting
- **Classification**: Rule-based SAFE/UNSAFE label with confidence score
- **Diagnostics**: Per-line findings with severity, code, and message
- **Zero latency**: No network calls — all analysis runs locally in WASM

```bash
# Build and serve locally
wasm-pack build bashrs-wasm --target web --release
cd bashrs-wasm && ruchy serve --port 8000
# Open http://localhost:8000/shell-safety.html

# Deploy to production
aws s3 sync bashrs-wasm/pkg/ s3://interactive.paiml.com-production-mces4cme/shell-safety/pkg/ \
  --content-type "application/wasm" --exclude "*.js"
aws s3 cp bashrs-wasm/shell-safety.html \
  s3://interactive.paiml.com-production-mces4cme/shell-safety/index.html
aws cloudfront create-invalidation --distribution-id ELY820FVFXAFF --paths "/shell-safety/*"
```

### WASM API

```javascript
import init, {
  lint_shell_wasm,
  lint_makefile_wasm,
  lint_dockerfile_wasm,
  classify_shell_wasm,
  bashrs_version,
} from './pkg/bashrs_wasm.js';

await init();
const result = JSON.parse(lint_shell_wasm('eval "$1"'));
// { diagnostics: [{ code: "SEC001", severity: "Error", message: "...", line: 1 }], count: 1 }
```

### CodeBERT in WASM: Kill Criterion Triggered

A pure-Rust CodeBERT encoder was implemented in `bashrs-wasm` (WASM-004) and
benchmarked honestly. The result: **2.7s for 33 tokens on native CPU** (release mode),
estimated 5-13s in WASM. This exceeds the 2s kill threshold from the spec.

**Decision**: Ship CLI only for CodeBERT classification. The browser app uses the
rule-based linter, which runs in <10ms. The encoder code remains in the crate
behind the `codebert` feature flag for future optimization or WebGPU acceleration.

```bash
# CodeBERT classification is CLI-only
bashrs classify --probe probe.json --model /path/to/codebert/ script.sh
```

### Probar Testing

All WASM testing uses Probar (`jugar-probar`), not Playwright. Three test layers:

| Layer | Tests | Status | Runs On |
|-------|-------|--------|---------|
| Logic (no browser) | 14 | ✅ Done | Every commit |
| Docker cross-browser | 3 | Planned | Pre-release |
| Performance benchmarks | 5 | ✅ Done | Every commit |

Run the Probar test suite:

```bash
# Layer 1 + Layer 3 (no browser needed)
cargo test -p bashrs-wasm --test probar_shell_safety

# With CodeBERT tests (requires WASM-002/004)
cargo test -p bashrs-wasm --test probar_shell_safety --features codebert

# Docker cross-browser (requires Docker daemon)
cargo test -p bashrs-wasm --test probar_shell_safety --features docker
```

Performance budgets enforced by tests:

| Budget | Target | Test |
|--------|--------|------|
| Linter latency | < 10ms | `test_prb005_linter_wasm_latency_under_10ms` |
| Classify latency | < 10ms | `test_prb005_classify_wasm_latency_under_10ms` |
| Explain latency | < 10ms | `test_prb005_explain_wasm_latency_under_10ms` |
| Full pipeline | < 30ms | `test_prb005_full_linter_pipeline_under_30ms` |
| Multi-format | < 30ms | `test_prb005_multiformat_lint_latency` |

Provable contract: `probar-shell-safety-v1.yaml` with 9 falsification tests.

See `docs/specifications/shell-safety-inference.md` Section 8.4 for the complete Probar design.

## Sovereign Stack

The entire pipeline runs without Python or PyTorch:

| Crate | Role |
|-------|------|
| **trueno** | SIMD + GPU tensor operations |
| **aprender** | ML framework (autograd, optimizers) |
| **entrenar** | Training engine (Transformer, LoRA) |
| **alimentar** | Data pipeline (stratified splitting) |
| **realizar** | Inference engine |
| **bashrs** | Training data (17,942-entry corpus) |

## Provable Contracts

The encoder, classifier, and inference code are backed by YAML contracts:

| Contract | What It Proves |
|----------|---------------|
| `bidirectional-attention-v1` | Full attention (no causal mask) |
| `learned-position-embedding-v1` | Absolute position lookup (bounds-checked) |
| `encoder-forward-v1` | Shape preservation, no NaN/Inf |
| `linear-probe-classifier-v1` | Frozen encoder, probability simplex |
| `classifier-pipeline-v1` | Embedding extraction, split determinism, MLP probe convergence, ship gate |
| `chat-inference-pipeline-v1` | Autoregressive generation termination, valid token range, greedy determinism, feature gate |
| `wasm-linter-v1` | WASM binary size <5MB, valid JSON output, safe/unsafe classification |
| `probar-shell-safety-v1` | Layer 1 logic tests, Layer 3 perf budgets, determinism (C-PRB-001/003/007) |

Contract files live in `provable-contracts/contracts/`. Pipeline: YAML contract -> scaffold -> proptest/Kani harnesses -> binding to real code -> audit.

All contracts are bound with falsification tests + proptests. `pv audit` reports 0 findings on all contracts.

## Cross-Format Support

```bash
# Bash scripts
bashrs classify script.sh

# Makefiles
bashrs classify Makefile

# Dockerfiles
bashrs classify Dockerfile
```

Format is auto-detected. Use `--format` to override.

## ShellSafetyBench (v12)

ShellSafetyBench is the first shell-specific security benchmark + model, included in bashrs.

### CWE Taxonomy Mapping

Every linter rule maps to a MITRE CWE identifier with CVSS v3.1 scores:

```bash
# View the CWE mapping table
bashrs corpus cwe-mapping

# Get mapping as JSON (for pipeline consumption)
bashrs corpus cwe-mapping --json
```

14 rules cover 8 unique CWEs, plus 4 OOD (out-of-distribution) CWEs for eval-only generalization testing.

### Benchmark Export

Export the corpus in DPO-compatible format for model training:

```bash
# Export full benchmark
bashrs corpus export-benchmark -o training/shellsafetybench/benchmark.jsonl

# Export first 100 entries
bashrs corpus export-benchmark --limit 100
```

Each entry contains: `id`, `lang`, `cwe`, `rule`, `severity`, `script`, `chosen`, `rejected`, `source`, `conversation_type`.

### Eval Harness

Six weighted metrics (S14.5):

| Metric | Weight | Description |
|--------|--------|-------------|
| Detection F1 | 25% | Binary safe/unsafe classification |
| Rule Citation | 20% | Correct rule ID cited |
| CWE Mapping | 10% | Correct CWE ID referenced |
| Fix Validity | 15% | Suggested fix removes vulnerability |
| Explanation | 15% | Coherent natural-language explanation |
| OOD Generalization | 15% | Performance on novel CWE patterns |

### Pipeline (Sovereign Tooling Only)

The full pipeline is defined in `configs/pipeline/ssc.yaml`:

```bash
# One-command execution (when apr-cli is available)
apr pipeline apply configs/pipeline/ssc.yaml

# Manual step-by-step:
bashrs corpus generate-conversations --entrenar -o conversations.jsonl
bashrs corpus export-benchmark -o benchmark.jsonl
bashrs corpus export-splits -o training/shellsafetybench/splits/
```

## See Also

- [Security Rules (SEC001-SEC024)](../linting/security.md)
- [Determinism Rules (DET001-DET003)](../linting/determinism.md)
- [Idempotency Rules (IDEM001-IDEM003)](../linting/idempotency.md)
- [Probar Testing](./probar-testing.md)
- [Corpus Testing](./corpus-testing.md)
