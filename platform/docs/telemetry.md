# Telemetry & Measurement

Agent-Bench is fundamentally a **measurement** system. It uses two stores with a
clean split of responsibilities:

| Store | Role | Workload |
|---|---|---|
| **SurrealDB** | Transactional state **+ enforcement** (PEP) | agents, benchmarks, runs, results; record-level `PERMISSIONS`; JWT auth |
| **ClickHouse** | **Telemetry & measurement** | high-volume per-step metrics, time-series, leaderboard analytics, drift |

> SurrealDB *governs and enforces*; ClickHouse *measures at scale*. Write-once
> telemetry that fans out to dashboards and analytics doesn't belong in the
> transactional store — it belongs in a columnar OLAP engine built for it.

## What we measure

Every run emits a wide, append-only event stream into ClickHouse:

| Dimension | Examples |
|---|---|
| Identity | tenant, agent, scaffold, model, benchmark, **hardware**, **dsl** |
| Outcome | success, correct, progress_rate |
| Cost | `cost_usd`, tokens (→ CLEAR Cost / CNA / CPS) |
| Latency | per-step ms, end-to-end, SLA breach (→ CLEAR Latency / SCR) |
| Reliability | per-trial pass (→ pass@k) |
| Performance | baseline vs. kernel latency (→ speedup, fast_p) |
| Assurance | policy-critical actions, violations, guard verdicts (→ PAS) |
| Time | tick / wall-clock, submission date (→ temporal leaderboard, drift) |

## Why ClickHouse for the measurement plane

- **High-cardinality, append-heavy**: agent × task × trial × step rows accumulate
  fast; columnar storage + `MergeTree` handle billions of rows cheaply.
- **Time-series native**: the temporal-leaderboard and drift questions
  (ranking ρ over time, pass-rate trends) are window/aggregate queries.
- **Aggregation at speed**: leaderboard roll-ups, percentiles (p50/p99 latency),
  geomean speedup across tasks — exactly ClickHouse's wheelhouse.

## Flow

```
run executes ─► OpenTelemetry spans/metrics
   ├─► SurrealDB   : authoritative run record + scores (enforced, transactional)
   └─► ClickHouse  : raw per-step telemetry (measurement)
                        │
                        ├─► Agent-Sight     (dashboards)
                        ├─► Agent-Analytics (cross-agent BI)
                        └─► Agent-Drift      (degradation detection → re-benchmark)
```

The scored, authoritative numbers live in SurrealDB and back the API/leaderboard;
the raw measurement firehose lives in ClickHouse and backs analytics, trends, and
drift — keeping enforcement and measurement cleanly separated.

## Status

Roadmap: the store abstraction exists (`Store`); a `TelemetrySink` trait with a
ClickHouse implementation is the next step. Until then, scores live in SurrealDB
and telemetry export is a no-op.
