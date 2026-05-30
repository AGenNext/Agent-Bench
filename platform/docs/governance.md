# Agent Governance & Authorization

How the platform governs *who can do what* and *what agents are allowed to do* —
adopting two purpose-built tools instead of building our own:

- **[Microsoft Agent Governance Toolkit (AGT)](https://github.com/microsoft/agent-governance-toolkit)**
  — deterministic, runtime-owned policy enforcement for autonomous agents.
- **[OpenFGA](https://openfga.dev/)** (CNCF) — relationship-based fine-grained
  authorization (Zanzibar-style ReBAC) for the platform's own multi-tenant access.

Both reinforce our two principles: **adopt over build**, and **enforcement is
owned by the runtime, never the application / never the prompt**.

---

## Why AGT (and why it matches our design)

AGT's core thesis is the same one we've been building toward:

> *"Prompt-level safety is not a control surface."* Prompt injection succeeds at
> near-perfect rates, so policy must be enforced by **deterministic
> application-level middleware**, making violations *structurally impossible*
> rather than merely unlikely — **fail-closed**.

This is exactly our "enforcement owned by the runtime" stance, so we adopt AGT as
the enforcement middleware rather than writing our own. It ships a **Rust**
implementation (fits our stack) and integrates with agent frameworks (LangGraph,
AutoGen, Semantic Kernel, OpenAI Agents SDK, Claude Code).

### What we use from AGT

| AGT capability | Role on our platform |
|---|---|
| **Policy engine** (YAML allow/deny, `PolicyEvaluator`, fail-closed) | Enforces what a submitted agent may do during an evaluation run. The app declares policy; AGT enforces. |
| **Privilege rings** (4 sandboxed levels) | Layered with Kata/WasmEdge — agents run at the least privilege ring for the benchmark domain. |
| **Identity & trust** (SPIFFE, DIDs, mTLS, delegation chains) | Same SPIFFE identity plane as our CNCF stack; differentiates agents sharing credentials. |
| **Merkle tamper-evident audit log** | Immutable decision records per run — the evidence trail behind our benchmark result packages. |
| **MCP Security Gateway** (tool-poisoning detection) | Guards tool calls when evaluating tool-use agents. |
| **Red-team scanning** (prompt injection) | Generates OWASP-LLM01 adversarial cases that feed our Assurance (PAS) benchmark suite. |
| **Kill switch / termination** | Runtime stops a misbehaving agent mid-run; we record the event. |
| **Compliance mapping** (OWASP Agentic Top 10, NIST AI RMF, EU AI Act, SOC 2) | Lets us report evaluations against recognized governance frameworks. |

AGT's audit + policy decisions become **observed Assurance signals**: PAS is
computed from AGT denials/kills, not from in-app heuristics.

---

## Why OpenFGA (platform authorization)

AGT governs *the agents*; **OpenFGA governs the platform's human/service users** —
which enterprise member may submit agents, view runs, or read another team's
leaderboard. ReBAC fits our object graph (enterprise → team → agent → run).

### Authorization model (sketch)

```fga
model
  schema 1.1

type user

type enterprise            # = tenant / SurrealDB namespace
  relations
    define admin: [user]
    define member: [user]

type benchmark
  relations
    define owner: [enterprise]
    define viewer: [user, enterprise#member]

type agent
  relations
    define owner_org: [enterprise]
    define submitter: [user]
    define can_submit_run: submitter or admin from owner_org
    define can_view: [user] or member from owner_org

type run
  relations
    define agent: [agent]
    define can_view: can_view from agent
```

The API calls OpenFGA `Check(user, action, object)` **before** every mutation and
read, **fail-closed**. Tenant isolation is thus enforced at three layers:
SurrealDB namespace (data), Cilium/vCluster (network/runtime), and OpenFGA
(authorization) — defense in depth, none of it trusting application logic alone.

---

## Where this sits in the request path

```
request ─► mTLS (SPIFFE) ─► OpenFGA Check (authz, fail-closed)
        ─► API handler (declares intent)
        ─► eval run inside AGT policy + Kata sandbox (enforcement)
        ─► AGT Merkle audit + Falco/Cilium runtime guards
        ─► PAS / Assurance signals observed from AGT/runtime verdicts
```

## Adopt vs. build (updated)

| We build | We adopt |
|---|---|
| Scoring engine, benchmark contract, leaderboard API | **AGT** (agent policy, audit, identity, red-team), **OpenFGA** (authz), plus the CNCF stack (sandbox, CNI, observability, cost) |

We do **not** build a policy engine, an authorization service, or an audit log —
AGT and OpenFGA own those, and they own *enforcement*.
