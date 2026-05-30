# AKG Kernel Agent — Glossary, Methods & Formulas

Reference notes distilled from:

> Du, Yuan, Zhang, Yi, Hu, et al. *AKG Kernel Agent: A Multi-Agent Framework for
> Cross-Platform Kernel Synthesis.* Huawei Technologies & Hunan University.
> arXiv:2512.23424v1, Dec 2025.

**One-line summary.** A multi-agent system (Designer · Coder · Verifier ·
Conductor) that automates GPU/NPU **compute-kernel** generation, migration, and
tuning across DSLs (Triton, TileLang, CUDA-C, CPP). Document-driven + retrieval-
augmented so new DSLs/hardware need no agent changes. On **KernelBench** it hits
up to 100% correctness (Level 1) and a **1.46× geometric-mean speedup** over
PyTorch Eager.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **Compute kernel** | A low-level routine (matmul, attention, …) whose performance dominates AI model throughput. |
| **AKG Kernel Agent** | AI-driven Kernel Generator — a multi-agent system automating kernel generation, migration, and tuning. |
| **DSL** | Domain-specific language for kernels: Triton, TileLang, CUDA-C, CPP. AKG targets several. |
| **Backend** | Target hardware: GPU, NPU, CPU. |
| **Designer** | Agent that makes high-level optimization *strategy* decisions (tiling, parallelization) — decoupled from coding. |
| **Coder** | Agent that synthesizes the actual kernel code in the target DSL, consulting API docs + retrieved examples. |
| **Verifier** | Agent that compiles, checks **correctness** against a reference, and **profiles** performance on target hardware. |
| **Conductor** | Central orchestrator: analyzes execution state, routes work among agents, handles errors — the iterative-refinement loop. |
| **Document-driven generalization** | New operators/DSLs/hardware are supported by ingesting structured docs, *without* changing agents. |
| **Hierarchical retrieval** | RAG over a knowledge base of API docs + optimized-kernel examples to ground generation. |
| **Search-based tuning** | Parallel, multi-iteration exploration; each round keeps the best kernels as baselines and plans the next. |
| **Closed-loop refinement** | Verifier feedback (errors, profile) is fed back to the agents instead of a single-shot generation. |
| **KernelBench** | Standardized benchmark of kernel-generation tasks (Levels by difficulty); scores correctness + speedup. |
| **PyTorch Eager baseline** | The reference implementation kernels are compared against for speedup. |
| **Correctness** | Functional equivalence of the generated kernel to the reference (within tolerance). |
| **Speedup** | Ratio of baseline latency to generated-kernel latency (>1 = faster). |
| **fast_p** | KernelBench metric: fraction of tasks that are both correct *and* faster than baseline by at least factor `p`. |
| **Reward hacking** | Exploiting benchmark gaps to score well without genuine speedup; the authors built a harder benchmark to resist it. |

---

## 2. Methods

### 2.1 Four-agent collaborative architecture
A coordinated loop with separation of concerns:
- **Conductor** orchestrates: reads execution state, routes between agents, diagnoses errors.
- **Designer** decides the optimization strategy (decoupled from implementation).
- **Coder** writes the kernel in the target DSL using API docs + retrieved examples.
- **Verifier** compiles, verifies correctness vs. a reference, and profiles on hardware.

Design principles: **(1)** decouple strategy (Designer) from implementation
(Coder) to cut per-call cognitive load and improve interpretability; **(2)**
extensibility through documentation, not hard-coded DSL/hardware support;
**(3)** closed-loop refinement driven by Verifier feedback.

### 2.2 End-to-end workflow
Given an operator spec (reference implementation): Designer plans → Coder
synthesizes DSL code (RAG-grounded) → Verifier compiles, checks correctness,
profiles → Conductor inspects errors/metrics and re-routes for another
iteration until correct and fast.

### 2.3 Document-driven + hierarchical retrieval
The system ingests structured DSL/hardware docs and retrieves relevant API
references and optimized-kernel exemplars (hierarchical RAG), so adding a new
DSL or backend is a knowledge-base change, not a code change.

### 2.4 Search-based performance tuning
Parallel search across multiple iterations; each round selects the best kernels
as new baselines and consults expert suggestions to plan the next exploration.

### 2.5 Evaluation & results
- Benchmarks: **KernelBench** (+ MultiKernelBench, TritonBench context) and a
  harder in-house suite with **dynamic shapes** to resist reward hacking.
- Across five DSL–backend combinations: **up to 100% correctness** on KernelBench
  Level 1; **85–91%** on the harder dynamic-shape benchmark.
- **1.46× geometric-mean speedup** over PyTorch Eager.

### Relevance to Agent-Bench
A concrete **`artifact_generation` + `cost_and_latency`** benchmark: agents are
scored on *correctness* (efficacy) and *speedup* (a performance ratio), with a
harder dynamic-shape variant designed to defeat reward hacking — directly
echoing the construct-validity concerns in our other references.

---

## 3. Formulas / metric definitions

**Speedup** of a generated kernel vs. baseline (latencies `t`):
```
speedup = t_baseline / t_kernel        # > 1 means faster than PyTorch Eager
```

**Geometric-mean speedup** across `N` tasks (the headline 1.46×):
```
GM = ( Π_{i=1..N} speedup_i ) ^ (1/N)
   = exp( (1/N) · Σ_{i=1..N} ln(speedup_i) )
```
(Geometric mean is used for ratios so no single task dominates.)

**Correctness rate:**
```
Correctness = (Tasks functionally correct vs. reference) / (Total tasks)
```

**fast_p** (KernelBench) — correct *and* at least `p×` faster than baseline:
```
fast_p = ( #tasks with correct=true AND speedup ≥ p ) / (Total tasks)
   fast_1 = fraction correct and at least as fast as baseline
```

**Iterative refinement** (conceptual loop until success or budget):
```
k_0 = Coder(Designer(spec))
loop:  (ok, profile) = Verifier(k_t)
       if ok and fast enough: return k_t
       k_{t+1} = Coder(Designer(spec, feedback=profile/errors))
```
