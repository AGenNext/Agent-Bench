# The Language Is Sacrosanct & Interoperable

In a permissionless, decentralized federation you cannot control who provides a
service — so the **one thing that must never fork is the shared language and its
meaning.** It is the federation's sole invariant. Everything else (who runs what,
where data lives, which engine) can vary; the **grammar and its semantics cannot**.

> If a benchmark result means one thing here and another there, attestations are
> worthless and interoperability collapses. The language — and its *meaning* —
> is therefore **sacrosanct** (stable, canonical, never silently changed) and
> **interoperable** (identical across every domain).

## Two properties, held together

- **Sacrosanct** — the meaning is canonical and versioned. A term's definition is
  immutable within a version; changing meaning means a *new version*, never an
  in-place redefinition. No domain may quietly reinterpret a term.
- **Interoperable** — the same definition is consumed identically everywhere,
  because it is **semantically grounded** (schema.org / JSON-LD-style typed,
  linked vocabulary) and carried by the declarative grammar (SurrealQL `DEFINE`)
  + standards ([Agent-Standard]).

## Where the meaning lives

| Layer | Holds | Role |
|---|---|---|
| **SurrealQL grammar** (`DEFINE`) | typed entities, fields, relationships, permissions | the *grammar* — what terms exist and their shapes |
| **Reference library** (`benchmarks/reference/`) | precise glossary + **formulas** per metric | the *semantics* — what each term/metric **means** and how it's computed |
| **Benchmark contract** (`contracts/`) | result-package schema | the *structure* of a verifiable claim |
| **Agent-Standard** | cross-domain specs | federation-wide interoperability |

The reference docs are not just notes: they are the **canonical definitions**.
"Speedup = baseline/kernel latency (geomean over correct tasks)", "PAS = 1 −
violations/critical-actions", "progress rate = max matching score so far" — these
are the *meaning*, pinned with formulas so every entity computes and interprets
them the same way.

## Why this is the linchpin of trust

The federation's trust model (`operating-entities.md`) rests on **verifiable
attestations**. An attestation is only verifiable if its terms mean the same
thing to issuer and consumer:

```
provider signs:  "agent X: speedup_geomean=1.46 on kernelbench@gpu-a100"
consumer verifies:  signature ✓  AND  semantics of `speedup_geomean` ✓ (canonical, versioned)
```

Without a sacrosanct, interoperable language, the second check is impossible —
so **guarding the meaning is what makes decentralized trust work at all.**

## Rules

1. **Version meaning, never mutate it** — definitions are immutable per version;
   semantic changes bump the version and are explicit in attestations.
2. **Ground it semantically** — typed, linked (JSON-LD/schema.org-style), so
   machines and agents resolve meaning unambiguously.
3. **Pin formulas, not prose** — every metric carries its computation, so there
   is one interpretation, not many.
4. **Standardize across domains** — the vocabulary is shared via Agent-Standard;
   no domain-private redefinition of shared terms.

## Sacrosanct, but it evolves — governed evolution

Sacrosanct does **not** mean frozen. The language **evolves** — but in a governed,
versioned way: meaning is immutable *within* a version; advancement happens by
issuing a *new* version, never by mutating an existing term. Evolution is driven
by two explicit inputs:

1. **Current research & development best practices.** New metrics, methods, and
   construct-validity fixes from the field flow in through the **reference
   library** (`benchmarks/reference/`) — e.g. mid-range task selection, CLEAR,
   progress rate, speedup/`fast_p`, reward-hacking-resistant variants. As research
   advances, the canonical definitions advance with it (new version), so the
   language reflects the state of the art rather than ossifying.
2. **Government & regulatory enforcement models.** Compliance frameworks — **EU
   AI Act, NIST AI RMF, OWASP Agentic, SOC 2** — shape required terms and
   thresholds. These enter via **Agent-Compliance**, so the language stays aligned
   with what regulators enforce, and benchmark attestations double as compliance
   evidence.

```
research (reference library) ─┐
                              ├─► proposed semantic change ─► new VERSION ─► attestations cite it
gov frameworks (Agent-Compliance) ┘        (reviewed, GitOps)      (old meaning preserved)
```

### Evolution rules

- **Additive & versioned** — add or refine via a new version; never silently
  redefine. Old attestations remain valid under their stated version.
- **Evidence-driven** — a change cites its source (a paper, a regulation), so the
  *why* is part of the record.
- **Reviewed like code** — semantic changes go through GitOps review (the grammar
  is declarative; see `data-architecture.md`), with both R&D and compliance
  stakeholders.
- **Backward-resolvable** — every attestation carries its language version, so any
  consumer can resolve the exact meaning that applied when it was issued.

This keeps the two demands in balance: **interoperability needs stability;
relevance needs change.** Versioned, governed evolution gives both — the meaning
is sacrosanct within a version and advances responsibly across versions, steered
by research and by the enforcement models that regulators actually use.

## What this means for Agent-Bench

Agent-Bench is a **steward of the evaluation language**: its glossary + formulas
+ result-package schema are part of the federation's canonical semantics. Keeping
them sacrosanct and interoperable is as important as the scores themselves — the
scores are only meaningful because the language is.
