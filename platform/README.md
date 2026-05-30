# Agent-Bench Platform

A cloud-native, **multi-tenant** enterprise agent evaluation & leaderboard
platform — SWE-bench-style benchmarking as a service. Each enterprise submits
its agents, runs them against benchmark suites, and gets **ranked** results plus
**identified improvement areas**.

Built natively on **SurrealDB** (embedded), **Axum**, and a pure-Rust scoring
engine derived from the [reference library](../benchmarks/reference/).

## Positioning — we bring the *model*, not the infra

> Hugging Face has the infra; we bring the model.

The infrastructure is **adopted, not rebuilt**, and **enforcement is owned by the
runtime**:

| Layer | Owned by (adopted) |
|---|---|
| Kernel / model hosting & registry | **Hugging Face** (Kernels Hub, model hosting) |
| Control plane / runtime | **AgentField** + the **CNCF stack** (k8s, Kata, Cilium, Argo, …) |
| **Policy & access enforcement** | **SurrealDB enforces** at the data layer — verifies Agent-Auth JWTs, applies record-level `PERMISSIONS` (see [docs/surrealdb-security.md](docs/surrealdb-security.md)). The app never enforces. |
| **Telemetry & measurement** | **ClickHouse** — high-volume run/step metrics, time-series, leaderboard analytics, drift signals (the *measurement* store; SurrealDB stays the transactional + enforcement store) |
| Identity / authz / governance | **Agent-Auth / IGA / PAM / Guard** (see [docs/ecosystem.md](docs/ecosystem.md)) |

What **Agent-Bench brings is the evaluation *model***: the scoring engine
(CLEAR · rank-fidelity · progress · perf/speedup), mid-range task selection,
benchmark contracts, and the ranked leaderboard with improvement areas. That —
not infrastructure — is the value we add on top.

### Cloud-native by construction

Staying cloud-native is a constraint, not an afterthought — it falls out of the
choices above:

- **Stateless, horizontally scalable** API; all state in SurrealDB → scale
  replicas freely (12-factor).
- **Adopt, don't build** — orchestration, sandboxing, autoscaling, policy,
  identity, observability, cost are CNCF/HF projects ([cncf-stack.md](docs/cncf-stack.md)).
- **Enforcement owned by the runtime** — SurrealDB (data), Cilium/Kata/OPA
  (infra), never the app.
- **Declarative everything** — Terraform (infra graph) → Helm (app graph) →
  `EvaluationPolicy` CRD, GitOps-synced ([declarative-stack.md](docs/declarative-stack.md)).
- **Reconciled, not scripted** — a Kubernetes-style loop converges to desired
  eval state ([reconciliation.md](docs/reconciliation.md)).
- **Two-store separation** — SurrealDB (transactional + enforcement) vs.
  ClickHouse (telemetry/measurement) ([telemetry.md](docs/telemetry.md)).
- **Observable & identity-aware from day one** — OpenTelemetry, W3C DIDs,
  per-record audit.

The platform behaves like infrastructure (a service you call), not a library you
import — exactly AgentField's model.

### The registry is the bearer of trust

**Everything is a registry artifact.** Via OCI/ORAS, the registry holds not just
container images and kernels but **agents, models, benchmark suites, scenario
fixtures, result packages, reproducibility manifests, and the `EvaluationPolicy`
itself** — every artifact content-addressed (by digest), versioned, and signed.
The benchmark contract's *result package* is literally an OCI artifact you pull
by digest, which is what makes a run reproducible.

Trust is then anchored in the **registry** (Harbor for images, HF Kernels Hub for
kernels, the agent/model registry) — not in the app, the network, or a prompt:

- **Signed + attested** — artifacts are signed (Sigstore/cosign) with SLSA
  provenance; nothing runs unless its signature verifies against the registry.
- **Benchmark scores become attestations** — Agent-Bench publishes a signed
  **result-package attestation** (score, hardware/DSL coordinates, reproducibility
  manifest) *to the registry*, bound to the artifact's digest. A score is then a
  verifiable claim, not a number in a database.
