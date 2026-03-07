#!/usr/bin/env python3
"""Evaluate SSC chat model (Qwen2.5-Coder-0.5B LoRA fine-tuned).

Runs inference on test samples and evaluates:
  C-CHAT-TRAIN-002: Classification accuracy > 85%
  C-CHAT-TRAIN-003: Fix suggestions pass shellcheck > 85%
  C-CHAT-TRAIN-004: Rule citations match linter output > 90%
"""

import json
import os
import subprocess
import sys
import random

# Paths
CHECKPOINT = os.path.join(os.path.dirname(__file__), "checkpoints/ssc-chat-v1")
CONVERSATIONS = os.path.join(os.path.dirname(__file__), "conversations.jsonl")

def load_model():
    """Load fine-tuned model and tokenizer."""
    from transformers import AutoModelForCausalLM, AutoTokenizer
    import torch

    device = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"Loading model on {device}...")

    tokenizer = AutoTokenizer.from_pretrained(CHECKPOINT, trust_remote_code=True)
    model = AutoModelForCausalLM.from_pretrained(
        CHECKPOINT,
        torch_dtype=torch.float16 if device == "cuda" else torch.float32,
        device_map=device,
        trust_remote_code=True,
    )
    model.eval()
    print(f"Model loaded: {sum(p.numel() for p in model.parameters())/1e6:.1f}M params")
    return model, tokenizer, device


def build_prompt(system_msg, instruction):
    """Build ChatML prompt."""
    return (
        f"<|im_start|>system\n{system_msg}<|im_end|>\n"
        f"<|im_start|>user\n{instruction}<|im_end|>\n"
        f"<|im_start|>assistant\n"
    )


def generate(model, tokenizer, device, prompt, max_new_tokens=256):
    """Generate response from model."""
    import torch

    inputs = tokenizer(prompt, return_tensors="pt", truncation=True, max_length=256)
    inputs = {k: v.to(device) for k, v in inputs.items()}

    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=max_new_tokens,
            temperature=0.1,
            top_p=0.9,
            do_sample=True,
            pad_token_id=tokenizer.eos_token_id,
        )

    # Decode only new tokens
    new_tokens = outputs[0][inputs["input_ids"].shape[1]:]
    response = tokenizer.decode(new_tokens, skip_special_tokens=True)
    return response.strip()


def load_test_set(n_safe=25, n_unsafe=25, seed=42):
    """Load balanced test set from conversations."""
    random.seed(seed)
    safe, unsafe = [], []

    with open(CONVERSATIONS) as f:
        for line in f:
            entry = json.loads(line)
            resp = entry["response"]
            has_rules = any(r in resp for r in [
                "SEC001", "SEC002", "SEC003", "SEC004", "SEC005",
                "SEC006", "SEC007", "SEC008", "DET001", "DET002",
                "DET003", "DET004", "DET005", "DET006",
                "IDEM001", "IDEM002", "IDEM003", "IDEM004",
            ])
            if has_rules:
                unsafe.append(entry)
            elif "safe" in resp.lower() and "unsafe" not in resp.lower():
                safe.append(entry)

    random.shuffle(safe)
    random.shuffle(unsafe)
    test_safe = safe[:n_safe]
    test_unsafe = unsafe[:n_unsafe]

    print(f"Test set: {len(test_safe)} safe + {len(test_unsafe)} unsafe = {len(test_safe)+len(test_unsafe)}")
    return test_safe, test_unsafe


def classify_response(response):
    """Parse model response to extract classification."""
    resp_lower = response.lower()
    if "unsafe" in resp_lower:
        return "unsafe"
    elif "safe" in resp_lower:
        return "safe"
    return "unknown"


def extract_rules(response):
    """Extract rule citations from response."""
    import re
    return set(re.findall(r'(SEC\d{3}|DET\d{3}|IDEM\d{3}|SC\d{4})', response))


