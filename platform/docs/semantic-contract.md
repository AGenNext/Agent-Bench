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

## Versioned protocols become the enforcement

In a permissionless federation there is no central authority to *grant* or
*deny* — so **enforcement is realized as conformance to a versioned protocol.**
The protocol *is* the enforcement: a thing that doesn't conform simply fails
verification, everywhere, without anyone having to block it.

```
protocol@v = grammar + semantics (formulas) + schema + conformance suite
artifact/attestation/agent  declares  protocol@v
consumer/gate  verifies conformance to protocol@v   →  pass = trusted, fail = rejected
```

- **The version is the contract.** Each protocol version pins exact meaning +
  a **machine-checkable conformance suite** (cf. AGT's RFCs + conformance tests).
  Declaring `protocol@v` is a binding, verifiable commitment.
- **Enforcement is distributed.** Every consumer enforces locally by checking
  conformance — no central PEP required. The same check yields the same verdict
  anywhere, because the protocol version is canonical and interoperable.
- **Non-conformance self-rejects.** You can't be stopped from *providing* a
  service, but a service/attestation that doesn't conform to the declared
  protocol version won't *verify* — so it isn't consumed. Capability stays open;
  conformance is the gate.
- **Evolution stays safe.** New behavior ships as a new protocol version with its
  own conformance suite; old versions keep verifying under their own rules
  (backward-resolvable). Upgrading enforcement = adopting a new protocol version.

This is the bridge from "meaning is sacrosanct" to "the runtime enforces":
**the versioned protocol carries the meaning *and* the enforcement** — define the
grammar, semantics, schema, and conformance tests once per version, and
verification against that version becomes the enforcement, executed by whoever is
consuming.

### Protocols carry security, governance — and excellence

A protocol version is not only a *semantic* contract. It bundles
conformance-checkable requirements across **three dimensions**, all enforced the
same way (verify conformance to the declared version):

| Dimension | What the protocol pins | Owned by | Conformance check |
|---|---|---|---|
| **Security** | threat resistance, prompt-injection/jailbreak bars, safe-output rules | Agent-Threat / Agent-Guard | adversarial conformance suite (OWASP-LLM) |
| **Governance** | policy, access, identity, compliance (EU AI Act, NIST, SOC 2) | Agent-Auth / IGA / Compliance | policy + attestation conformance |
| **Excellence** | quality & performance bars — pass thresholds, CLEAR mins, speedup floors | **Agent-Bench** | benchmark conformance (the scoring suite) |

So "the protocol enforces" means all three at once: an artifact that declares
`protocol@v` is verifiably **secure, governed, *and* excellent** — or it fails
conformance. **Agent-Bench is the steward of the *excellence* protocols**: the
benchmark thresholds, metric definitions, and result-package schema that define
what "good enough" means, machine-checkably, federation-wide. Security and
governance protocols come from their planes; excellence is ours — and they
compose into one versioned protocol whose conformance is the gate.

#### Protocols are best practices, not just restrictions

Crucially, protocols are **not merely restrictive** — they are **codified best
practices**. They are *constructive*: they tell you how to do the right thing,
not just what's forbidden. Two faces of the same protocol:

- **Enabling (the default reading).** A protocol is the distilled, research- and
  theory-grounded best practice for security, governance, and excellence — adopt
  it and you inherit the field's hard-won knowledge. Conformance = "you are doing
  it the proven-good way."
- **Guiding when unmet.** A failed conformance check is not just a rejection; it
  is **diagnostic** — it names the gap. This is exactly Agent-Bench's
  **improvement areas**: the protocol shows you *how to get better*, turning a
  gate into guidance.

So enforcement and excellence are the same coin: the protocol pulls quality *up*
(best practice to aspire to and adopt), and only incidentally filters *out* what
falls short. We optimize for elevation, not prohibition — a permissionless
federation grows by **raising the floor with shared best practice**, not by
gatekeeping.

### Research, theory, test — the precursors to enforcement

Nothing becomes enforcement directly. A capability matures through **precursor
stages** before it is allowed to bind as protocol:

```
research ──► theory ──► test ──► protocol@v ──► enforcement
(papers,     (formal      (benchmarks +    (canonical,     (conformance
 reference    grounding,    unit +          versioned        verified by
 library)     IRT, proofs)  conformance)    semantics)       consumers)
```

- **Research** — observed in the field; enters via the **reference library**
  (e.g. scaffold-driven shift, reward hacking, CLEAR dimensions).
- **Theory** — formal grounding for *why* it holds (e.g. IRT/Fisher information
  behind mid-range selection; geometric mean for ratio aggregation).
- **Test** — empirically validated: benchmark runs, unit tests, and a
  **conformance suite** that pins the behavior machine-checkably.
- **Only then → protocol@v** — promoted into the canonical, versioned language,
  at which point conformance verification makes it enforcement.

So enforcement is **earned**, not declared: a metric or rule is researched,
theoretically justified, and tested first — and the test artifacts *become* the
conformance suite that later enforces it. This keeps the language credible
(grounded in research + theory) and the enforcement trustworthy (backed by
tests), closing the loop with governed evolution above.

## What this means for Agent-Bench

Agent-Bench is a **steward of the evaluation language**: its glossary + formulas
+ result-package schema are part of the federation's canonical semantics. Keeping
them sacrosanct and interoperable is as important as the scores themselves — the
scores are only meaningful because the language is.
