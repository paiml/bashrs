#!/usr/bin/env python3
"""Evaluate PyTorch canary adapter on test split (CPU inference).

Usage: uv run --with 'torch,transformers,peft' eval_canary.py

Answers the ship/kill question: does the trained model classify shell scripts?
"""

import json, time, sys

MODEL_DIR = "/home/noah/src/models/qwen3-4b/"
ADAPTER_DIR = "/home/noah/src/bashrs/training/checkpoints/canary-pytorch/adapter/"
TEST_DATA = "/home/noah/src/bashrs/training/shellsafetybench/splits/test.jsonl"
MAX_ENTRIES = 100  # Subset for CPU speed (full test split is 2935 entries)
MAX_TOKENS = 64

def main():
    import torch
    from transformers import AutoTokenizer, AutoModelForCausalLM
    from peft import PeftModel

    print(f"Loading base model from {MODEL_DIR}...")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_DIR, trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    model = AutoModelForCausalLM.from_pretrained(
        MODEL_DIR, torch_dtype=torch.float32, device_map="cpu"
    )

    print(f"Loading LoRA adapter from {ADAPTER_DIR}...")
    model = PeftModel.from_pretrained(model, ADAPTER_DIR)
    model.eval()
    print(f"Model loaded. Parameters: {sum(p.numel() for p in model.parameters()):,}")

    # Load test data
    print(f"Loading test data from {TEST_DATA}...")
    entries = []
    with open(TEST_DATA) as f:
        for i, line in enumerate(f):
            if i >= MAX_ENTRIES:
                break
            entries.append(json.loads(line))
    print(f"Loaded {len(entries)} test entries")

    # Run inference
    results = {"total": 0, "correct": 0, "format_ok": 0, "safe": 0, "unsafe": 0,
               "true_safe": 0, "true_unsafe": 0, "predictions": []}
    start = time.time()

    for i, entry in enumerate(entries):
        text = entry.get("input", entry.get("text", ""))
        label = entry.get("label", 0)  # 0=safe, 1=unsafe
        ground_truth = "unsafe" if label == 1 else "safe"

        if label == 1:
            results["true_unsafe"] += 1
        else:
            results["true_safe"] += 1

        # Tokenize and generate
        inputs = tokenizer.encode(text, max_length=448, truncation=True, return_tensors="pt")
        with torch.no_grad():
            outputs = model.generate(
                inputs, max_new_tokens=MAX_TOKENS, temperature=0.1,
                do_sample=False, pad_token_id=tokenizer.pad_token_id,
            )

        response = tokenizer.decode(outputs[0][inputs.shape[1]:], skip_special_tokens=True)

        # Parse classification
        lower = response.lower()
        if "classification: unsafe" in lower or "classification:unsafe" in lower:
            predicted = "unsafe"
            results["format_ok"] += 1
        elif "classification: safe" in lower or "classification:safe" in lower:
            predicted = "safe"
            results["format_ok"] += 1
        elif "unsafe" in lower:
            predicted = "unsafe"
        else:
            predicted = "safe"

        if predicted == "unsafe":
            results["unsafe"] += 1
        else:
            results["safe"] += 1

        correct = predicted == ground_truth
        if correct:
            results["correct"] += 1
        results["total"] += 1

        results["predictions"].append({
            "id": f"test-{i}",
            "label": label,
            "ground_truth": ground_truth,
            "predicted": predicted,
            "correct": correct,
            "response_preview": response[:100],
        })

        if (i + 1) % 10 == 0 or i == 0:
            elapsed = time.time() - start
            rate = (i + 1) / elapsed
            eta = (len(entries) - i - 1) / rate if rate > 0 else 0
            acc = results["correct"] / results["total"] * 100
            print(f"  [{i+1}/{len(entries)}] acc={acc:.1f}% pred={predicted} "
                  f"truth={ground_truth} {'✓' if correct else '✗'} "
                  f"({rate:.2f} it/s, ETA {eta:.0f}s)")

    elapsed = time.time() - start
    acc = results["correct"] / results["total"] * 100 if results["total"] > 0 else 0
    fmt = results["format_ok"] / results["total"] * 100 if results["total"] > 0 else 0

    print(f"\n{'='*60}")
    print(f"EVALUATION RESULTS ({results['total']} entries, {elapsed:.0f}s)")
    print(f"{'='*60}")
    print(f"Accuracy:         {acc:.1f}% ({results['correct']}/{results['total']})")
    print(f"Format compliance: {fmt:.1f}% ({results['format_ok']}/{results['total']})")
    print(f"Predicted safe:   {results['safe']}")
    print(f"Predicted unsafe: {results['unsafe']}")
    print(f"True safe:        {results['true_safe']}")
    print(f"True unsafe:      {results['true_unsafe']}")
    print(f"\nSHIP/KILL GATE: {'PASS (acc > 50%)' if acc > 50 else 'FAIL (acc <= 50%) — KILL-QLORA-002'}")

    # Save results
    out_path = "/tmp/canary_eval_results.json"
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to {out_path}")

if __name__ == "__main__":
    main()