def check_shellcheck(script):
    """Run shellcheck on a script, return True if passes."""
    try:
        result = subprocess.run(
            ["shellcheck", "-s", "sh", "-"],
            input=script, capture_output=True, text=True, timeout=5,
        )
        return result.returncode == 0
    except Exception:
        return False


def extract_fix(response):
    """Extract code block from response (fix suggestion)."""
    import re
    blocks = re.findall(r'```(?:bash|sh)?\n(.*?)```', response, re.DOTALL)
    return blocks[-1].strip() if blocks else None


def evaluate():
    model, tokenizer, device = load_model()
    test_safe, test_unsafe = load_test_set()

    results = {
        "correct": 0, "total": 0,
        "shellcheck_pass": 0, "shellcheck_total": 0,
        "citation_match": 0, "citation_total": 0,
        "responses": [],
    }

    all_entries = [(e, "safe") for e in test_safe] + [(e, "unsafe") for e in test_unsafe]

    for i, (entry, expected_label) in enumerate(all_entries):
        prompt = build_prompt(entry["system"], entry["instruction"])
        response = generate(model, tokenizer, device, prompt)

        predicted = classify_response(response)
        correct = predicted == expected_label
        results["correct"] += int(correct)
        results["total"] += 1

        # C-CHAT-TRAIN-003: shellcheck on fix suggestions
        fix = extract_fix(response)
        if fix:
            sc_pass = check_shellcheck(fix)
            results["shellcheck_pass"] += int(sc_pass)
            results["shellcheck_total"] += 1

        # C-CHAT-TRAIN-004: rule citations
        if expected_label == "unsafe":
            expected_rules = extract_rules(entry["response"])
            predicted_rules = extract_rules(response)
            if expected_rules:
                overlap = len(expected_rules & predicted_rules) / len(expected_rules)
                results["citation_match"] += overlap
                results["citation_total"] += 1

        status = "OK" if correct else "MISS"
        print(f"[{i+1:3d}/{len(all_entries)}] {status} expected={expected_label} predicted={predicted} | {entry['instruction'][:60]}...")

        results["responses"].append({
            "expected": expected_label,
            "predicted": predicted,
            "correct": correct,
            "response_snippet": response[:200],
        })

    # Summary
    accuracy = results["correct"] / results["total"] if results["total"] else 0
    sc_rate = results["shellcheck_pass"] / results["shellcheck_total"] if results["shellcheck_total"] else 0
    cite_rate = results["citation_match"] / results["citation_total"] if results["citation_total"] else 0

    print("\n" + "=" * 60)
    print("EVALUATION RESULTS")
    print("=" * 60)
    print(f"C-CHAT-TRAIN-002: Classification accuracy = {accuracy:.1%} ({results['correct']}/{results['total']}) {'PASS' if accuracy > 0.85 else 'FAIL'}")
    print(f"C-CHAT-TRAIN-003: Shellcheck pass rate    = {sc_rate:.1%} ({results['shellcheck_pass']}/{results['shellcheck_total']}) {'PASS' if sc_rate > 0.85 else 'FAIL' if results['shellcheck_total'] else 'N/A'}")
    print(f"C-CHAT-TRAIN-004: Citation accuracy        = {cite_rate:.1%} {'PASS' if cite_rate > 0.90 else 'FAIL' if results['citation_total'] else 'N/A'}")

    # Save results
    out_path = os.path.join(CHECKPOINT, "eval_results.json")
    with open(out_path, "w") as f:
        json.dump({
            "accuracy": accuracy,
            "shellcheck_rate": sc_rate,
            "citation_rate": cite_rate,
            "n_correct": results["correct"],
            "n_total": results["total"],
            "n_shellcheck_pass": results["shellcheck_pass"],
            "n_shellcheck_total": results["shellcheck_total"],
            "n_citation_match": results["citation_match"],
            "n_citation_total": results["citation_total"],
            "responses": results["responses"],
        }, f, indent=2)
    print(f"\nResults saved to {out_path}")


if __name__ == "__main__":
    evaluate()
