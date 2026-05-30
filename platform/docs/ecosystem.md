# AGenNext Ecosystem & Repo Boundaries

Agent-Bench is one repo in a larger system. **Agent-Bench owns benchmarking and
performance evaluation only.** Identity, auth, and governance have dedicated
repos and must not be reimplemented here.

| Repo | Owns | This platform's relationship |
|---|---|---|
| **Agent-Bench** (here) | Reproducible benchmark suites, eval runs, scoring engine (CLEAR / rank fidelity / progress rate), leaderboards, result packages | — |
| **[Agent-Auth](https://github.com/AGenNext/Agent-Auth)** | Authentication & authorization plane — identity provider (casdoor), tokens, OpenFGA fine-grained authz | Platform validates tokens & delegates authz decisions here |
| **[Agent-IGA](https://github.com/AGenNext/Agent-IGA)** | Identity Governance & Administration — agent/user lifecycle, entitlements, access reviews, zero-trust policy | Platform consumes governance verdicts; enforcement owned upstream |
| **[Agent-Access](https://github.com/AGenNext/Agent-Access)** | Access management — access requests, grants, entitlement enforcement for agents & users | Platform's resource access is brokered here; it issues no grants itself |
| **[Agent-PAM](https://github.com/AGenNext/Agent-PAM)** | Privileged Access Management — agent privilege rings, just-in-time elevation, session brokering, kill-switch | Agents-under-test run under PAM-granted privilege; platform records what PAM allowed/killed |
| **[Agent-Compliance](https://github.com/AGenNext/Agent-Compliance)** | Compliance frameworks — SOC 2, EU AI Act, NIST AI RMF, OWASP Agentic mappings, evidence & attestation | Benchmark result packages feed compliance evidence; framework mappings owned here |
| **[Agent-Guard](https://github.com/AGenNext/Agent-Guard)** | Guardrails for every agent — input/output filtering, policy at inference time | **Enforces** agent behavior at runtime; Agent-Bench measures how agents fare against guardrail-style threats |
| **[Agent-Cognitive-Guard](https://github.com/AGenNext/Agent-Cognitive-Guard)** | Cognitive-layer safety — reasoning/intent monitoring, jailbreak & prompt-injection detection | Detects/blocks at the cognitive layer; its signals feed the Assurance (PAS) metric |
| **[Agent-Security](https://github.com/AGenNext/Agent-Security)** | Security plane — threat detection, posture, incident response across the fleet | Provides the security signals/threat model; Agent-Bench measures agents against them |
| **[Agent-Threat](https://github.com/AGenNext/Agent-Threat)** | Threat intelligence & modeling — attack taxonomies, adversarial test corpora, TTPs | Supplies the adversarial scenarios that become Assurance (PAS) benchmark cases |
| **[Agent-Health](https://github.com/AGenNext/Agent-Health)** | Agent health & SRE — liveness, drift, reliability monitoring of deployed agents | Live health metrics complement benchmark Reliability (pass@k); shared telemetry |
| **[Agent-SLA](https://github.com/AGenNext/Agent-SLA)** | SLA/SLO management — latency/availability targets, error budgets, breach alerting | Defines the SLA thresholds Agent-Bench scores against (CLEAR Latency / SCR) |
| **[Agent-Risk](https://github.com/AGenNext/Agent-Risk)** | Risk management — risk scoring, assessment, registers across agents | Consumes benchmark scores + Assurance as risk inputs |
| **[Agent-Trust](https://github.com/AGenNext/Agent-Trust)** | Trust scoring — reputation, attestation, trust-chain across agents | Folds benchmark rankings + Assurance into trust scores; we feed it, it doesn't gate eval |
| **[Agent-LCM](https://github.com/AGenNext/Agent-LCM)** | Agent Lifecycle Management — versioning, promotion, rollout, retirement | **Consumes Agent-Bench rankings**: eval results gate promotion/retirement |
| **[Agent-Memory](https://github.com/AGenNext/Agent-Memory)** | Agent memory layer (recall, conflict, decay) | Benchmarked by the AMB-001 suite |
| **Agent-Eval** | Reusable scoring functions, rubrics, CLEAR components | Agent-Bench calls these for scoring |

> **[Agent-Projects](https://github.com/AGenNext/Agent-Projects)** — portfolio /
> coordination across the AGenNext suite (project & task tracking). Not a runtime
> dependency of Agent-Bench; listed for completeness.

## Boundary rule

> Enforcement is owned by the runtime and by the auth/IGA plane — **never** by
> Agent-Bench. This platform *declares intent*, *validates tokens*, and *reads
> decisions*. It does not run an IdP, a policy engine, or an authz service.

## Docs that belong elsewhere (to migrate)

These were drafted here while scoping the platform but conceptually belong to
**Agent-Auth / Agent-IGA**. They are kept for now and should move when we split:

- `governance.md` — AGT + OpenFGA → **Agent-Auth**
- `zero-trust.md` — AuthZEN PEP/PDP, zero-trust posture → **Agent-Auth / Agent-IGA**
- `security-owasp.md` — Part 1 (platform defense) stays; Part 2 (agent threat
  battery) feeds the **Assurance** benchmark category and stays in Agent-Bench
- `cncf-stack.md` — infra runtime; the auth/identity rows reference Agent-Auth

## What stays in Agent-Bench, always

- The **scoring engine** (`src/metrics`, `src/scoring`) — the heart of the repo.
- Benchmark **contracts**, **suites**, and **result packages**.
- The **leaderboard** and **improvement-area** surfacing.
- The thin **token-validation + AuthZEN PEP adapter** that talks to Agent-Auth.
