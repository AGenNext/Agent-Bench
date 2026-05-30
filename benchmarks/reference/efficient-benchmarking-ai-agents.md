# Efficient Benchmarking of AI Agents — Glossary, Methods & Formulas

Reference notes distilled from:

> Franck Ndzomga. *Efficient Benchmarking of AI Agents.* arXiv:2603.23749v1, March 2026.
> Code/data: https://github.com/fsndzomga/efficient-benchmarking-ai-agents

**One-line summary.** Agent leaderboards consume *rankings*, not absolute
scores. Rankings survive scaffold/temporal distribution shift even when
absolute-score prediction collapses. Exploiting this asymmetry, you can
evaluate new agents only on **mid-difficulty tasks (30–70% historical pass
rate)** and cut 44–70% of tasks while preserving rank fidelity.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **Agent** | An AI system that solves tasks via interactive rollouts: multi-step reasoning, tool use, and interaction with external environments. |
| **Scaffold (harness)** | The framework wrapping the underlying model — it governs tool use, memory, retry logic, and execution flow. Performance depends on the scaffold, not just the model. |
| **Scaffold-driven distribution shift** | The shift in the task-performance relationship caused by changing scaffolds. Unique to agent benchmarks; absent in static LM evaluation. |
| **Performance matrix (X)** | An `n × m` matrix in `[0,1]` where `n` = agents, `m` = tasks. Entry `Xij` = whether (or fraction of trials in which) agent `i` solved task `j`. |
| **Cell** | A single agent–task entry `Xij`. Binary for one trial; fractional success rate for multiple trials. |
| **Pass rate** | Fraction of agents (historically) that solve a given task. Drives task difficulty classification. |
| **Benchmark score (yᵢ)** | An agent's mean success across all tasks; the value a full benchmark run reports. |
| **Benchmark reduction** | Selecting a subset `S` of tasks (`\|S\| = k ≪ m`) whose performance predicts the full-benchmark signal. |
| **Task selection method (σ)** | A function mapping the full task set (+ optional history) to a subset `S`. |
| **Score prediction** | Predicting each agent's *absolute* full-benchmark score. Requires calibration; sensitive to shift. Measured by R². |
| **Rank prediction** | Predicting the *ordering* of agents. Robust to affine distortions of the score scale. Measured by Spearman ρ and Kendall τ. |
| **ρ–R² divergence** | The central empirical finding: under shift, R² collapses while ρ and τ stay high. |
| **Item Response Theory (IRT)** | Psychometric theory of latent ability/item difficulty; motivates selecting items near 50% pass rate where they are most informative. |
| **Fisher information** | `I(θ) = p(1−p)` — information a Bernoulli item carries about latent ability; maximized at `p = 0.5`. |
| **Mid-Range Difficulty Filter (MR)** | The proposed rule: keep all tasks with pass rate in `[0.30, 0.70]`. Deterministic, optimization-free. |
| **Greedy-k** | Baseline: forward-select the `k` tasks that maximize leave-one-agent-out R² under Ridge regression. Prone to overfitting under shift. |
| **Random-k** | Baseline: sample `k` tasks uniformly (averaged over 100 seeds). High variance; unusable as a deployment strategy. |
| **Hardest-k / Easiest-k** | Baselines: the `k` lowest / highest pass-rate tasks. Hardest-k is catastrophic; Easiest-k is competitive only when it overlaps the MR band. |
| **Stratified-k** | Baseline: uniform sampling across difficulty deciles. Hurt by forced inclusion of hard, noisy tasks. |
| **Ridge regression** | Regularized linear regression (`α = 1.0`) used to calibrate full-benchmark score from subset performance. |
| **Nested cross-validation** | Task selection occurs *inside* the CV loop so the test agent's data never informs selection — prevents optimistic bias. |
| **LOAO** | Leave-One-Agent-Out: predict each held-out agent from all others. |
| **Within-Scaffold LOAO** | LOAO restricted to agents sharing one scaffold (lowest distribution shift). |
| **LOSO** | Leave-One-Scaffold-Out: train on other scaffolds, test on a held-out scaffold (high shift). |
| **Temporal expanding window** | Train on agents submitted before time `t`, predict the agent at `t`. Mirrors real leaderboard operation. |
| **HAL** | Holistic Agent Leaderboard (Kapoor et al., 2026) — standardized agent evaluation; source of seven of the eight benchmarks. |
| **Terminal-Bench 2.0** | CLI-agent benchmark (89 tasks ×5 trials, 101 agents, 23 scaffolds) with rich temporal structure for out-of-distribution validation. |
| **Cold start** | The up-front cost of fully evaluating ~5–15 agents before MR reduction becomes reliable. |
| **Construct validity failure** | When a benchmark fails to measure its target capability (trivial agents pass, or graders credit wrong answers), biasing absolute scores. |
| **Saturation** | When agents universally pass tasks; the mid-range band empties and reselection or retirement is required. |

---

## 2. Methods

### 2.1 Problem setup
Given a per-task performance matrix `X ∈ [0,1]^{n×m}`, the full benchmark
score vector is the row-mean of `X`. The goal of reduction is to find a column
subset `S` (`k ≪ m`) such that performance on `S` predicts the full score
vector `y`. Score calibration uses **Ridge regression** (`ŷ = X_S β`, `α = 1.0`);
for rank prediction alone, an unweighted mean over selected tasks suffices.

