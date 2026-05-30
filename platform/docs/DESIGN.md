# Agent-Bench — Canonical Design Map

The single source of truth for what this repo is and how each part maps to a
concrete **input → protocol → output**. Everything here is implemented (see file
paths) or is a declared schema. No philosophy — just the design.

> **Purpose.** This is the canonical entry point for any contributor — human or
> agent — to understand the repo end to end without reading all the code: its
> concept, what it answers, and the exact input/protocol/output of every
> component (with file paths). Read this first; it is designed so work can
> proceed in parallel and autonomously against an unambiguous spec.

## Top level

| | |
|---|---|
| **Concept** | A platform that **measures, publishes, and certifies** an agent against a supplied protocol. |
| **What it means** | A neutral instrument. It does **not** define protocols, metrics, or thresholds — those are inputs. |
| **Provides (answers)** | Exactly two questions: **(1) how good is this agent?** and **(2) what should it do next to reach the next level?** |
| **Input** | A **subject** + per-task/query results for one **attribute**. The subject is anything in the registries — not only a whole agent. |
| **Protocol** | Any supplied `protocol@version` (metric set + thresholds). Memory ships against **AMB-001**. |
| **Output** | A scored result (rank/grade) + improvement areas — published as a result package. |

> **Subjects under evaluation** come from two registries (owned in their
> respective repos):
> - **Artifact registry** — skill, prompt, model, template, uikit, …
> - **Image registry** — agents, APIs, apps, SaaS, runtimes, …
>
> Agent-Bench treats any of these as a subject to measure against a supplied
> protocol, answering the same two questions. The **Memory** attribute is the
> first implemented.
>
> Each registry entry has an **identity** (DID) and a **card** (its metadata
> descriptor — model-card / agent-card style). Agent-Bench **binds every result
> to the subject's identity** and **actually updates the card's values** — it
> writes the evaluation block (grade, scores, improvement areas, protocol@version)
> into the card via a patch keyed by attribute (`src/card.rs`). The card *schema*
> is owned by the registry; Agent-Bench owns and writes the evaluation values.

## Component map

Each row: **concept → meaning → answer → input → protocol → output**, with the
implementing file.

### Evaluation core (pure Rust, no I/O)

| Component | Means | Answers | Input | Protocol | Output |
|---|---|---|---|---|---|
| **Attribute eval** (`src/attributes/`) | evaluate one agent attribute at a time | both questions, per attribute | per-task results for the attribute | the attribute's protocol | attribute verdict |
| **Memory attribute** (`src/attributes/memory.rs`) | memory quality of an agent | how good is its memory / what to fix | `MemoryQueryResult[]` (recall/gap/conflict/latency) + cold-start + deps | `MemoryThresholds` (AMB-001) | `MemoryVerdict` (grade, passed, `improvement_areas`) |
| **Memory comparison** (`compare()`) | rank frameworks on memory | how good vs. others / gap-to-leader | `FrameworkMemory[]` (Agent-Memory, Mem0, Zep, Letta) | `MemoryThresholds` | `MemoryComparison` (ranking, per-metric leader, focal next-level) |
| **CLEAR** (`src/metrics/clear.rs`) | cost/latency/efficacy/assurance/reliability | quality across dimensions | `TaskObservation[]` | thresholds + weights | `ClearScores` (CNA, CPS, SCR, PAS), `pass_at_k`, composite |
| **Ranking** (`src/metrics/ranking.rs`) | rank fidelity + cost reduction | how stable is the ranking / which tasks suffice | predicted/actual scores; task pass-rates | mid-range band `[0.30,0.70]` | Spearman ρ, Kendall τ, selected task set |
| **Progress** (`src/metrics/progress.rs`) | incremental task advancement | how far did it get | matching scores / subgoals; actions | — | progress rate, success rate, grounding accuracy |
| **Perf** (`src/metrics/perf.rs`) | multi-hardware kernel/codegen perf | how fast + correct | `PerfObservation[]` (correct, baseline vs kernel latency) | per-hardware target | `PerfScores` (correctness, geomean speedup, `fast_p`) |
| **Scoring** (`src/scoring.rs`) | aggregate a run | run-level answer | `TaskResult[]` | `ClearWeights` | `RunScores` + `improvement_areas()` |
| **Card update** (`src/card.rs`) | write results to the subject's card | persists the answers on the subject | a verdict + subject DID + `protocol@v` | registry card schema | `CardEval` + `as_card_patch()` (keyed by attribute) |

### Service layer (`server` feature)

