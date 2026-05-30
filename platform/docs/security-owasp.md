# Security — OWASP Mapping

[OWASP Top Ten](https://owasp.org/www-project-top-ten/) applies to this platform
in **two roles**:

- **Defend** — securing the platform service itself (it's a multi-tenant web API
  running untrusted code). → OWASP Top 10 (Web, 2021).
- **Evaluate** — the threats we test submitted agents *against*; these become
  Assurance (PAS) test cases in benchmarks. → OWASP Top 10 for **LLM/Agentic
  Applications**.

> **Repo boundary:** formal compliance-framework mappings (SOC 2, EU AI Act,
> NIST AI RMF, OWASP Agentic) and attestation evidence are owned by
> **[Agent-Compliance](https://github.com/AGenNext/Agent-Compliance)**. Here we
> only define the **evaluation** side — turning OWASP-LLM threats into scored
> Assurance test cases whose result packages become compliance evidence upstream.

Consistent with our principle that **enforcement is owned by the runtime**, each
control names the CNCF/runtime component that enforces it — the app declares
intent and observes verdicts.

---

## Part 1 — Defending the platform (OWASP Web Top 10, 2021)

| # | Risk | Control (runtime-owned) |
|---|---|---|
| A01 | Broken Access Control | Tenant isolation by SurrealDB **namespace** + Cilium NetworkPolicy + vCluster + SPIFFE SVIDs. The API never trusts an in-process tenant check alone. |
| A02 | Cryptographic Failures | mTLS everywhere via **cert-manager** + Linkerd mesh; secrets via External Secrets, never in env/images. |
| A03 | Injection | **All SurrealQL uses parameter binding** (`.bind(...)`), never string interpolation of user input — see `src/db.rs`. Tenant header validated to `[A-Za-z0-9_-]`. |
| A04 | Insecure Design | Threat-modeled: untrusted agent code is assumed hostile and runs only inside a sandbox (Kata/WasmEdge). |
| A05 | Security Misconfiguration | **OPA Gatekeeper / Kyverno** admission policies; no privileged pods, read-only rootfs enforced at admission. |
| A06 | Vulnerable & Outdated Components | **Harbor** image scanning + Trivy; `cargo audit` in CI. |
| A07 | Identification & Auth Failures | **SPIFFE/SPIRE** workload identity; mesh mTLS authorizes service-to-service. |
| A08 | Software & Data Integrity Failures | **Sigstore/cosign** signed images + SLSA provenance; Argo CD verifies before deploy. |
| A09 | Logging & Monitoring Failures | **OpenTelemetry + Fluent Bit + Falco**; per-tenant audit trail of runs. |
| A10 | SSRF | **Critical here** — agents can be coerced into SSRF. **Cilium L7 egress policy** restricts outbound; the sandbox has no cloud-metadata access. Enforced by the CNI, not the app. |

---

## Part 2 — What we evaluate agents for (OWASP Top 10 for LLM Apps)

These map directly to the platform's **Assurance (A)** dimension and become
scored test cases in security-focused benchmark suites. PAS (Policy Adherence
Score) aggregates the agent's pass rate across them.

| # | LLM risk | How the benchmark tests it | Feeds |
|---|---|---|---|
| LLM01 | Prompt Injection | Adversarial-prompt test cases (the CLEAR reference's 500-case suite); does the agent obey injected instructions? | PAS, prompt-injection resistance |
| LLM02 | Sensitive Information Disclosure | Seed PII in context; check the agent doesn't leak it. | PAS, data-leak rate |
| LLM03 | Supply Chain | Test tool/plugin provenance the agent pulls in. | Assurance |
| LLM04 | Data & Model Poisoning | Out of scope at eval time; covered by platform integrity (A08). | — |
| LLM05 | Improper Output Handling | Does downstream-executed agent output get sanitized (no injection into our tools)? | Assurance |
| LLM06 | Excessive Agency | Does the agent take unauthorized actions / exceed granted scope? | PAS, policy violations |
| LLM07 | System Prompt Leakage | Probe for system-prompt extraction. | PAS |
| LLM08 | Vector/Embedding Weaknesses | For RAG agents: retrieval poisoning / cross-tenant leakage. | Assurance |
| LLM09 | Misinformation | Hallucination / grounding checks (AgentBoard grounding accuracy). | Efficacy, Assurance |
| LLM10 | Unbounded Consumption | Cost/latency runaway (Reflexion-style loops). Caught by k8s limits + measured as cost. | Cost, Reliability |

### Enforcement vs. evaluation
- **Enforcement is owned upstream.** Agent behavior is blocked by
  **[Agent-Guard](https://github.com/AGenNext/Agent-Guard)** (I/O guardrails) and
  **[Agent-Cognitive-Guard](https://github.com/AGenNext/Agent-Cognitive-Guard)**
  (reasoning/jailbreak detection); infra violations (egress, sandbox escape) by
  Cilium/Falco/Kata. Agent-Bench never enforces.
- **Evaluation is ours.** When measuring an agent's security posture, the
  benchmark presents the OWASP-LLM scenarios and scores the response into PAS —
  no enforcement, just observation. Guard/runtime verdicts are *inputs* to that
  score.

The same OWASP taxonomy thus secures the platform *and* defines a security
benchmark category, keeping defense and evaluation aligned.

---

## Implementation notes already in the codebase

- **A03 (Injection):** every query in `src/db.rs` binds parameters; tenant input
  is character-validated in `src/tenancy.rs`.
- **A01 (Access Control):** `tests/end_to_end.rs::tenants_are_isolated` asserts
  cross-tenant invisibility at the data layer.
- **LLM06/PAS:** `metrics/clear.rs` computes Policy Adherence Score from
  policy-critical actions and violations; `scoring.rs` flags `assurance` as an
  improvement area when PAS is low.
