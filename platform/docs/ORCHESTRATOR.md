# Agent-Bench — Orchestrator, Control Plane & Metrics

> **Concept.** Agent-Bench runs a **replica of the real world**: a substrate
> where agentic workloads do real work — run research, run a workflow/benchmark,
> create an agent, improve quality — and are **measured** while they do it. The
> platform is the **orchestrator + control plane + metrics** for that substrate.
> It is **multi-cloud and cloud-agnostic**: the control plane keeps ownership and
> availability; the clouds are interchangeable execution capacity.

This extends the canonical design map (`DESIGN.md`). Same rule holds: Bench
**measures and reports**; it does **not** define metrics (those live in
Agent-Metrics) and it does **not** define protocols (those are inputs).

## The three planes

| Plane | Means | Owns | Where |
|---|---|---|---|
| **Control plane** | the single authority that accepts, schedules, and tracks work | identity, tenancy, run records, results, leaderboards | `src/api.rs`, `src/db.rs`, `src/tenancy.rs`, SurrealDB |
| **Orchestrator** | places each workload onto execution capacity on *some* cloud | placement, lifecycle, retries, failover | `src/orchestrator.rs` *(new)* + a `Runtime` backend |
| **Metrics plane** | the measurements taken while/after work runs | references to Agent-Metrics ids; measured values | `schema/metrics.surql` (`metric_ref`), `src/evaluation.rs` |

The control plane is **stateful authority** (all state in SurrealDB). The
orchestrator is **stateless placement** (it holds no truth; it reconciles
desired vs. observed and writes observations back to the control plane). This is
why the API can scale freely and why a whole cloud can disappear without losing
control: the truth never lived in the cloud that ran the container.

## A workload "can be anything"

The unit the orchestrator places is a generic **Workload** — not a
benchmark-specific job. It mirrors the generic entity-attribute metamodel: one
shape, many kinds.

```
Workload {
  id            : did
  kind          : research | bench | create-agent | improve-quality | <any>
  subject       : did            // what is being acted on / measured (registry entry)
  protocol      : protocol@ver   // supplied; metric set + thresholds (input, not defined here)
  image         : oci-ref        // the container that does the work (cloud-agnostic)
  inputs        : object         // kind-specific payload
  placement     : PlacementSpec  // cloud/region constraints, HA policy
  budget        : { cpu, mem, wall, cost }
}
   │
   ├─ run ──►  Runtime backend (k8s Job today; cloud-agnostic by trait)
   │            executes `image` with `inputs`, emits observations
   ▼
Observation[]  ──►  evaluation.rs  ──►  AttributeScore (grade, passed, improvement areas, level)
                       (uses metric_ref values + protocol thresholds; computes no formula)
                                         │
                                         ▼
                              control plane: run record + card update + leaderboard
```

`kind` is open. Each kind is just a different `image` + `inputs` shape that
produces `Observation[]`. The platform does not special-case them — it
orchestrates, measures, records. Examples:

| kind | the container does | measured against | answers |
|---|---|---|---|
| `research` | runs a research harness, produces a cited report | a research protocol (coverage, citation validity) | how good is the output / what to deepen |
| `bench` | replays a benchmark suite for a subject | the benchmark's `protocol@ver` (e.g. AMB-001) | how good is the agent / what to fix |
| `create-agent` | synthesizes/assembles an agent into the registry | a build protocol (passes gates, has identity+card) | is it admissible / what's missing |
| `improve-quality` | runs an improvement loop on a subject | before/after on the subject's attribute | did quality rise / next lever |

## Multi-cloud, cloud-agnostic runtime

The orchestrator talks to a **`Runtime` trait**, never to a specific cloud SDK.

```rust
// src/orchestrator.rs (new)
pub trait Runtime {
    /// Place a workload's container onto this backend's capacity.
    fn submit(&self, w: &Workload) -> Result<Handle, RuntimeError>;
    /// Observed lifecycle state (for reconcile).
    fn status(&self, h: &Handle) -> Result<Phase, RuntimeError>;
    /// Collected observations once terminal.
    fn observations(&self, h: &Handle) -> Result<Vec<Observation>, RuntimeError>;
    /// Stop / reclaim.
    fn cancel(&self, h: &Handle) -> Result<(), RuntimeError>;
}
```

- **Cloud-agnostic** = the only contract is OCI image + Kubernetes Job
  semantics. A `KubeRuntime` targets any conformant cluster (EKS, GKE, AKS, k3s).
  Adding a cloud is adding a `kubeconfig` / cluster registration — **no code
  change**. Non-k8s backends (a serverless container service, a local Docker
  runner for dev) are additional `impl Runtime`, behind the same trait.
- **Keeping control** = placement decisions and results are recorded in the
  control plane *before and after* a cloud touches them. The cloud is given a
  container and credentials scoped to one workload; it returns observations; it
  is never the system of record.
- **Availability** = HA is a property of two things, kept separate:
  1. **Control plane HA** — stateless API replicas (`deploy/platform.yaml`) over
     a distributed store (SurrealDB on TiKV, see `deploy/surrealdb.yaml` HA
     block). Survives node/zone loss.
  2. **Workload HA** — `PlacementSpec` carries a failover policy; on a backend
     outage the orchestrator re-places the workload on another registered
     cluster. Idempotent because the workload id + protocol fully define the run.

```
                ┌──────────────── Control plane (authority, HA) ───────────────┐
   X-Tenant ──► │  api.rs  ──►  orchestrator.rs  ──►  db.rs (SurrealDB/TiKV)    │
                └───────┬──────────────┬──────────────────────────────┬────────┘
                        │ submit       │ submit                        │ submit
                   ┌────▼────┐    ┌────▼────┐                     ┌────▼────┐
                   │ cloud A │    │ cloud B │   …   (registered    │ cloud N │
                   │  k8s    │    │  k8s    │        clusters,      │  k8s    │
                   └─────────┘    └─────────┘        interchange.) └─────────┘
                   run image, emit Observation[] ──────────────────► back to control plane
```

## Boundary (unchanged, restated for this layer)

- Bench **orchestrates and measures**; it does not run the agent's logic (that's
  the workload image) and does not define the metric (that's Agent-Metrics).
- `metric_ref` stays reference-only. Observations carry **measured values**;
  `evaluation.rs` produces a generic `AttributeScore` against supplied
  thresholds — **no formula is computed here**.
- Protocols remain inputs and are **versioned**; a run is reproducible from
  `image@digest + inputs + protocol@ver + placement`.

## Status

| Built (tested) | This doc proposes (next) |
|---|---|
| stateless API, SurrealDB `any`-engine store + migrations, generic `AttributeScore` via `evaluation.rs`, multi-tenant, k8s deploy manifests, `metric_ref` to Agent-Metrics | `Workload` type + open `kind`; `Runtime` trait with `KubeRuntime`; `orchestrator.rs` reconcile + failover; cluster registry (multi-cloud); `PlacementSpec`/`budget`; control-plane HA on TiKV |

> Implementation lands incrementally behind the `server` feature, one concern
> per draft PR, with `cargo test` / `cargo test --features server` green and this
> doc + `DESIGN.md` kept in sync (per the proposal mechanism in `DESIGN.md`).
