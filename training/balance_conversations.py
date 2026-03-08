#!/usr/bin/env python3
"""Create class-balanced training data for run 6.

Addresses the 97.7% safe imbalance that causes the model to default to "safe".
Strategy: downsample safe to match unsafe count, augment with adversarial data.
"""

import json
import random

INPUT_V2 = "training/conversations_v2.jsonl"
ADVERSARIAL = "adversarial.jsonl"
OUTPUT = "training/conversations_v3.jsonl"

SYSTEM_MSG = (
    "You are a shell script safety analyzer. You classify scripts as safe or unsafe "
    "and explain your reasoning.\n\n"
    "IMPORTANT: Always begin your response with exactly one of these on the first line:\n"
    "  Classification: safe\n"
    "  Classification: unsafe\n\n"
    "Then provide your analysis. For unsafe scripts, cite specific rule IDs "
    "(SEC001-SEC008, DET001-DET006, IDEM001-IDEM004, SC-prefixed ShellCheck rules) "
    "and suggest fixes in ```bash code blocks."
)

UNSAFE_INSTRUCTIONS = [
    "Analyze this shell script.",
    "Is this script safe?",
    "Review this bash script for me.",
    "Check this script for issues.",
    "What security problems does this script have?",
    "Can I safely run this script?",
    "Find the source of randomness in this script.",
    "Why is this script non-deterministic?",
    "Help me make this script deterministic.",
    "Review this script for safety issues.",
]


def build_chatml(system, instruction, response):
    """Build ChatML formatted text."""
    return (
        f"<|im_start|>system\n{system}<|im_end|>\n"
        f"<|im_start|>user\n{instruction}<|im_end|>\n"
        f"<|im_start|>assistant\n{response}<|im_end|>"
    )


def adversarial_to_conversation(entry, rng):
    """Convert adversarial entry to conversation format."""
    script = entry["input"]
    instruction = rng.choice(UNSAFE_INSTRUCTIONS)
    instruction = f"{instruction}\n\n```bash\n{script}\n```"

    response = (
        "Classification: unsafe\n\n"
        "This script contains potentially unsafe patterns that could lead to "
        "security vulnerabilities or non-deterministic behavior. "
        "Review the script carefully before running it in production."
    )

    return {
        "instruction": instruction,
        "response": response,
        "system": SYSTEM_MSG,
        "text": build_chatml(SYSTEM_MSG, instruction, response),
    }


def main():
    rng = random.Random(42)

    safe_entries = []
    unsafe_entries = []

    with open(INPUT_V2) as f:
        for line in f:
            entry = json.loads(line)
            if entry["response"].startswith("Classification: unsafe"):
                unsafe_entries.append(entry)
            else:
                safe_entries.append(entry)

    # Convert adversarial entries
    adversarial_convos = []
    with open(ADVERSARIAL) as f:
        for line in f:
            entry = json.loads(line)
            adversarial_convos.append(adversarial_to_conversation(entry, rng))

    total_unsafe = len(unsafe_entries) + len(adversarial_convos)
    print(f"Original safe: {len(safe_entries)}, unsafe: {len(unsafe_entries)}")
    print(f"Adversarial unsafe: {len(adversarial_convos)}")
    print(f"Total unsafe: {total_unsafe}")

    # Downsample safe to match unsafe count (with 2x multiplier for slight safe bias)
    target_safe = min(len(safe_entries), total_unsafe * 2)
    rng.shuffle(safe_entries)
    safe_sample = safe_entries[:target_safe]

    # Combine and shuffle
    all_entries = safe_sample + unsafe_entries + adversarial_convos
    rng.shuffle(all_entries)

    with open(OUTPUT, "w") as f:
        for entry in all_entries:
            f.write(json.dumps(entry) + "\n")

    # Count final distribution
    n_safe = sum(1 for e in all_entries if e["response"].startswith("Classification: safe"))
    n_unsafe = sum(1 for e in all_entries if e["response"].startswith("Classification: unsafe"))
    print(f"\nBalanced dataset: {len(all_entries)} entries")
    print(f"  Safe: {n_safe} ({100*n_safe/len(all_entries):.1f}%)")
    print(f"  Unsafe: {n_unsafe} ({100*n_unsafe/len(all_entries):.1f}%)")
    print(f"Output: {OUTPUT}")


if __name__ == "__main__":
    main()
