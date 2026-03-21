#!/usr/bin/env python3
"""FALSIFY-PARITY-V2-001..004: Isolate cuBLAS zero-output bug.

Run on GB10 with: uv run --extra-index-url https://download.pytorch.org/whl/cu130 \
  --with 'torch,transformers,bitsandbytes,accelerate,scipy' test_cublas_parity.py
"""

import json, torch, time

MODEL = "/home/noah/src/models/qwen3-4b/"

def test_001_single_gemm_parity():
    """FALSIFY-PARITY-V2-001: Single q_proj GEMM through both paths."""
    from transformers import AutoModelForCausalLM, BitsAndBytesConfig
    import bitsandbytes.functional as F

    print("=== TEST 001: Single projection GEMM parity ===")

    # Load model with NF4
    bnb = BitsAndBytesConfig(load_in_4bit=True, bnb_4bit_quant_type="nf4",
                              bnb_4bit_compute_dtype=torch.bfloat16)
    model = AutoModelForCausalLM.from_pretrained(MODEL, quantization_config=bnb,
                                                  device_map="auto", torch_dtype=torch.bfloat16)

    # Get q_proj from first layer
    layer0 = model.model.layers[0]
    q_proj = layer0.self_attn.q_proj

    # The weight is stored as bnb Linear4bit
    print(f"  q_proj type: {type(q_proj)}")
    print(f"  q_proj weight shape: {q_proj.weight.shape}")
    print(f"  q_proj weight dtype: {q_proj.weight.dtype}")

    # Dequantize the weight
    w_deq = F.dequantize_4bit(q_proj.weight.data, q_proj.weight.quant_state)
    print(f"  Dequantized shape: {w_deq.shape}")
    print(f"  Dequantized dtype: {w_deq.dtype}")
    print(f"  Dequantized first 5: {w_deq.flatten()[:5].tolist()}")
    print(f"  Dequantized nonzero: {(w_deq != 0).sum().item()} / {w_deq.numel()}")

    # Create test activation
    A = torch.randn(4, w_deq.shape[1], dtype=torch.bfloat16, device=w_deq.device)

    # Method 1: bitsandbytes way — dequant().to(dtype).t() then linear
    W_t = w_deq.to(A.dtype).t()  # [N,K] → [K,N] via stride change
    print(f"  W_t shape: {W_t.shape}, strides: {W_t.stride()}")
    C_bnb = torch.nn.functional.linear(A, w_deq.to(A.dtype))  # linear does A @ W.T internally
    print(f"  C_bnb shape: {C_bnb.shape}, first 5: {C_bnb.flatten()[:5].tolist()}")

    # Method 2: explicit matmul with transposed weight
    C_mm = A @ W_t
    print(f"  C_mm shape: {C_mm.shape}, first 5: {C_mm.flatten()[:5].tolist()}")

    # Method 3: contiguous transposed copy (what our cuBLAS path does)
    W_t_contig = W_t.contiguous()
    print(f"  W_t_contig strides: {W_t_contig.stride()}, is_contiguous: {W_t_contig.is_contiguous()}")
    C_contig = A @ W_t_contig
    print(f"  C_contig shape: {C_contig.shape}, first 5: {C_contig.flatten()[:5].tolist()}")

    # Compare
    diff_mm = (C_bnb - C_mm).abs().max().item()
    diff_contig = (C_bnb - C_contig).abs().max().item()
    print(f"  Max diff (bnb vs matmul): {diff_mm}")
    print(f"  Max diff (bnb vs contiguous transpose): {diff_contig}")

    # KEY QUESTION: does .t() (stride change) produce different results from
    # .t().contiguous() (physical transpose)?
    diff_stride_vs_contig = (C_mm - C_contig).abs().max().item()
    print(f"  Max diff (stride .t() vs contiguous .t()): {diff_stride_vs_contig}")

    assert diff_mm < 0.01, f"FAIL: bnb vs matmul diff {diff_mm}"
    assert diff_contig < 0.01, f"FAIL: bnb vs contiguous diff {diff_contig}"
    print("  PASS\n")

    return w_deq, A, C_bnb

def test_002_stride_vs_contiguous():
    """Does PyTorch .t() (stride) produce different cuBLAS behavior than .t().contiguous()?"""
    print("=== TEST 002: Stride transpose vs contiguous transpose ===")

    # Simple test
    W = torch.randn(256, 128, dtype=torch.float32, device="cuda")
    A = torch.randn(4, 128, dtype=torch.float32, device="cuda")

    # Stride transpose
    W_t_stride = W.t()  # [128, 256] via stride, NOT contiguous
    C_stride = A @ W_t_stride

    # Contiguous transpose
    W_t_contig = W.t().contiguous()  # [128, 256] via copy, IS contiguous
    C_contig = A @ W_t_contig

    diff = (C_stride - C_contig).abs().max().item()
    print(f"  Stride transpose strides: {W_t_stride.stride()}")
    print(f"  Contiguous transpose strides: {W_t_contig.stride()}")
    print(f"  Max diff: {diff}")
    print(f"  {'PASS' if diff < 1e-5 else 'FAIL'}\n")

def test_003_what_cublas_sees():
    """Show exactly what data layout cuBLAS receives for stride vs contiguous."""
    print("=== TEST 003: Memory layout comparison ===")

    W = torch.tensor([[1.0, 2.0, 3.0],
                       [4.0, 5.0, 6.0]], dtype=torch.float32, device="cuda")

    W_t_stride = W.t()
    W_t_contig = W.t().contiguous()

    # The raw memory bytes
    print(f"  W[2,3] raw: {W.flatten().tolist()}")
    print(f"  W.t() stride[3,2]: {W_t_stride.flatten().tolist()} (stride={W_t_stride.stride()})")
    print(f"  W.t().contiguous()[3,2]: {W_t_contig.flatten().tolist()} (stride={W_t_contig.stride()})")

    # W raw memory: [1,2,3,4,5,6] (row-major [2,3])
    # W.t() raw memory: SAME [1,2,3,4,5,6] but stride=(1,3) → col-major view
    # W.t().contiguous() raw memory: [1,4,2,5,3,6] (row-major [3,2])

    # When cuBLAS reads W.t() with stride (1,3), it correctly reads column-major
    # When cuBLAS reads W.t().contiguous() with stride (2,1), it reads row-major
    # BOTH should produce the same matrix — but cuBLAS sees different memory layout

    A = torch.tensor([[1.0, 0.0, 0.0],
                       [0.0, 1.0, 0.0]], dtype=torch.float32, device="cuda")

    # A[2,3] @ W.t()[3,2] should give W[2,2]... wait that's wrong dims
    # A[2,3] @ W.t()[3,2] = C[2,2]
    C_stride = A @ W_t_stride
    C_contig = A @ W_t_contig

    print(f"  A @ W.t() (stride): {C_stride.tolist()}")
    print(f"  A @ W.t() (contig): {C_contig.tolist()}")
    print(f"  Expected: A=identity selects rows of W.t() = columns of W")
    print(f"  {'PASS' if torch.allclose(C_stride, C_contig) else 'FAIL'}\n")

if __name__ == "__main__":
    print("FALSIFY-PARITY-V2: cuBLAS zero-output isolation tests\n")
    test_002_stride_vs_contiguous()
    test_003_what_cublas_sees()
    test_001_single_gemm_parity()
    print("All tests complete.")