| Component | Means | Answers | Input | Protocol | Output |
|---|---|---|---|---|---|
| **API** (`src/api.rs`) | HTTP surface | serves the two questions | JSON requests + `X-Tenant` header | REST over `/v1/*` | JSON responses |
| **Run submission** (`POST /v1/runs`) | record + score a run | how good (this run) | `SubmitRun {agent_id, benchmark_id, hardware, dsl, trials, results[]}` | the benchmark | `Run {status: scored, scores}` |
| **Leaderboard** (`GET /v1/leaderboard/:id`) | rank agents | how good vs. others / what to improve | benchmark id `[?hardware=…]` | the benchmark | `LeaderboardEntry[]` (rank, scores, `improvement_areas`) |
| **Store** (`src/db.rs`) | multi-tenant persistence + enforcement | — | domain objects | SurrealQL schema (`migrations/*.surql`); namespace-per-tenant | persisted records |
| **Tenancy** (`src/tenancy.rs`) | resolve tenant → namespace | — | `X-Tenant` header | `[A-Za-z0-9_-]` | `Tenant` |
| **ML** (`src/ml.rs`, `surrealml`) | readiness prediction | how good (composite) | `RunScores` | SurrealML `ml::*` | readiness score |

### Schemas & references

| Artifact | Means | Provides | Form |
|---|---|---|---|
| **Benchmark contract** (`contracts/benchmark-contract.md`) | the result-package schema | structure of a verifiable result | YAML schema |
| **AMB-001** (`benchmarks/memory/AMB-001-benchmark.yaml`) | the memory protocol | metrics + thresholds for memory | benchmark YAML |
| **Reference library** (`benchmarks/reference/`) | canonical metric definitions (glossary + formulas) | the *meaning* of each metric the engine computes | markdown |
| **Migrations** (`platform/migrations/*.surql`) | tenant schema + fields + permissions | the storage grammar | SurrealQL `DEFINE` |

## Data flow (memory attribute, end to end)

```
MemoryQueryResult[]  ──score()──►  MemoryScores
                                      │
                MemoryThresholds ─────┤ evaluate()
                  (AMB-001)           ▼
                                  MemoryVerdict   →  Q1 grade/passed
                                                     Q2 improvement_areas

FrameworkMemory[]   ──compare()──►  MemoryComparison
  (ours + others)     (AMB-001)        ├─ ranking            →  Q1 how good vs others
                                       ├─ per_metric_leader
                                       └─ focal_next_level   →  Q2 path to next level
```

## Inputs / protocols / outputs — at a glance

| | |
|---|---|
| **Inputs** | agents/frameworks; per-task or per-query results; hardware/DSL; cold-start + dep counts |
| **Protocols** | supplied per attribute — `AMB-001` (memory), benchmark thresholds (`pass_thresholds` in YAML), `MemoryThresholds`, `ClearWeights`, mid-range band |
| **Outputs** | `MemoryVerdict`, `MemoryComparison`, `RunScores`, `LeaderboardEntry[]` — all reducing to **rank/grade + improvement areas** |

## Proposing corrections & improvements

This map is editable by every contributor — human or agent. If something here is
wrong, stale, or could be better, **propose the change**; don't work around it.

**Mechanism (uniform, so 200 agents propose consistently):**

1. **Open a draft PR.** One concern per PR.
2. **Keep this map in sync.** Any change to a component's behavior **must**
   update its row (concept · means · answers · input · protocol · output). The
   map and the code move together — a PR that drifts the map is incomplete.
3. **Tests gate it.** Behavior changes add/adjust tests; `cargo test` and
   `cargo test --features server` must stay green. New metrics ship with the
   unit tests that pin their formula.
4. **Use the proposal block** (in the PR description) so reviews are mechanical:

   ```
   PROPOSAL
   kind:       correction | improvement
   component:  <file path / map row, e.g. src/attributes/memory.rs>
   problem:    <what is wrong or missing, with evidence>
   change:     <the concrete change>
   io-impact:  <input/protocol/output changes, or "none">
   protocol:   <does this touch a protocol version? if so, bump it>
   tests:      <added/changed tests>
   ```

5. **Protocol changes are versioned.** If a proposal changes a metric's meaning
   or a threshold, it is a **new protocol version**, not an in-place edit — old
   results stay valid under their stated version.

Corrections (the map/code is wrong) and improvements (it could be better) use the
same flow; `kind:` distinguishes them. The reviewer checks: map updated, tests
green, io-impact accurate, protocol versioned if meaning changed.

## Status

| Built (tested) | Schema only / planned |
|---|---|
| scoring engine (CLEAR, ranking, progress, perf), memory attribute + comparison, multi-tenant API, SurrealDB `any`-engine store, migrations, multi-hardware leaderboard, deploy manifests — **22 lib + 3 e2e tests** | other attributes (runtime, tools, rules, skills); signed result-package attestation; remote ClickHouse/telemetry |
