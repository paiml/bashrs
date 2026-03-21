#!/usr/bin/env python3
"""PyTorch canary training — ground truth for entrenar comparison.

Usage: uv run --with 'torch,transformers,peft,bitsandbytes,accelerate,datasets' canary_pytorch.py

Replicates entrenar's exact config: Qwen3-4B NF4 QLoRA, lr=5e-6, cosine decay,
warmup=100, batch_size=4, seq_len=512, grad_clip=1.0, LoRA rank=16 on Q+V.
"""

import json
import math
import os
import time

MODEL_DIR = "/home/noah/src/models/qwen3-4b/"
DATA_PATH = "/home/noah/src/bashrs/training/conversations_v4.jsonl"
OUTPUT_DIR = "/home/noah/src/bashrs/training/checkpoints/canary-pytorch"
MAX_STEPS = 500
BATCH_SIZE = 4
SEQ_LEN = 512
LR = 5e-6
WARMUP_STEPS = 100
GRAD_CLIP = 1.0
LORA_RANK = 16
LORA_ALPHA = 32
SEED = 42

def main():
    import torch
    from transformers import AutoTokenizer, AutoModelForCausalLM, BitsAndBytesConfig
    from peft import LoraConfig, get_peft_model, TaskType
    from torch.optim import AdamW
    from torch.utils.data import DataLoader, Dataset

    os.makedirs(OUTPUT_DIR, exist_ok=True)
    torch.manual_seed(SEED)

    # --- Model ---
    print(f"Loading model from {MODEL_DIR}...")
    bnb_config = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_compute_dtype=torch.bfloat16,
    )
    model = AutoModelForCausalLM.from_pretrained(
        MODEL_DIR,
        quantization_config=bnb_config,
        device_map="auto",
        torch_dtype=torch.bfloat16,
    )

    # --- LoRA ---
    lora_config = LoraConfig(
        r=LORA_RANK,
        lora_alpha=LORA_ALPHA,
        target_modules=["q_proj", "v_proj"],
        lora_dropout=0.0,
        bias="none",
        task_type=TaskType.CAUSAL_LM,
    )
    model = get_peft_model(model, lora_config)
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    print(f"Trainable: {trainable:,} / {total:,} ({100*trainable/total:.2f}%)")

    # --- Tokenizer ---
    tokenizer = AutoTokenizer.from_pretrained(MODEL_DIR, trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    # --- Dataset ---
    class ConversationDataset(Dataset):
        def __init__(self, path, tokenizer, seq_len):
            self.examples = []
            with open(path) as f:
                for line in f:
                    entry = json.loads(line)
                    text = entry.get("text", "")
                    if not text:
                        continue
                    ids = tokenizer.encode(text, max_length=seq_len, truncation=True)
                    if len(ids) < 10:
                        continue
                    # Pad to seq_len
                    if len(ids) < seq_len:
                        ids = ids + [tokenizer.pad_token_id] * (seq_len - len(ids))
                    else:
                        ids = ids[:seq_len]
                    self.examples.append(torch.tensor(ids, dtype=torch.long))
            print(f"Loaded {len(self.examples)} training examples")

        def __len__(self):
            return len(self.examples)

        def __getitem__(self, idx):
            ids = self.examples[idx]
            return {"input_ids": ids, "labels": ids.clone()}

    dataset = ConversationDataset(DATA_PATH, tokenizer, SEQ_LEN)
    dataloader = DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True, drop_last=True)

    # --- Optimizer + Scheduler ---
    optimizer = AdamW(
        [p for p in model.parameters() if p.requires_grad],
        lr=LR,
        weight_decay=0.01,
    )

    def cosine_lr(step):
        if step < WARMUP_STEPS:
            return step / max(WARMUP_STEPS, 1)
        progress = (step - WARMUP_STEPS) / max(MAX_STEPS - WARMUP_STEPS, 1)
        return 0.5 * (1.0 + math.cos(math.pi * progress))

    scheduler = torch.optim.lr_scheduler.LambdaLR(optimizer, cosine_lr)

    # --- Training loop ---
    print(f"Training for {MAX_STEPS} steps...")
    model.train()
    step = 0
    start_time = time.time()
    log_path = os.path.join(OUTPUT_DIR, "canary_log.jsonl")
    log_file = open(log_path, "w")

    data_iter = iter(dataloader)
    while step < MAX_STEPS:
        try:
            batch = next(data_iter)
        except StopIteration:
            data_iter = iter(dataloader)
            batch = next(data_iter)

        input_ids = batch["input_ids"].to(model.device)
        labels = batch["labels"].to(model.device)

        outputs = model(input_ids=input_ids, labels=labels)
        loss = outputs.loss

        loss.backward()

        # Grad clip
        grad_norm = torch.nn.utils.clip_grad_norm_(
            [p for p in model.parameters() if p.requires_grad], GRAD_CLIP
        ).item()

        optimizer.step()
        scheduler.step()
        optimizer.zero_grad()

        step += 1
        elapsed = time.time() - start_time
        tokens = step * BATCH_SIZE * SEQ_LEN
        tok_s = tokens / elapsed if elapsed > 0 else 0
        current_lr = scheduler.get_last_lr()[0]

        entry = {
            "step": step,
            "loss": loss.item(),
            "grad_norm": grad_norm,
            "lr": current_lr,
            "tok_s": tok_s,
            "elapsed_s": elapsed,
        }
        log_file.write(json.dumps(entry) + "\n")
        log_file.flush()

        if step % 10 == 0 or step == 1:
            print(f"  step {step}/{MAX_STEPS}  loss={loss.item():.4f}  "
                  f"gnorm={grad_norm:.2f}  lr={current_lr:.2e}  "
                  f"tok/s={tok_s:.1f}  elapsed={elapsed:.0f}s")

    log_file.close()
    total_time = time.time() - start_time
    print(f"\nDone. {MAX_STEPS} steps in {total_time:.0f}s ({total_time/60:.1f} min)")
    print(f"Final loss: {loss.item():.4f}")
    print(f"Avg throughput: {MAX_STEPS * BATCH_SIZE * SEQ_LEN / total_time:.1f} tok/s")
    print(f"Log: {log_path}")

    # Save adapter
    adapter_path = os.path.join(OUTPUT_DIR, "adapter")
    model.save_pretrained(adapter_path)
    print(f"Adapter saved: {adapter_path}")

if __name__ == "__main__":
    main()
