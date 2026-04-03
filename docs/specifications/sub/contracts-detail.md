# Sub-spec: Contract Details

**Parent:** [shell-safety-inference.md](../shell-safety-inference.md) Section 3

---

## Contract Architecture

Every bashrs contract follows the falsification pattern (Popper, 1959):
tests are designed to **refute**, not confirm. A contract PASSES when
all FALSIFY tests fail to find a violation.

### Contract Categories

| Category | Count | Purpose |
|----------|-------|---------|
| Training pipeline | 6 | WGPU backward, QLoRA, parity, stability |
| Data pipeline | 2 | SSB splits, benchmark quality |
| Inference | 3 | Encoder, classifier, embeddings |
| WASM | 3 | Linter bindings, probar tests, CodeBERT (killed) |
| GPU optimization | 4 | NF4 codebook, cuBLAS, throughput, kernels |

### Verification Levels

Following provable-contracts spec (pv-spec.md Section 2):

```
Level   Method              Tool            bashrs Coverage
-----   ------              ----            ---------------
  L3    Property-based      probar/proptest 19 WASM + 100+ corpus property tests
  L2    Falsification       #[test]         68 FALSIFY tests across 18 contracts
  L1    Type system         rustc           build.rs AllImplemented (16/16 gaps = 0)
  L0.5  Schema/audit        pv lint         18/18 contracts pass schema validation
```

L4 (Kani BMC) and L5 (Lean 4 proof) are defined in the contract schema
but not yet exercised for bashrs contracts. Target: L4 for linter rules
(bounded input → bounded verification is exhaustive).

---

## Gap Analysis

### GAP-1: Transpiler Stdlib Contracts

**Current:** GH-148 added `capture()`, `exit()`, `sleep()` to the
transpiler but no contract verifies their behavior.

**Fix:** Create `transpiler-stdlib-v1.yaml` with:

```yaml
contract:
  name: transpiler-stdlib-v1
  version: "1.0.0"
  description: >
    Transpiler stdlib functions produce correct POSIX shell output.
falsification_tests:
  - id: F-STDLIB-001
    description: "capture('whoami') produces $(whoami)"
    assertion: "output contains '$(whoami)'"
  - id: F-STDLIB-002
    description: "exit(0) produces 'exit 0'"
    assertion: "output contains 'exit 0'"
  - id: F-STDLIB-003
    description: "sleep(5) produces 'sleep 5'"
    assertion: "output contains 'sleep 5'"
  - id: F-STDLIB-004
    description: "capture('date +%Y') splits program from args"
    assertion: "output contains '$(date' and not '$('date'"
```

### GAP-2: Benchmark Expansion Contracts

**Current:** SSB expanded from 27,842 to 49,842 entries via
`generate-expansion` but no contract gates quality.

**Fix:** Create `benchmark-expansion-v1.yaml` with:

```yaml
contract:
  name: benchmark-expansion-v1
  version: "1.0.0"
  description: >
    Generated expansion entries meet quality gates.
falsification_tests:
  - id: F-EXPAND-001
    description: "Generated entries have both safe and unsafe labels"
    assertion: "unsafe_count > 0 AND safe_count > 0"
  - id: F-EXPAND-002
    description: "Deterministic: same seed produces identical output"
    assertion: "hash(gen(seed=42)) == hash(gen(seed=42))"
  - id: F-EXPAND-003
    description: "Labels agree with linter"
    assertion: "for all entries: label == lint(input)"
  - id: F-EXPAND-004
    description: "No empty inputs"
    assertion: "for all entries: len(input) > 0"
```

### GAP-3: HuggingFace Publication Contracts

**Current:** `publish-benchmark` generates HF-ready directory but no
contract verifies the output structure.

**Fix:** Create `hf-publish-v1.yaml` with:

```yaml
contract:
  name: hf-publish-v1
  version: "1.0.0"
  description: >
    HuggingFace publication directory is complete and valid.
falsification_tests:
  - id: F-HF-001
    description: "README.md has YAML front matter"
    assertion: "README.md starts with '---'"
  - id: F-HF-002
    description: "All three split files exist"
    assertion: "train.jsonl AND validation.jsonl AND test.jsonl exist"
  - id: F-HF-003
    description: "dataset_infos.json is valid JSON"
    assertion: "serde_json::from_str(dataset_infos.json) succeeds"
  - id: F-HF-004
    description: "Split entry counts match summary"
    assertion: "wc -l train.jsonl == summary.train_count"
```

### GAP-4: Linter Coverage Contracts

**Current:** 14 linter rules (SEC001-008, DET001-003, IDEM001-003)
across 3 formats but no contract ensures each rule fires on at least
one corpus entry per format.

**Fix:** Create `linter-coverage-v1.yaml` with 42 obligations
(14 rules × 3 formats). Each obligation requires at least one corpus
entry that triggers the rule in the specified format.

---

## Existing Contract Details

### shellsafetybench-v1.yaml (11 FALSIFY)

Pipeline correctness: merge, split, label, export, decontamination.

| Test | What it falsifies |
|------|-------------------|
| F-SSB-001 | Merged data has both safe and unsafe |
| F-SSB-002 | Splits sum to total |
| F-SSB-003 | No exact duplicates across train/test |
| F-SSB-004 | Unsafe ratio > 5% in each split |
| F-SSB-005 | All entries have valid JSON |
| F-SSB-006 | Labels are binary (0 or 1) |
| F-SSB-007 | No preamble contamination |
| F-SSB-008 | No trivial inputs (< 3 chars) |
| F-SSB-009 | Format distribution covers all 3 languages |
| F-SSB-010 | CWE mapping exists for all unsafe entries |
| F-SSB-011 | Export produces valid JSONL |

### wgpu-production-training-v1.yaml (7 FALSIFY)

Ship criteria for sovereign-stack training.

| Test | What it falsifies |
|------|-------------------|
| F-PROD-001 | Ship: MCC > 0.50 on test entries |
| F-LORA-BWD-001 | Real LoRA backward (not frozen) |
| F-LORA-BWD-002 | Gradient norms decrease over training |
| F-LORA-BWD-003 | LoRA weights change after optimizer step |
| F-SEQLEN | seq_len=128 tokens per sample |
| F-GRADACC | Gradient accumulation = 4 steps |
| F-EPOCH | Multi-epoch training loop |