Two prediction targets are distinguished:
- **Score prediction** — recover absolute scores (needs calibration; fragile under shift).
- **Rank prediction** — recover the agent ordering (robust; what leaderboards actually use).

### 2.2 The Mid-Range Difficulty Filter (MR)
Keep every task whose historical pass rate `p ∈ [0.30, 0.70]`. Rationale: under
a Bernoulli/logistic response model, Fisher information about latent ability is
`I(θ) = p(1−p)`, maximized at `p = 0.5` and at half-maximum near `p ≈ 0.146`
and `p ≈ 0.854`. The 30–70% band is a narrower, practical slice of the full
high-information region (15–85%) that targets ~50% task reduction. MR is
*deterministic and optimization-free*. It is skipped when the mid-range band is
too sparse (e.g. SciCode, only ~4 tasks in band).

### 2.3 Baselines (matched budget)
For each fold the MR band size `kᵢ` is computed on the **training agents
only**, then every baseline is evaluated at exactly that budget `k`:
Greedy-k (forward selection on LOAO R²), Random-k (uniform, 100 seeds),
Hardest-k, Easiest-k, Stratified-k.

### 2.4 Evaluation protocols (increasing distribution shift)
1. **Within-Scaffold LOAO** — same scaffold only (≥10 runs per scaffold).
2. **LOAO** — across all agents; tasks reselected per fold.
3. **Random 80/20 splits** — ×100 seeds.
4. **LOSO** — held-out scaffold.
5. **Temporal expanding window** — predict the newest agent from the past.

All protocols use **nested cross-validation** (selection inside the loop). The
three reported metrics are Spearman ρ, Kendall τ, and R².

### 2.5 Key empirical results
- **ρ–R² divergence:** ρ stays 0.90–0.96 across protocols while R² falls from
  0.90 (LOAO) to 0.65 (LOSO) to 0.54 (random splits); R² even goes negative on
  Online Mind2Web under LOSO while ρ > 0.90.
- **MR wins:** highest mean (ρ = 0.94) and best worst-case (ρ = 0.87);
  best on 5/8 benchmarks, close second otherwise. Greedy/Random degrade to
  ρ ≈ 0.54–0.56 under shift.
- **What matters is exclusion, not inclusion:** both MR and Easiest-k succeed
  by dropping the *hardest* (noisy) tasks; Easiest-k is competitive only where
  it happens to overlap the MR band (overlap vs. advantage `r = −0.71`).
- **Cost:** task reduction 44–70% (median 58%), with per-run savings up to
  ~$253 (Online Mind2Web).

### 2.6 Recommended deployment protocol
- **Phase 0 — Cold start:** estimate per-task pass rates from 5–10 historical runs.
- **Phase 1 — Setup:** select the 30–70% band; if <10% of tasks qualify, widen
  to 25–75%, then to the IRT bound 15–85%.
- **Phase 2 — Ongoing:** run new agents on selected tasks only; report rankings
  (or Ridge-predicted scores with confidence intervals).
- **Phase 3 — Maintenance:** refit Ridge weights every 5–10 agents; trigger
  full reselection only if validation ρ < 0.75.

**Inappropriate when:** few agents are expected (cold start not recouped),
absolute capability claims are required, or <10% of tasks fall in the band.

---

## 3. Formulas

**(1) Per-agent benchmark score** — mean across all tasks:

```
yᵢ = (1/m) · Σ_{j=1..m} Xᵢⱼ
```

**(2) Benchmark score vector:**

```
y = (y₁, …, yₙ)ᵀ
```

**(3) Coefficient of determination (score prediction):**

```
R² = 1 − [ Σᵢ (yᵢ − ŷᵢ)² ] / [ Σᵢ (yᵢ − ȳ)² ]
```
where `ŷᵢ` is the Ridge-predicted score and `ȳ = (1/n) Σᵢ yᵢ`.

**(4) Spearman's ρ** — Pearson correlation on rank vectors (`rᵢ`, `sᵢ` =
predicted / actual ranks, ties → midranks):

```
ρ = [ Σᵢ (rᵢ − r̄)(sᵢ − s̄) ] / [ √(Σᵢ (rᵢ − r̄)²) · √(Σᵢ (sᵢ − s̄)²) ]
```

**(5) Kendall's τ (τ_b variant)** — concordant vs. discordant pairs:

```
τ = (C − D) / √[ (n₀ − n₁)(n₀ − n₂) ]
```
where `C`, `D` = concordant / discordant pair counts, `n₀ = C(n,2)` total pairs,
and `n₁`, `n₂` = pairs tied in the predicted / actual rankings.
Probabilistic reading: `(τ + 1)/2` = probability a random agent pair is ranked
correctly (e.g. τ = 0.80 ⇒ 90% of pairwise comparisons agree).

**Ridge calibration** (subset → full score):

```
ŷ = X_S · β ,   β ∈ ℝᵏ ,   regularization α = 1.0
```

**Fisher information / IRT motivation for the band:**

```
I(θ) = p(1 − p)        maximized at p = 0.5
I(θ) ≥ ½·I_max   ⇔   p ∈ [0.146, 0.854]   (full high-information region)
MR band: p ∈ [0.30, 0.70]                  (practical ~50%-reduction slice)
```
