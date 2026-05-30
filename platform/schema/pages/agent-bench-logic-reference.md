# Agent-Bench — Logic Reference

Notion-importable reference for the logic implemented in Agent-Bench. Each metric
is given precisely: **formula → DDL/DSL function → Rust path**. Terms are used in
their exact sense; "built" vs "planned" is marked so nothing is overstated.

---

## 1. Purpose (exact)

Agent-Bench **measures** a subject and **reports** two answers, per attribute,
against a **supplied protocol** (it does not define protocols):

1. **How good is this agent?** → a grade + per-metric scores.
2. **What should it do next to reach the next level?** → improvement areas.

> Built: measurement + reporting + writing results to the subject card.
> Planned (not yet built): cryptographic signing / attestation of results.
> So "certify/verifiable" is a *planned* property, not a current claim.

Subject = any registry entry (artifact registry: skill, prompt, model, template,
uikit; image registry: agent, API, app, SaaS, runtime), addressed by **DID**,
described by a **card**.

---

## 2. Evaluation logic (flow)

```
per-task/query results  ──score()──►  scores
                          + thresholds (protocol) ──evaluate()──►  verdict
                                                       ├─ grade  (Q1)
                                                       └─ improvement_areas (Q2)
verdict + TestConditions ──CardEval──►  card patch (writes to subject's card)
```

- **Grade** = fraction of the attribute's metrics that meet their threshold, in [0,1].
- **Passed** = every metric meets its threshold.
- **Improvement areas** = metrics below threshold, ordered by severity (distance
  from threshold), worst first.

---

## 3. Metrics — formula · DDL/DSL · code

DDL/DSL lives in `platform/schema/metrics.surql` (verified executing by
`platform/tests/schema_ddl.rs`). Rust reference in `platform/src/metrics/`.

### CLEAR (`src/metrics/clear.rs`)
| Metric | Formula | DSL fn |
|---|---|---|
| efficacy | mean(success) | — |
| CNA (cost-normalized accuracy) | accuracy% / cost = (efficacy·100) / total_cost | `fn::cna` |
| CPS (cost per success) | total_cost / successes (∞ if 0) | `fn::cps` |
| SCR (SLA compliance rate) | within_sla / total | `fn::scr` |
| PAS (policy adherence) | 1 − violations / critical (1 if no critical) | `fn::pas` |
| pass@k | windows with k consecutive successes / total windows | — |
| CLEAR composite | Σ wᵢ·dimᵢ (cost,lat,eff,ass,rel; Σwᵢ=1) | `fn::clear_composite` |

### Perf — multi-hardware (`src/metrics/perf.rs`)
| Metric | Formula | DSL fn |
|---|---|---|
| speedup | baseline_latency / kernel_latency (correct only) | `fn::speedup` |
| geomean speedup | (Π speedupᵢ)^(1/n) = exp(mean ln) | `fn::geomean` |
| correctness | correct / total | `fn::correctness` |
| fast_p | (correct AND speedup ≥ p) / total | — |

### Progress (`src/metrics/progress.rs`)
| Metric | Formula | DSL fn |
|---|---|---|
| progress rate (continuous) | max(matching_scores), clamped [0,1] | `fn::progress_continuous` |
| progress rate (subgoal) | completed / k | `fn::progress_subgoal` |
| success rate | passed / total | `fn::success_rate` |
| grounding accuracy | valid_actions / total_actions | `fn::grounding_accuracy` |

### Ranking (`src/metrics/ranking.rs`)
| Metric | Formula | DSL fn |
|---|---|---|
| Spearman ρ | Pearson correlation on rank vectors (midrank ties) | — |
| Kendall τ (τ_b) | (C − D) / √((n₀−n₁)(n₀−n₂)) | — |
| pairwise-correct prob | (τ + 1) / 2 | `fn::pairwise_correct_prob` |
| mid-range filter | keep tasks with pass rate ∈ [0.30, 0.70] | — |
| reduction ratio | 1 − selected / total | `fn::reduction_ratio` |

(Spearman/Kendall are array computations kept in Rust as the reference; the
closed-form metrics are also defined in DSL.)

---

## 4. Memory attribute (`src/attributes/memory.rs`)

The only **implemented** attribute. Protocol: **AMB-001**.

**Scores** (DDL: `memory_scores` in `schema/memory_attribute.surql`):
recall_accuracy, gap_handling, conflict_handling_avg (0–2), cold_start_latency_ms,
p50_recall_latency_ms, p99_recall_latency_ms, external_deps_required.

**Thresholds** (AMB-001, DDL `memory_thresholds:AMB-001@0.1.0`, seeded as data):
recall ≥ 0.70 · gap ≥ 0.50 · conflict_avg ≥ 1.0 · cold_start ≤ 5000ms ·
p50 ≤ 500ms · p99 ≤ 2000ms.

**Verdict** = grade (metrics passed / 6) + passed + improvement_areas (worst-first).

**Comparison** (`compare()`): rank frameworks (Agent-Memory vs Mem0/Zep/Letta),
per-metric leader, and the focal framework's gap-to-leader per metric (its path
to the next level).

Metric families (recognized agent performance metrics):
- **Agent EVAL** → recall, gap, conflict
- **Agent SLA** → cold-start, p50/p99 latency
- **Agent GPA** → the composite grade

---

## 5. Card write (`src/card.rs`)

Agent-Bench **writes the result into the subject's card** (the registry-owned
metadata descriptor), as a documented, reproducible claim.

`CardEval` (DDL: `card_eval`):
- `subject_did`, `attribute`, `grade`, `passed`, `metrics`, `improvement_areas`,
  `evaluated_at`
- `conditions` (TestConditions): `protocol@version`, `dataset`, `sample_size`,
  `trials`, `hardware`, `dsl`, `evaluator_version`, `notes`

`as_card_patch()` → `{ "evaluations": { "<attribute>": { … } } }`, keyed by
attribute so repeated runs update in place.

Meaning: *"tested under these conditions → this result."*

---

## 6. Service (`server` feature)

- Multi-tenant: tenant → SurrealDB **namespace**; selected by `X-Tenant`.
- API: `POST /v1/agents`, `/v1/benchmarks`, `/v1/runs`;
  `GET /v1/leaderboard/:benchmark_id[?hardware=…]` (rankings are per-hardware).
- Store: SurrealDB `any` engine — embedded (`memory`/`surrealkv://`) or remote
  (`ws://…`); versioned `.surql` migrations per namespace.

---

## 7. Status

| Built (27 tests green) | Planned |
|---|---|
| scoring engine (CLEAR, ranking, progress, perf), memory attribute + comparison, card writes with conditions, DDL/DSL definitions verified in-engine, multi-tenant API, multi-hardware leaderboard, SurrealDB store, deploy manifests | other attributes (runtime, tools, rules, skills); signed result attestation; remote ClickHouse telemetry; reconcile controller |

Tests: 24 lib + 3 e2e (`cargo test --features server`) + 3 DDL (`--test schema_ddl`).

---

## 8. Definitions are DDL/DSL, not prose

The authoritative definitions live in `platform/schema/*.surql` (SurrealQL):
metric formulas as `DEFINE FUNCTION`, result shapes + thresholds as
`DEFINE TABLE/FIELD` with range `ASSERT`s. This document is a *reference* over
those definitions; the schema files are the source of truth and are verified to
execute by `tests/schema_ddl.rs`.
