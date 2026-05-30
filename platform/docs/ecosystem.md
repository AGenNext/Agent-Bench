# AGenNext / AgentField Ecosystem & Repo Boundaries

**[AgentField](https://agentfield.ai/)** is the umbrella: an open-source AI
*control plane* / backend — *"build, run and scale AI agents like APIs and
microservices: observable, auditable, and identity-aware from day one,"* where
**your infrastructure governs** (it sits in the stack like a database, not a
library) and every agent gets a **W3C DID**, not a shared key.

That philosophy *is* the one this platform encodes: **enforcement owned by the
infrastructure**, identity-first, audit built in. **Agent-Bench is AgentField's
benchmarking / evaluation plane** — it owns performance evaluation only.

AGenNext is a large suite (~169 repos). Everything outside benchmarking has a
dedicated home and must not be reimplemented here. This doc keeps detailed prose
for the repos Agent-Bench *actually exchanges data with*, and groups the rest by
plane.

## Direct integrations (data in/out of Agent-Bench)

These touch the scoring path directly.

| Repo | Owns | Relationship to Agent-Bench |
|---|---|---|
| **Agent-Eval** | Reusable scoring functions, rubrics, CLEAR components | Agent-Bench *calls* these to score runs |
| **[Agent-Frameworks](https://github.com/AGenNext/Agent-Frameworks)** | Agent frameworks / **scaffolds** (ReAct, Plan-Execute, …) | These are the *scaffolds under test* — the source of scaffold-driven distribution shift the benchmark must generalize across |
| **[Agent-Runs](https://github.com/AGenNext/Agent-Runs)** | Canonical run/execution records | Agent-Bench reads run records and writes scored results |
| **[Agent-Traces](https://github.com/AGenNext/Agent-Traces)** | Per-step execution traces/rollouts | Trajectory metrics + progress rate are scored over these; cited as result-package evidence |
| **[Agent-Memory](https://github.com/AGenNext/Agent-Memory)** | Agent memory layer (recall, conflict, decay) | Benchmarked by the AMB-001 suite |
| **[Agent-LCM](https://github.com/AGenNext/Agent-LCM)** | Lifecycle management — promotion, rollout, retirement | **Consumes rankings** to gate promotion/retirement |
| **[Agent-Drift](https://github.com/AGenNext/Agent-Drift)** | Behavioral/performance/data drift detection | **Triggers re-benchmarking** (paper's ρ<0.75 reselection) |
| **[Agent-SLA](https://github.com/AGenNext/Agent-SLA)** | SLA/SLO targets, error budgets | Defines thresholds for CLEAR Latency / SLA Compliance Rate |
| **[Agent-FinOps](https://github.com/AGenNext/Agent-FinOps)** + **[Agent-Wallet](https://github.com/AGenNext/Agent-Wallet)** | Cost management / billing / credits | Supply per-run \$ cost → CLEAR **Cost** (CNA, CPS) |
| **[Agent-Threat](https://github.com/AGenNext/Agent-Threat)** | Threat intel, adversarial corpora, TTPs | Source the OWASP-LLM scenarios that become Assurance (PAS) cases |
| **[Agent-Compliance](https://github.com/AGenNext/Agent-Compliance)** | SOC 2 / EU AI Act / NIST / OWASP mappings, evidence | **Consumes result packages** as compliance evidence |
| **[Agent-Risk](https://github.com/AGenNext/Agent-Risk)** / **[Agent-Trust](https://github.com/AGenNext/Agent-Trust)** | Risk scoring / trust scoring | Consume benchmark scores + Assurance as inputs (downstream; don't gate eval) |
| **[Agent-Analytics](https://github.com/AGenNext/Agent-Analytics)** | Cross-agent BI, trends | Consumes results/leaderboards for analysis |
| **[Agent-Sight](https://github.com/AGenNext/Agent-Sight)** | Observability — traces, metrics, dashboards | Receives Agent-Bench OTel telemetry; eval data surfaces here |
| **[Agent-Hooks](https://github.com/AGenNext/Agent-Hooks)** | Event bus / webhooks | Agent-Bench emits (run scored, leaderboard changed) & subscribes (new agent → eval) |

## Identity, access & governance plane (enforcement owned upstream)

Agent-Bench *declares intent, validates tokens, reads decisions* — it runs no
IdP, policy engine, or authz service.

| Repo | Owns |
|---|---|
| **[Agent-Auth](https://github.com/AGenNext/Agent-Auth)** | AuthN/Z — identity (casdoor), tokens, OpenFGA |
| **[Agent-Access](https://github.com/AGenNext/Agent-Access)** | Access requests, grants, entitlements |
| **[Agent-IGA](https://github.com/AGenNext/Agent-IGA)** | Identity governance & administration |
| **[Agent-PAM](https://github.com/AGenNext/Agent-PAM)** | Privileged access — rings, JIT elevation, kill-switch |
| **[Agent-Secrets](https://github.com/AGenNext/Agent-Secrets)** | Secrets management |

## Security & safety plane (runtime enforcement)

| Repo | Owns |
|---|---|
| **[Agent-Security](https://github.com/AGenNext/Agent-Security)** | Security plane — detection, posture, IR |
| **[Agent-Guard](https://github.com/AGenNext/Agent-Guard)** | I/O guardrails at inference |
| **[Agent-Cognitive-Guard](https://github.com/AGenNext/Agent-Cognitive-Guard)** | Cognitive-layer safety, jailbreak/injection detection |

## Ops, health & platform plane

| Repo | Owns |
|---|---|
| **[Agent-Health](https://github.com/AGenNext/Agent-Health)** | Liveness / reliability monitoring |
| **AgentKube** | Kubernetes operator / runtime for agents |
| **Agent-Flow** | Agent workflow / orchestration |
| **[Agent-Swarm](https://github.com/AGenNext/Agent-Swarm)** | Multi-agent swarm orchestration (also a benchmark target: `agent_coordination`) |
| **Agent-Handoff** | Agent-to-agent handoff / delegation (benchmark target: coordination) |
| **Agent-Features** | Feature store / flags |
| **Agent-Standard** | Interop standards & specs |
| **Agent-Projects** | Portfolio / coordination (project & task tracking) |

> The lists above cover the repos relevant to Agent-Bench; the org has many more
> (~169). New repos are slotted into the appropriate plane rather than given
> bespoke prose unless they exchange data with the scoring path.

## Boundary rule

> Enforcement is owned by the runtime and by the auth/IGA/guard plane — **never**
> by Agent-Bench. This platform *declares intent*, *validates tokens*, and *reads
> decisions*. It does not run an IdP, a policy engine, or an authz service.

## Docs that belong elsewhere (to migrate on split)

Drafted here while scoping the platform; conceptually belong upstream:

- `governance.md` — AGT + OpenFGA → **Agent-Auth**
- `zero-trust.md` — AuthZEN PEP/PDP, zero-trust → **Agent-Auth / Agent-IGA**
- `security-owasp.md` Part 1 (platform defense) stays; Part 2 (agent threat
  battery) feeds the **Assurance** benchmark category and stays here
- `surrealdb-security.md` — data-layer enforcement; identity from **Agent-Auth**
- `cncf-stack.md` — infra runtime; auth/identity rows reference the planes above

## What stays in Agent-Bench, always

- The **scoring engine** (`src/metrics`, `src/scoring`) — the heart of the repo.
- Benchmark **contracts**, **suites**, and **result packages**.
- The **leaderboard** and **improvement-area** surfacing.
- The thin **token-validation + AuthZEN PEP adapter** that talks to Agent-Auth.
