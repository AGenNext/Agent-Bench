# Agent-Bench Platform

A multi-tenant platform that **measures, publishes, and certifies** agents
against a supplied protocol — SWE-bench-style benchmarking as a service.

## It answers exactly two questions

1. **How good is this agent?** — measured against a supplied `protocol@version`.
2. **What should it do next to reach the next level?** — the improvement areas.

Agent-Bench does **not** define protocols. It measures against the protocol you
bring (e.g. AMB-001 for the memory attribute), per agent attribute, and reports
a rank plus improvement areas.

Built on **SurrealDB** (embedded or remote via the `any` engine), **Axum**, and a
pure-Rust scoring engine derived from the [reference library](../benchmarks/reference/).

## Architecture

```
                 ┌─────────────────────────────────────────────┐
   X-Tenant ───► │  Axum API  (src/api.rs)                      │
                 │   /v1/agents  /v1/benchmarks                 │
                 │   /v1/runs    /v1/leaderboard/:benchmark     │
                 └───────────────┬─────────────────────────────┘
                                 │
              ┌──────────────────▼───────────────────┐
              │  Scoring engine (src/metrics,         │  ← pure Rust, no I/O
              │   src/scoring, src/attributes)        │     fully unit-tested
              └──────────────────┬───────────────────┘
                                 │
              ┌──────────────────▼───────────────────┐
              │  Store (src/db.rs)                    │
              │   SurrealDB (any engine)              │
              │   namespace-per-tenant + migrations   │
              └───────────────────────────────────────┘
```

### Multi-tenancy
Each tenant maps to its own **SurrealDB namespace** (native isolation). The
`X-Tenant` request header selects the namespace; migrations under
`migrations/*.surql` are applied once per namespace and tracked in a
`_migration` table.

### Scoring engine (`src/metrics/`, `src/attributes/`)
Pure Rust, no DB — reusable by the API, a CLI, or offline analysis. Each module
maps to a reference doc:

| Module | Metric | Reference |
|---|---|---|
| `metrics/clear.rs` | CNA, CPS, SCR, PAS, pass@k, composite | `clear-enterprise-evaluation.md` |
| `metrics/ranking.rs` | Spearman ρ, Kendall τ, Mid-Range Difficulty Filter | `efficient-benchmarking-ai-agents.md` |
| `metrics/progress.rs` | progress rate, success rate, grounding accuracy | `agentboard.md` |
| `metrics/perf.rs` | correctness, geomean speedup, `fast_p` (multi-hardware) | `akg-kernel-agent.md` |
| `scoring.rs` | run aggregation + improvement-area detection | (bridges the above) |
| `attributes/memory.rs` | per-attribute memory eval + comparative ranking | AMB-001 |

### Multi-kernel / multi-hardware
Runs carry `hardware` and `dsl`; `metrics/perf.rs` scores speedup vs. a baseline.
Leaderboards are sliceable per hardware backend (`?hardware=gpu-a100`), because
an agent that wins on GPU may lose on NPU.

### Attributes
Agents are evaluated one attribute at a time. **Memory** is implemented
(`src/attributes/memory.rs`), scored against AMB-001, with a comparative
evaluator that ranks frameworks (Agent-Memory vs Mem0/Zep/Letta) and reports each
framework's gap-to-leader.

### Improvement areas & SurrealML
`improvement_areas()` flags an agent's weakest dimensions. With the `surrealml`
feature, `src/ml.rs` calls a SurrealML model (`ml::*` SurrealQL functions) for
readiness prediction; otherwise it falls back to the deterministic composite.

## Build & run

```bash
# Pure scoring core — fast, fully tested, no heavy deps:
cargo test

# Full multi-tenant server (embedded SurrealDB + Axum):
cargo test --features server
cargo run  --features server                       # in-memory store on :8080
AGENTBENCH_DB_PATH=./data cargo run --features server   # persistent SurrealKV
AGENTBENCH_DB_URL=ws://surrealdb:8000 cargo run --features server  # remote server

# With native ML inference:
cargo run --features surrealml
```

## API

| Method | Path | Description |
|---|---|---|
| `GET`  | `/health` | liveness |
| `POST` | `/v1/agents` | register an agent (`{name, scaffold, model, version}`) |
| `GET`  | `/v1/agents` | list tenant's agents |
| `POST` | `/v1/benchmarks` | upsert a benchmark suite |
| `GET`  | `/v1/benchmarks` | list benchmarks |
| `POST` | `/v1/runs` | submit a run (`{agent_id, benchmark_id, hardware, dsl, trials, results[]}`) → scored |
| `GET`  | `/v1/leaderboard/:benchmark_id[?hardware=…]` | ranked agents + improvement areas |

All endpoints require the `X-Tenant: <enterprise>` header.

### Example

```bash
curl -s localhost:8080/v1/agents -H 'X-Tenant: acme' \
  -H 'content-type: application/json' \
  -d '{"name":"Strong","scaffold":"plan-execute","model":"opus","version":"1"}'

curl -s localhost:8080/v1/runs -H 'X-Tenant: acme' \
  -H 'content-type: application/json' \
  -d '{"agent_id":"strong_1","benchmark_id":"swe-lite","trials":1,
       "results":[{"task_id":"t1","success":1.0,"cost_usd":0.2}]}'

curl -s 'localhost:8080/v1/leaderboard/swe-lite?hardware=gpu-a100' -H 'X-Tenant: acme'
```

## Deployment

See [`deploy/`](deploy/) for Kubernetes manifests (SurrealDB + the stateless API)
and the `Dockerfile`.