- **Gates verify against the registry** — admission (OPA/Gatekeeper) and
  promotion (Agent-LCM) check the registry-anchored signature + a fresh passing
  attestation before anything ships ([reconciliation.md](docs/reconciliation.md)).
- **Trust flows outward** — Agent-Trust and Agent-Compliance consume these
  registry attestations as evidence; the registry is the root they chain to.

So the chain of custody is: *signed artifact in the registry → benchmarked →
signed result attestation back in the registry → gates verify → promote.* The
registry carries the trust end to end.

## Architecture

```
                 ┌─────────────────────────────────────────────┐
   X-Tenant ───► │  Axum API  (src/api.rs)                      │
                 │   /v1/agents  /v1/benchmarks                 │
                 │   /v1/runs    /v1/leaderboard/:benchmark     │
                 └───────────────┬─────────────────────────────┘
                                 │
              ┌──────────────────▼───────────────────┐
              │  Scoring engine (src/metrics, scoring)│  ← pure Rust, no I/O
              │   CLEAR · rank fidelity · progress    │     fully unit-tested
              └──────────────────┬───────────────────┘
                                 │
              ┌──────────────────▼───────────────────┐
              │  Store (src/db.rs)                    │
              │   embedded SurrealDB                  │
              │   namespace-per-tenant + migrations   │
              └───────────────────────────────────────┘
```

### Multi-tenancy
Each enterprise tenant maps to its own **SurrealDB namespace** (native
isolation). The `X-Tenant` request header selects the namespace; migrations
under `migrations/*.surql` are applied once per namespace and tracked in a
`_migration` table (SurrealDB schema-migration-library pattern).

### Scoring engine (`src/metrics/`)
Pure Rust, no DB — reusable by the API, a CLI, or offline analysis. Each module
maps to a reference doc:

| Module | Metric | Reference |
|---|---|---|
| `metrics/clear.rs` | CNA, CPS, SCR, PAS, pass@k, composite | `clear-enterprise-evaluation.md` |
| `metrics/ranking.rs` | Spearman ρ, Kendall τ, Mid-Range Difficulty Filter | `efficient-benchmarking-ai-agents.md` |
| `metrics/progress.rs` | progress rate, success rate, grounding accuracy | `agentboard.md` |
| `scoring.rs` | run aggregation + improvement-area detection | (bridges the above) |

### Improvement areas & SurrealML
`improvement_areas()` flags an agent's weakest CLEAR dimensions. With the
`surrealml` feature, `src/ml.rs` calls a SurrealML model (`ml::*` SurrealQL
functions) to predict deployment-readiness natively in the database; otherwise
it falls back to the deterministic composite.

## Build & run

```bash
# Pure scoring core — fast, fully tested, no heavy deps:
cargo test

# Full multi-tenant server (embedded SurrealDB + Axum):
cargo test --features server
cargo run  --features server          # in-memory store on :8080
AGENTBENCH_DB_PATH=./data cargo run --features server   # persistent SurrealKV

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
| `POST` | `/v1/runs` | submit a run (`{agent_id, benchmark_id, trials, results[]}`) → scored |
| `GET`  | `/v1/leaderboard/:benchmark_id` | ranked agents + improvement areas |

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

curl -s localhost:8080/v1/leaderboard/swe-lite -H 'X-Tenant: acme'
```

## Roadmap (SurrealDB-native capabilities)

- **Time-series telemetry** — store every run as a time-series record for
  per-agent trend tracking and drift monitoring (temporal leaderboard).
- **Aggregation queries** — leaderboard aggregation pushed fully into SurrealQL
  (`GROUP BY`, `math::*`) for best-run-per-agent at scale.
- **Vector search / RAG** — embed improvement-area explanations and retrieve
  remediation guidance from the reference library.
- **Mid-range task selection** — use historical pass rates to auto-select the
  30–70% band per benchmark, cutting evaluation cost 44–70%.
- **Observability** — OpenTelemetry tracing across API + DB.
