#!/usr/bin/env python3
"""Restructure conversations.jsonl for Option C: add classification prefix.

Every response gets prefixed with "Classification: safe" or "Classification: unsafe"
on the first line. The system message is also updated to instruct the model to always
begin with this prefix.

This teaches the model the output FORMAT, not just the domain CONTENT.
"""

import json
import sys

INPUT = "training/conversations.jsonl"
OUTPUT = "training/conversations_v2.jsonl"

# Updated system message that instructs format compliance
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

# Rule IDs that indicate unsafe
UNSAFE_RULES = [
    "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
    "DET001", "DET002", "DET003", "DET004", "DET005", "DET006",
    "IDEM001", "IDEM002", "IDEM003", "IDEM004",
    "SC1020", "SC1028", "SC1035", "SC1037", "SC1041", "SC1044", "SC1065",
    "SC1078", "SC1140", "SC2105", "SC2140", "SC2148", "SC2219",
]


SAFE_INDICATORS = [
    "I don't see any security issues",
    "This script follows good practices",
    "This script is safe to run",
    "No security concerns found",
    "This script looks good",
    "This is a clean, safe script",
    "No issues detected",
    "This script appears production-ready",
    "This script looks safe",
    "This script appears to be well-written",
    "doesn't contain known unsafe patterns",
]


def _has_rule_citation(resp):
    return any(rule in resp for rule in UNSAFE_RULES)


def _has_safe_indicator(resp):
    return any(resp.startswith(s) or s in resp for s in SAFE_INDICATORS)


def classify_entry(entry):
    """Classify entry as safe/unsafe using rule citations and safe indicators."""
    resp = entry["response"]
    if _has_rule_citation(resp):
        return "unsafe"
    if _has_safe_indicator(resp):
        return "safe"
    return "unsafe"


def build_chatml(system, instruction, response):
    """Build ChatML formatted text."""
    return (
        f"<|im_start|>system\n{system}<|im_end|>\n"
        f"<|im_start|>user\n{instruction}<|im_end|>\n"
        f"<|im_start|>assistant\n{response}<|im_end|>"
    )


def main():
    stats = {"safe": 0, "unsafe": 0, "total": 0}

    with open(INPUT) as fin, open(OUTPUT, "w") as fout:
        for line in fin:
            entry = json.loads(line)
            label = classify_entry(entry)
            stats[label] += 1
            stats["total"] += 1

            # Prefix response with classification
            new_response = f"Classification: {label}\n\n{entry['response']}"

            # Build new entry with updated system message and prefixed response
            new_entry = {
                "instruction": entry["instruction"],
                "response": new_response,
                "system": SYSTEM_MSG,
                "text": build_chatml(SYSTEM_MSG, entry["instruction"], new_response),
            }

            fout.write(json.dumps(new_entry) + "\n")

    print(f"Restructured {stats['total']} entries: {stats['safe']} safe, {stats['unsafe']} unsafe")
    print(f"Output: {OUTPUT}")

    # Verify a sample
    with open(OUTPUT) as f:
        first = json.loads(f.readline())
        print(f"\nSample response prefix: {first['response'][:80]}...")
        print(f"Sample text contains 'Classification:': {'Classification:' in first['text']}")


if __name__ == "__main__":
    main()
