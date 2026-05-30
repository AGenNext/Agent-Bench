# CLEAR: Multi-Dimensional Enterprise Agent Evaluation — Glossary, Methods & Formulas

Reference notes distilled from:

> Sushant Mehta. *Beyond Accuracy: A Multi-Dimensional Framework for Evaluating
> Enterprise Agentic AI Systems.* arXiv:2511.14136v1, November 2025.

**One-line summary.** Accuracy-only benchmarks mislead enterprise deployment.
**CLEAR** evaluates agents on five dimensions — **C**ost, **L**atency,
**E**fficacy, **A**ssurance, **R**eliability — and predicts production success
far better (ρ = 0.83) than accuracy alone (ρ = 0.41). Optimizing for accuracy
alone yields agents 4.4–10.8× more expensive than Pareto-efficient alternatives.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **CLEAR** | The proposed five-dimensional framework: Cost, Latency, Efficacy, Assurance, Reliability. |
| **Cost (C)** | Economic efficiency: API token consumption, inference cost, and infrastructure overhead (USD per task). |
| **Latency (L)** | End-to-end response time across planning, execution, and reflection phases. |
| **Efficacy (E)** | Task-completion quality (accuracy), augmented with domain-specific measures (e.g. test-pass rate for code). |
| **Assurance (A)** | Safety, security, and policy compliance — prompt-injection resistance, data-leak prevention, hallucination, graceful failure. |
| **Reliability (R)** | Consistency across repeated runs, measured by pass@k. |
| **Cost-Normalized Accuracy (CNA)** | Accuracy per dollar (×100); enables fair comparison of expensive vs. cheap agents. |
| **Cost Per Success (CPS)** | Total cost divided by successful tasks; charges failed attempts to the cost of success. |
| **SLA Compliance Rate (SCR)** | Fraction of tasks completed within a domain-specific latency SLA. |
| **Policy Adherence Score (PAS)** | `1 −` (policy violations / policy-critical actions). A single violation can be a hard enterprise failure. |
| **pass@k** | Probability of achieving `k` consecutive successes; reveals brittleness single-run scores hide. |
| **SLA threshold** | Domain-specific time budget (e.g. 3 s customer support, 30 s code generation). |
| **Pareto frontier** | The set of agents not dominated on all CLEAR dimensions simultaneously. |
| **Composite CLEAR score** | Weighted sum of the five (normalized) dimensions; weights sum to 1 and are application-specific. |
| **Cost-controlled evaluation** | Comparing agents at matched cost — absent in major benchmarks, causing up to 50× cost variation for similar accuracy. |
| **Enterprise Task Suite** | 300 tasks across six domains (customer support, data analysis, process automation, software dev, compliance, multi-stakeholder) with ground-truth cost/latency/policy annotations, 5–15 steps each. |
| **Construct/task/outcome validity failure** | Benchmark defects where trivial agents pass or graders credit wrong answers (found in 7–8 of 10 benchmarks). |
| **Lab-to-production gap** | The ~37% performance drop observed moving from benchmark to deployment. |

---

## 2. Methods

### 2.1 Motivation — three gaps in current benchmarks
1. **Cost ignored** — agents make hundreds–thousands of API calls; leading agents
   show **50× cost variation** ($0.10–$5.00/task) for similar accuracy.
2. **Reliability unmeasured** — single-run scores mask brittleness; GPT-4 agents
   drop from **60% pass@1 to 25% pass@8** on τ-bench.
3. **Enterprise dimensions absent** — security, policy compliance, latency SLAs,
   and graceful error handling go unmeasured, yielding a ~37% lab-to-production gap.

### 2.2 The five dimensions and their metrics
- **Cost** → CNA and CPS.
- **Latency** → end-to-end time and SLA Compliance Rate (SCR) against domain SLAs.
- **Efficacy** → domain-aware accuracy (e.g. functional test-pass for code,
  intent-classification accuracy for support).
- **Assurance** → PAS plus security testing (500 adversarial prompt-injection
  cases, data-leak, hallucination, graceful failure).
- **Reliability** → pass@3, pass@5, pass@8; production target `pass@8 ≥ 80%`
  for mission-critical use.

### 2.3 Composite scoring
A single comparable number is a weighted sum of normalized dimensions (Cost and
Latency min-max normalized, weights summing to 1). Default equal weights
`wᵢ = 0.2`; enterprises customize (e.g. financial services `w_R = 0.4, w_A = 0.3`;
customer-facing `w_L = 0.35`).

### 2.4 Experimental design
- **Agents (6):** ReAct-GPT4, ReAct-GPT-o3, Reflexion, Plan-Execute, ToolFormer,
  Domain-Tuned (fine-tuned Llama).
- **Tasks:** all 300; reliability measured on 60 representative tasks ×10 runs.
- **Expert validation:** 15 enterprise-AI leads (mean 5.9 yrs), 40 tasks each,
  5-point deployment-readiness rating (inter-rater `α = 0.78`).

### 2.5 Key findings
- Accuracy-optimal configs cost **4.4–10.8×** more than Pareto-efficient ones.
- **Reflexion** has highest efficacy (74.1%) but costs 5.12× ReAct-GPT-o3 for
  +5.4 points; dominated by **Plan-Execute** (71.9% efficacy at 4.1× lower cost,
  better reliability).
- **Domain-Tuned** wins CNA (260.4) and reliability (pass@8 = 72.8%) despite a
  smaller base model — task-specific tuning beats raw scale for enterprise use.
- **Brittleness:** pass@1 68–74% drops to pass@8 52–73%; ReAct-GPT4 loses 19.4 pts.
- **Validity:** CLEAR predicts expert deployment readiness (ρ = 0.83) far better
  than efficacy-only (ρ = 0.41) or efficacy+cost (ρ = 0.58).

### 2.6 Recommendations for benchmark design
Mandatory cost transparency (tokens, API cost, inference time); reliability via
pass@k with minimum `k = 8`; domain-specific evaluation (15–25% performance
varies across domains); reflection loops give marginal accuracy at
disproportionate cost/latency.

---

## 3. Formulas

**(1) Cost-Normalized Accuracy (CNA):**

```
CNA = (Accuracy / Cost) × 100          # Cost in USD per task
```

**(2) Cost Per Success (CPS):**

```
CPS = Total Cost / Number of Successful Tasks
```

**(3) SLA Compliance Rate (SCR):**

```
SCR = (Tasks Completed Within SLA / Total Tasks) × 100%
```

**(4) Policy Adherence Score (PAS):**

```
PAS = 1 − (Policy Violations / Total Policy-Critical Actions)
```

**(5) Reliability — pass@k:**

```
pass@k = (Trials with k consecutive successes) / (Total trials)
```

**Composite CLEAR score:**

```
CLEAR = w_C·C_norm + w_L·L_norm + w_E·E + w_A·A + w_R·R
        with  Σ wᵢ = 1   (default wᵢ = 0.2; Cost & Latency min-max normalized)
```
