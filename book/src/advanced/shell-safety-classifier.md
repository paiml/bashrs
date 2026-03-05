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
# Classify using CodeBERT model (~20ms)
bashrs classify --model codebert script.sh

# Combined: linter + CodeBERT
bashrs check script.sh
```

CodeBERT is an encoder-only model (RoBERTa architecture) pretrained on 6 programming languages. It sees the entire input bidirectionally, making it naturally suited for classification tasks. At 125M parameters, it runs in ~20ms on CPU and fits in a browser via WASM (~125MB int8).

## How It Works

### Stage 0: Rule-Based Pipeline

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
| 0 | Linear probe (frozen CodeBERT) | 1,538 | Seconds |
| 1 | Fine-tune top-2 layers + head | ~15M | ~30 min |
| 2 | Full fine-tune all layers | 125M | ~1 hr |
| 3 | Continue-pretrain on shell + fine-tune | 125M | ~4 hrs |

### Stage 2: Chat Model

Qwen2.5-Coder-1.5B-Instruct + LoRA, trained on synthetic conversations derived from Stage 1 confidence scores + bashrs linter findings:

```bash
# Interactive analysis
bashrs explain script.sh

# Suggest fixes
bashrs fix script.sh
```

## Training Data

The classifier trains on bashrs's 17,942-entry transpilation corpus:

| Format | Entries |
|--------|---------|
| Bash | ~16,431 |
| Makefile | ~804 |
| Dockerfile | ~707 |
| **Total** | **17,942** |

Class distribution: 93.5% safe, 6.5% unsafe (14.5:1 imbalance). Class weights auto-applied via sqrt-inverse.

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

| Baseline | Strategy | Expected MCC |
|----------|----------|-------------|
| Majority class | Always predict "safe" | 0.0 |
| Keyword regex | Pattern match on 17 unsafe keywords | ~0.3-0.5 |
| bashrs linter | Use 14 SEC/DET rules as classifier | ~0.4-0.6 |

Contract C-CLF-001 requires any ML classifier to beat all three baselines on MCC, achieve accuracy > 93.5%, and generalization >= 50%.

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

Multi-signal validation checks: linter findings, 14 known unsafe patterns, structural checks (non-idempotent, unquoted variables). Target: >= 90% accuracy (C-LABEL-001).

### Generalization Tests (Section 5.6)

50 hand-written OOD (out-of-distribution) scripts with no lexical overlap with training data:

```bash
# Run generalization tests
bashrs corpus generalization-tests

# Run the example
cargo run -p bashrs --example generalization_tests
```

6 categories: injection (10), non-determinism (10), race-condition (10), privilege (10), exfiltration (5), destructive (5). Target: linter catches >= 50%.

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

### SSC Status Report

Generate a comprehensive readiness report covering all SSC validation sections:

```bash
# Full SSC readiness report
bashrs corpus ssc-report

# Run the example
cargo run -p bashrs --example ssc_report
```

The report covers 7 sections:

| Section | Spec Ref | What It Checks |
|---------|----------|---------------|
| Corpus | S5.3 | Entry count >= 17,000, format distribution |
| Tokenizer (C-TOK-001) | S5.2 | >= 70% constructs acceptable |
| Label Audit (C-LABEL-001) | S5.3 | >= 90% accuracy, false positives <= 10% |
| Baselines (C-CLF-001) | S5.5 | Majority, keyword, linter MCC scores |
| Generalization (OOD) | S5.6 | >= 50% of 50 OOD scripts caught |
| Dataset Splits | S5.3 | 80/10/10 train/val/test proportions |
| Conversations (S6) | S6 | Generation capacity, quality gates |

Overall readiness: YES when no section has FAIL status.

## WASM Deployment

CodeBERT at 125M int8 (~125MB) fits in a browser. The WASM app at `interactive.paiml.com/shell-safety/` provides:

- **ScriptEditor**: Paste or type a shell script
- **AnalyzeButton**: Run linter + classifier on click
- **SafetyResult**: Classification label + findings
- **FixSuggestion**: Corrected script (when available)
- **ModelStatus**: Model loading state (NotLoaded -> Loading -> Ready)

The linter runs on every keystroke (<10ms). The classifier runs on button click. No network calls after initial model download (weights cached in IndexedDB).

### Probar Testing

All WASM testing uses Probar (`jugar-probar`), not Playwright. Three test layers:

| Layer | Tests | Runs On |
|-------|-------|---------|
| Logic (no browser) | 12 | Every commit |
| Docker cross-browser | 3 | Pre-release |
| Performance benchmarks | 6 | Every commit |

Performance budgets enforced by tests:

| Budget | Target |
|--------|--------|
| Linter latency | < 10ms |
| Classifier inference | < 500ms |
| WASM memory | < 200MB |
| Model load | < 5s |
| Full pipeline | < 600ms |

See `docs/specifications/shell-safety-inference.md` for the complete v11 spec.

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

Pipeline: YAML contract -> scaffold -> proptest/Kani harnesses -> binding to real code -> audit.

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

## See Also

- [Security Rules (SEC001-SEC008)](../linting/security.md)
- [Determinism Rules (DET001-DET003)](../linting/determinism.md)
- [Idempotency Rules (IDEM001-IDEM003)](../linting/idempotency.md)
- [Probar Testing](./probar-testing.md)
- [Corpus Testing](./corpus-testing.md)
