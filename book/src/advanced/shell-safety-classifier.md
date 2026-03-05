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
