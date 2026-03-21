# Shell Safety Chat — Qwen2.5-Coder-0.5B LoRA

Shell script safety analyzer fine-tuned from Qwen2.5-Coder-0.5B-Instruct using LoRA on 17,942 synthetic conversations from the bashrs corpus.

## Model Details

- **Base model**: Qwen2.5-Coder-0.5B-Instruct (494M params)
- **Fine-tuning**: Full f32 causal LM training (LoRA rank=16, alpha=32, Q+V projections)
- **Training data**: 17,942 shell safety conversations (ChatML format)
- **Hardware**: NVIDIA RTX 4090, CUDA 12.8
- **Training time**: 87 minutes (3 epochs, 13,458 steps)
- **Final loss**: 4.800 (best: 0.764)
- **Framework**: entrenar (sovereign Rust AI stack)

## Usage

The model classifies shell scripts as safe/unsafe, explains vulnerabilities, and suggests fixes.

### Input format (ChatML)

```
<|im_start|>system
You are a shell script safety analyzer...<|im_end|>
<|im_start|>user
Analyze this script for safety issues:
```bash
rm -rf $DIR
```<|im_end|>
<|im_start|>assistant
```

### Expected output

The model provides:
1. **Classification**: safe or unsafe
2. **Explanation**: What security rules are violated (SEC001-SEC008, DET001-DET006, IDEM001-IDEM006)
3. **Fix suggestion**: Corrected script that passes shellcheck

## Training Configuration

| Parameter | Value |
|-----------|-------|
| Epochs | 3 |
| Batch size | 4 |
| Sequence length | 512 |
| Learning rate | 2e-4 (AdamW) |
| Gradient accumulation | 4 |
| Peak throughput | 5,172 tok/s (MFU 18.6%) |

## Limitations

- Trained on synthetic conversations only (not real-world shell scripts)
- 0.5B model has limited reasoning capacity compared to larger models
- Sequence length capped at 512 tokens
- Base model loaded in f32 (NF4 quantization not yet supported in entrenar bridge)

## License

Apache-2.0

## Citation

```bibtex
@software{bashrs_shell_safety_chat,
  title = {Shell Safety Chat: Qwen2.5-Coder-0.5B LoRA for Shell Script Analysis},
  author = {paiml engineering},
  year = {2026},
  url = {https://huggingface.co/paiml/shell-safety-chat}
}
```
