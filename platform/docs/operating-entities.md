# Entities Operate Everything

The actors that run this system are **entities — autonomous agents — not only
humans**. Entities submit agents, trigger runs, read scores, reconcile desired
state, open PRs, approve gates, and govern. The platform is designed so an entity
can do **everything** a human can, through the same interfaces.

This is the throughline behind the rest of the architecture — every prior choice
exists to make the system *agent-operable*:

| Design choice | Why it enables entities |
|---|---|
| **Identity-first** (W3C DID / SPIFFE) | Every entity is a first-class principal with its own verifiable identity — no shared keys. |
| **Declarative + self-describing** (Terraform / SurrealQL grammar / CRDs) | An entity can *introspect and reason* over the typed, linked grammar (schema.org/JSON-LD style) instead of hard-coding endpoints. |
| **Per-action authorization** (AuthZEN, no standing privilege) | An entity is authorized at each action; "an agent does everything" never means "an agent is trusted with everything." |
| **Registry as bearer of trust** | Entities verify and produce signed, content-addressed artifacts + attestations — machine-checkable, not human-mediated. |
| **Reconciliation loop** | Entities operate by *declaring desired state*; the loop converges — no imperative human-in-the-loop required. |
| **Enforcement owned by the runtime** | Since entities (not just humans) act, enforcement *must* live below the actor — in SurrealDB/OPA/Cilium/guards — never in the caller. |

## The principle, stated

> Everything is built so an autonomous entity can operate it end to end —
> *and* so the runtime contains it. Capability and containment scale together:
> entities can do everything, **and** every action is identified, authorized,
> enforced, and audited.

## Federated, decentralized, permissionless

These entities don't live under one owner — the system is a **federation of
decentralized domains**, and it is **permissionless**: *we cannot stop anyone
from providing a service.*

- **Federated** — each domain (org, tenant, provider) runs its own stores,
  registry, and control plane, and they **interoperate** through a shared
  declarative grammar and standards ([Agent-Standard], DIDs, AuthZEN). No central
  hub everyone must route through.
- **Decentralized** — identity is **DID-based**, not issued by one authority;
  trust derives from **verifiable, signed attestations** (registry + provenance),
  not from a central database row someone controls.
- **Permissionless** — anyone may stand up a service: a benchmark provider, an
  evaluator, an agent. You **cannot prevent provision** — and you don't try to.

### Consequence: trust is consumed, not granted

Because provision can't be blocked, **enforcement moves to the point of
consumption**:

- A provider publishes a service + signed attestations. Nobody gatekeeps that.
- A *consumer* (or a gate) decides whom to trust by **verifying attestations**
  and weighing **reputation** (Agent-Trust) — fail-closed on *their* side.
- Agent-Bench's role is to make those attestations **meaningful and
  verifiable**: a benchmark result is a signed, registry-anchored claim that any
  party can independently check before consuming a service.

So "gates" are not a central authority blocking providers; they are **local,
verifiable trust decisions** over decentralized attestations. Capability is open;
trust is verifiable. That is how an open federation stays safe without a
gatekeeper.

## What this means for Agent-Bench

- The API, the declarative grammar, the registry, and the reconcile loop are all
  **agent-operable surfaces**, not human-only UIs.
- An operator-entity can: register an agent, declare an `EvaluationPolicy`, let
  the loop run the benchmark, read the registry-anchored result attestation, and
  promote — with each step identified (DID), authorized per-action, enforced by
  the runtime, and audited (change feed / Merkle log).
- The agents *under test* and the entities *operating* the platform are governed
  by the **same** identity + enforcement plane — one model, no privileged side
  door for automation.
