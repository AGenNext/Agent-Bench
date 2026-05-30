# Zero-Trust Architecture

The platform treats **every actor — human, service, and agent-under-test — as
untrusted until verified, on every request**. This doc ties together:

- **[Zero-trust principles](https://www.ibm.com/think/topics/zero-trust)** (IBM) — the posture.
- **[Unified zero-trust for LLMs & AI agents](https://xage.com/unified-zero-trust-for-llms-and-ai-agents/)** (Xage) — agents as first-class identities.
- **[OpenID AuthZEN](https://openid.net/wg/authzen/)** — the *standard* interface
  between enforcement points (PEPs) and decision points (PDPs), so we aren't
  locked to one policy engine.

It builds on `governance.md` (AGT + OpenFGA) and `cncf-stack.md`.

---

## Zero-trust tenets → our enforcement

| Tenet (IBM) | How the platform applies it | Owned by (runtime) |
|---|---|---|
| **Verify explicitly** | Every request carries a verifiable identity; nothing is trusted by network location. | SPIFFE/SPIRE SVIDs + mesh mTLS |
| **Least privilege** | Agents run at the lowest AGT privilege ring; users get only OpenFGA relations they're granted. | AGT rings + OpenFGA |
| **Assume breach** | Untrusted agent code is assumed hostile and confined; blast radius is one sandbox + one tenant namespace. | Kata/WasmEdge + Cilium + SurrealDB namespace |
| **Continuous verification** | Authorization is re-checked **per action**, not once per session — critical for multi-step agent runs. | AuthZEN PEP→PDP on each step |
| **Micro-segmentation** | Per-tenant network policy; no lateral movement between enterprises. | Cilium NetworkPolicy + vCluster |
| **Tamper-evident audit** | Every decision is recorded immutably. | AGT Merkle log |

---

## Agents are first-class identities (Xage model)

A submitted agent is not "code we run" — it's a **principal with its own
identity** that must authenticate and be authorized per action:

- **Identity:** each agent (and each step it takes) gets a SPIFFE SVID / DID via
  AGT — even agents sharing model credentials are differentiated.
- **No standing privilege:** an agent holds no ambient permissions; every tool
  call, egress, or data read is authorized at the moment of use.
- **Per-action authorization:** during a multi-step rollout, *each* action is an
  authorization decision — not a single up-front grant. This is where most
  agent breaches happen (excessive agency, OWASP LLM06).
- **Continuous trust scoring:** AGT delegation chains + trust scores degrade if
  an agent behaves anomalously mid-run; Falco events can revoke in-flight.

---

## AuthZEN: one standard interface, swappable engines

We decouple **enforcement (PEP)** from **decision (PDP)** using the OpenID
AuthZEN Authorization API. PEPs never decide; they ask a PDP and **fail closed**.

```
            ┌─────────── PEPs (enforce) ───────────┐
 request ─► │ API gateway · AGT middleware · sidecar│
            └───────────────┬───────────────────────┘
                            │  AuthZEN access-evaluation request
                            ▼
            ┌─────────── PDPs (decide) ─────────────┐
            │ OpenFGA (ReBAC)  · OPA  · AGT policy   │
            └───────────────────────────────────────┘
```

### AuthZEN evaluation request (shape)

```json
{
  "subject":  { "type": "agent", "id": "acme/strong@1",
                "properties": { "ring": 2, "trust": 0.9 } },
  "action":   { "name": "tool.invoke" },
  "resource": { "type": "tool", "id": "http_fetch" },
  "context":  { "tenant": "acme", "run": "run:abc", "step": 7 }
}
```

### Response

```json
{ "decision": false,
  "context": { "reason": "egress not permitted at ring 2 (OWASP LLM10/SSRF)" } }
```

Because the interface is standardized, we can:
- route **user/resource** authz to **OpenFGA**, **agent-action** authz to **AGT**,
  and **admission/config** policy to **OPA** — behind one PEP contract;
- swap or add a PDP without touching enforcement code;
- aggregate decisions (deny-overrides) across engines.

---

## Request lifecycle (end to end)

```
1. mTLS handshake          → SPIFFE verifies caller identity        (verify explicitly)
2. AuthZEN Check (PEP→PDP) → OpenFGA: may this user act on this run? (least privilege)
3. API handler             → declares intent only (no enforcement)
4. Eval run starts         → agent gets its own SVID, ring N         (agent identity)
5. Each agent action       → AuthZEN Check (PEP→AGT/OPA), fail-closed (continuous verify)
6. Sandbox + CNI           → Kata/Cilium contain + segment           (assume breach)
7. Every decision          → AGT Merkle audit                        (tamper-evident)
8. Verdicts                → feed CLEAR Assurance (PAS)               (evaluate)
```

## Adopt vs. build (final)

| We build | We adopt |
|---|---|
| Scoring engine, benchmark contract, leaderboard API, AuthZEN PEP glue | AuthZEN spec, OpenFGA/OPA/AGT PDPs, SPIFFE, Kata/WasmEdge, Cilium, Falco |

The only authorization code we own is the thin **PEP adapter** that speaks
AuthZEN — the decisions, identities, sandboxing, and audit are all adopted.
