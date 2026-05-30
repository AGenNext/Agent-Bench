# CNCF Cloud-Native Stack

The tools we'll use to run the agent-evaluation platform as a cloud-native,
multi-tenant service. Selected from the [CNCF Landscape](https://landscape.cncf.io/)
and mapped to *this platform's* specific needs — not a generic checklist.

Three properties drive the choices:
1. We execute **untrusted enterprise agent code** → sandboxing & runtime security.
2. We are **multi-tenant** → hard isolation between enterprises.
3. **Cost, policy, reliability** are first-class product metrics (CLEAR) → the
   infra must measure them too.

Maturity key: 🟢 graduated · 🟡 incubating · 🔵 sandbox.

## Guiding principles

**1. Adopt tools, don't build them.** Every box below is an off-the-shelf CNCF
project. We write application code only for what is genuinely ours — the
scoring engine and the API. Isolation, scheduling, autoscaling, policy,
identity, networking, observability, and cost accounting are all delegated to
mature projects. Building any of these ourselves is slower and worse.

**2. Enforcement is owned by the runtime — never the application.**
The Rust service *declares intent* and *reads signals*; it does not enforce.
Concretely:

| Concern | Who enforces (runtime) | What the app does |
|---|---|---|
| Tenant isolation | Kata microVM + Cilium NetworkPolicy + vCluster | tags workloads with `tenant`; never trusts in-process checks |
| Resource limits | Kubernetes limits/quotas + KEDA | requests a budget; the kubelet enforces it |
| Egress / data exfil | Cilium L7 policy + Falco | declares allowed egress; the CNI blocks the rest |
| Policy / compliance | OPA Gatekeeper (admission) + Falco (runtime) | emits the desired policy; the runtime admits/kills |
| Identity / authn | SPIFFE/SPIRE SVIDs + mesh mTLS | presents an SVID; the mesh authorizes |
| Fine-grained authz | OpenFGA `Check` (fail-closed) | asks "may user X do Y on Z?"; never decides itself |
| Agent policy / audit | Microsoft AGT middleware | declares policy; AGT enforces + audits |
| Sandbox escape | Kata / gVisor / WasmEdge | runs the agent; the sandbox contains it |

This means the platform's **Assurance (PAS)** signals are *observed* from
Gatekeeper denials and Falco events — the app reports what the runtime caught,
rather than trying to police agents itself. An agent that violates policy is
stopped by the runtime; we record the violation as a CLEAR signal.

## Core platform

| Need | Project | Why |
|---|---|---|
| Orchestration | **Kubernetes** 🟢 | Control plane for the whole platform. |
| Container runtime | **containerd** 🟢 | Standard runtime under k8s. |
| Sandbox untrusted agents | **Kata Containers** + **WasmEdge** 🔵 | Agents are arbitrary code — run each eval in a microVM (Kata) or as Wasm (WasmEdge). Hard isolation, not just namespaces. |
| Packaging / deploy | **Helm** 🟢 | Chart the API, workers, SurrealDB. |
| GitOps delivery | **Argo CD** 🟢 / **Flux** 🟢 | Declarative deploys from this repo. |
| Image registry | **Harbor** 🟢 | Signed, scanned images. |

## Evaluation execution (the compute-heavy part)

| Need | Project | Why |
|---|---|---|
| Benchmark runs as DAGs | **Argo Workflows** 🟡 | Each run = ingest → rollout per task → score → package. Natural fit for a benchmark pipeline. |
| Event-driven triggers | **Argo Events** 🟡 | Kick a run when an agent is submitted. |
| Worker autoscaling | **KEDA** 🟢 | Scale eval workers by queue depth (runs are bursty); scale to zero between. |
| Async job / telemetry bus | **NATS** 🟡 | Lightweight, great Rust support; carries run requests + per-step telemetry. |
| Serverless eval endpoints | **Knative** 🔵 | Scale-to-zero for on-demand agent endpoints. |

## Multi-tenancy & security

| Need | Project | Why |
|---|---|---|
| Tenant isolation | **vCluster** 🔵 / **Capsule** 🔵 | Virtual clusters / namespace tenancy per enterprise — mirrors our SurrealDB namespace-per-tenant model at the infra layer. |
| Workload identity | **SPIFFE/SPIRE** 🟢 | Zero-trust identity per tenant workload; no shared secrets across tenants. |
| Fine-grained authz | **OpenFGA** 🔵 | Zanzibar-style ReBAC: who may submit/view which agents, runs, leaderboards. Called fail-closed before every op. See `governance.md`. |
| Agent governance | **Microsoft AGT** (not CNCF) | Deterministic, fail-closed policy + Merkle audit + privilege rings for the *agents under test*. Rust impl. See `governance.md`. |
| Policy enforcement | **OPA/Gatekeeper** 🟢 + **Kyverno** 🟡 | **Runtime owns enforcement** at admission. The app emits intended policy; Gatekeeper admits or denies. PAS is observed from denials, not computed in-app. |
| Runtime security | **Falco** 🟢 | Enforces at runtime: detects/kills escapes & policy abuse from untrusted agents. Falco events feed the Assurance signal. |
| TLS / certs | **cert-manager** 🟢 | Automated mTLS for API + mesh. |
| Secrets | **External Secrets Operator** 🟡 | Per-tenant credential injection. |

## Networking

| Need | Project | Why |
|---|---|---|
| CNI + network policy | **Cilium** 🟢 | eBPF networking; per-tenant network isolation policies. |
| Service mesh | **Linkerd** 🟢 | mTLS + golden metrics between services (lighter than Istio). |
| Ingress / API gateway | **Envoy** 🟢 + **Gateway API** | L7 routing, rate-limiting per tenant. |

## Observability (also feeds CLEAR metrics)

| Need | Project | Why |
|---|---|---|
| Tracing | **OpenTelemetry** 🟢 | Already emitting via `tracing`; export spans for latency (the **L** in CLEAR). |
| Trace backend | **Jaeger** 🟢 | Store/query agent run traces. |
| Metrics | **Prometheus** 🟢 | Run counts, latencies, pass rates. |
| Long-term metrics | **Thanos** 🟡 / **Cortex** 🟡 | Historical leaderboard / drift trends. |
| **Telemetry / measurement store** | **ClickHouse** | High-volume per-step run telemetry, time-series, leaderboard analytics, drift. The *measurement* plane (SurrealDB stays transactional + enforcement). See `telemetry.md`. |
| Logs | **Fluent Bit** 🟢 | Per-tenant log shipping. |
| **Cost** | **OpenCost** 🟡 | Measures per-run \$ cost — directly populates the **C** (Cost) dimension of CLEAR / CNA / CPS. |

## Storage

| Need | Project | Why |
|---|---|---|
| Persistent volumes (SurrealKV) | **Longhorn** 🟢 / **Rook-Ceph** 🟢 | Durable storage for the embedded SurrealDB data dir. |
| Result-package artifacts | S3-compatible (e.g. MinIO) | Store reproducibility manifests + traces per the benchmark contract. |

## Standout picks for *this* platform

- **Kata Containers / WasmEdge** — non-negotiable: we run other people's agents,
  and the **sandbox** (not the app) is what contains them.
- **Argo Workflows + KEDA** — the eval pipeline and its bursty autoscaling, both
  off-the-shelf so we ship faster.
- **OpenCost** — turns infra cost into a product metric (CLEAR Cost / CNA / CPS)
  without us instrumenting billing ourselves.
- **OPA Gatekeeper + Falco** — **own all policy/safety enforcement**; the app
  only consumes their verdicts as Assurance (PAS) signals.
- **vCluster / SPIFFE + Cilium** — tenant isolation enforced at the runtime,
  mirroring our namespace-per-tenant DB model.

## Build vs. adopt

| We build (ours) | We adopt (CNCF) |
|---|---|
| Scoring engine (CLEAR, rank fidelity, progress rate) | Everything else: orchestration, sandboxing, autoscaling, policy, identity, networking, observability, cost |
| Benchmark contract & result schema | Argo Workflows runs the pipeline that produces them |
| Multi-tenant API + leaderboard logic | vCluster/SPIFFE/Cilium enforce the isolation the API assumes |

If a need maps to a mature CNCF project, we use it — custom infra is the
exception, gated on "no graduated/incubating project fits."

> Note: SurrealDB itself is not a CNCF project; it runs embedded in our Rust
> service and is backed by CNCF storage (Longhorn/Rook) for persistence.
