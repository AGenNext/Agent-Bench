# Reconciliation Model

Agent-Bench follows the **Kubernetes reconciliation pattern**: evaluation is
*declarative*. You declare the **desired** evaluation state; a controller
continuously **reconciles** actual → desired by scheduling runs. No imperative
"run this now" — the loop converges.

> Kubernetes reconciles. So does Agent-Bench: declare *which agents must have a
> current score on which (benchmark, hardware)*, and the controller makes it so.

## Desired state (a CRD)

```yaml
apiVersion: bench.agentfield.io/v1
kind: EvaluationPolicy
metadata: { name: prod-agents, namespace: acme }
spec:
  selector: { scaffold: plan-execute }        # which agents
  benchmarks: [swe-lite, kernelbench]
  hardware: [gpu-a100, npu]                    # score required per backend
  freshness: 24h                               # a score older than this is stale
  onDrift: rebenchmark                         # Agent-Drift signal → full re-run
```

## Reconcile loop

```
observe:  for each (agent, benchmark, hardware) in desired set:
             current = latest scored run in SurrealDB
diff:      missing OR older than spec.freshness OR drift flagged?
act:       enqueue a run  (Argo Workflows / KEDA-scaled workers)
           → agent runs in a Kata sandbox → scored → written to SurrealDB
           → telemetry to ClickHouse
repeat:    re-observe; converge when every required score is fresh
```

This is level-triggered, not edge-triggered: a missed event self-heals on the
next pass, and the system is **idempotent** — re-running reconcile is safe.

## Who does what (adopted)

| Concern | Component |
|---|---|
| Desired-state API (CRD) | Kubernetes + a lightweight controller (our thin glue) |
| Run scheduling / pipeline | **Argo Workflows / Argo Events** |
| Worker autoscaling | **KEDA** (scale by queue depth, scale to zero) |
| Sandbox execution | **Kata / WasmEdge** |
| Authoritative scores | **SurrealDB** (transactional + enforced) |
| Telemetry | **ClickHouse** |
| Drift trigger | **Agent-Drift** (ρ < 0.75 → `onDrift: rebenchmark`) |
| Lifecycle gates | **Agent-LCM** (promotion blocked until fresh passing scores) |

## Why it matters for benchmarking

- **Freshness without manual runs**: new hardware or a new scaffold version is
  declared once; the loop fills in the missing scores.
- **Drift-aware**: a drift signal marks scores stale → reconcile re-benchmarks →
  the *Efficient Benchmarking* reselection trigger (ρ < 0.75) becomes a control
  loop, not a cron job.
- **Cost-bounded**: combined with mid-range task selection, reconcile runs only
  the tasks needed to refresh a ranking, not the full suite.

## GitOps — checks at all gates

Everything is **declarative in git** and synced by **Argo CD / Flux**; the
`EvaluationPolicy` is version-controlled, reviewed, and rolled back like any
manifest. Crucially, **Agent-Bench scores are required checks at every gate** —
an agent advances only if it passes, enforced (not advisory) at each stage:

| Gate | Check (fail-closed) |
|---|---|
| **PR / merge** | A change to an agent triggers a reduced-suite eval; the PR check fails if ranking/score regresses or Assurance (PAS) drops. |
| **CI pipeline** | Argo Workflows runs the benchmark stage; red = pipeline stops. |
| **Admission** | OPA Gatekeeper reads the agent's latest SurrealDB score; deploys are denied without a fresh passing result for the target hardware. |
| **Promotion (LCM)** | Agent-LCM blocks promotion to prod until required `(benchmark, hardware)` scores are fresh and above threshold. |
| **Drift** | Agent-Drift demotes/flags in git; reconcile re-benchmarks before re-promotion. |

Because the desired state, the policy thresholds, and the gate definitions all
live in git, **the benchmark is the gate** — provenance and approvals flow
through normal GitOps review, and enforcement happens at the runtime
(Gatekeeper/LCM), never as an advisory comment.

## Status

Roadmap: the scoring + storage core exists today (synchronous `submit_run`). The
`EvaluationPolicy` CRD + reconcile controller + GitOps gate wiring are the
operator layer on top.
