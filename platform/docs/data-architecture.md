# Data Architecture — Tiers & Roles

The data layer is **tiered by workload**, each tier a purpose-built engine. They
are complementary, not competing.

| Tier | Engine | Role | Workload |
|---|---|---|---|
| **Enterprise DAM** (asset substrate) | **ArangoDB** (multimodal) | Digital Asset Management — the store of record for *all* assets | code, artifacts, logic, **tool definitions**, **skill definitions**, relationships, memories |
| **Analytics & apps** | **ClickHouse** | All analytical tools, BI, dashboards, measurement | per-step telemetry, time-series, leaderboard analytics, drift |
| **Control plane + enforcement** | **SurrealDB** (embedded) | The platform's own transactional state + data-layer PEP | agents, benchmarks, runs, scores; record `PERMISSIONS` + JWT |

## Enterprise DAM → ArangoDB (multimodal)

The enterprise's Digital Asset Management layer **becomes a multimodal database**:
**ArangoDB**, which is natively **document + graph + key/value** in one engine.
That fits the asset graph directly:

- **Documents** — code, artifacts, logic, tool/skill **definitions** (typed JSON).
- **Graph** — the relationships that matter: `skill → tools`, `skill → skill`,
  `agent → skills`, `artifact → dependencies`. Traversals (AQL) answer
  "what can this agent do, via which tools?" natively.
- **Key/value** — fast lookups by digest/id.

So "everything lives in the (multimodal) database, and the database protects it"
is realized by ArangoDB as the asset substrate; large immutable blobs are
referenced by **registry digest** (OCI/ORAS) for signed integrity.

## Analytics & apps → ClickHouse

Every analytical tool and app reads from **ClickHouse** — the measurement plane
(`telemetry.md`): high-volume, append-only, columnar, time-series-native.
Leaderboard roll-ups, p50/p99 latency, geomean speedup, drift trends, and the BI
in Agent-Analytics / dashboards in Agent-Sight all run here.

## Why three engines, not one

| Property | ArangoDB | ClickHouse | SurrealDB |
|---|---|---|---|
| Shape | multimodal asset graph | columnar OLAP | transactional + enforcement |
| Access pattern | rich traversals, CRUD | aggregate scans over billions of rows | governed reads/writes, per-record authz |
| Mutability | live, versioned | append-only | live, audited (change feeds) |
| Protects? | asset ACLs | — | record `PERMISSIONS` (PEP) |

Forcing one engine to do all three is the anti-pattern: OLAP telemetry doesn't
belong in a transactional store, and an asset graph doesn't belong in a columnar
scanner.

## How Agent-Bench uses the tiers

```
assets (agents, tools, skills, artifacts)  ──►  ArangoDB (DAM)  ── referenced by digest ─►  registry (signed)
run executes ──► scores                      ──►  SurrealDB (authoritative, enforced)
            └──► per-step telemetry           ──►  ClickHouse (measured) ──► Sight / Analytics / Drift
```

## Event streams — start at SurrealDB, graduate to Kafka

Events are first-class, but the **event bus evolves with scale**:

- **Day one (light):** SurrealDB **Live Queries + change feeds** *are* the event
  stream. `LIVE SELECT` pushes run/score changes to subscribers; `CHANGEFEED`
  gives a replayable log. No extra infrastructure — the store you already run is
  the event source.
- **At scale:** graduate to **Kafka (Strimzi)** / NATS for high-throughput,
  multi-consumer fan-out (telemetry → ClickHouse, triggers → reconcile,
  webhooks → Agent-Hooks). SurrealDB change feeds become a *source* feeding
  Kafka, not the bus itself.

The producer/consumer contract stays the same; only the transport graduates, so
nothing downstream is rewritten.

## Start light: single node, single cluster

The platform is **embedded-first by design** — you begin with the smallest thing
that works and grow only when scale demands:

| Stage | Footprint |
|---|---|
| **Start (light)** | single-node **k3s**, embedded SurrealDB (`memory`/`surrealkv`), SurrealDB change-feeds as the event stream, one API binary. *Runs on a laptop.* |
| **Grow** | external SurrealDB server, ClickHouse for telemetry, ArangoDB DAM, Kafka event bus, KEDA autoscaling, multi-node cluster. |
| **Scale** | distributed backends (TiKV under SurrealDB, ClickHouse/ArangoDB clusters), multi-cluster; **ArangoDB DAM → data lake** for long-term asset + analytical storage. |

This is why the code uses the SurrealDB `any` engine and a stateless API: the
*same binary* runs embedded on one node or against a distributed backend — the
growth path is config, not a rewrite. **Start with light components on a single
node, single cluster; add heavy components only when the load justifies them.**

### We start light, but we know where it ends

Starting small is deliberate, not naive: **the end state is designed up front**,
so every early choice is forward-compatible with it. The known destination:

```
embedded SurrealDB ──► SurrealDB server ──► + TiKV (HA)         [control plane]
SurrealDB change feed ─► Kafka / NATS                            [event bus]
(none) ──────────────► ClickHouse cluster                       [analytics]
(records) ───────────► ArangoDB DAM ──► data lake                [assets, long-term]
single-node k3s ─────► multi-node ──► multi-cluster              [runtime]
```

**ArangoDB DAM → data lake** is the terminal asset tier: live multimodal assets
in ArangoDB tier down into a data lake for long-term retention and large-scale
analytics. Because the destination is known, the light start avoids dead-ends —
schemas, the registry-digest references, and the `Store` abstraction are all
shaped for the end state from day one.

> **Note / open decision.** The platform ships today on **embedded SurrealDB**
> for its control plane + enforcement (working, tested). The **enterprise DAM is
> ArangoDB** and **analytics is ClickHouse**. Whether the platform's control-plane
> store later consolidates onto ArangoDB or stays SurrealDB is an open call; the
> repository layer (`Store`) keeps that swappable. Pinned here so the tiering is
> explicit and the docs don't imply a single-store world.
