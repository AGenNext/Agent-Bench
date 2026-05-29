# Agent Memory Benchmarks

Reproducible benchmark suites comparing agent memory frameworks on tasks
that matter in production.

## AMB-001 — Recall & Conflict Resolution

**Status:** Fixture development in progress. First run scheduled after
Agent-Memory v0.1.0 ships and live deployment metrics begin publishing.

**Participants:** Agent-Memory, Mem0, Zep, Letta

**What it measures:**

| Metric | Why it matters |
|---|---|
| Recall accuracy | Does the right memory surface for the right query |
| Gap handling | Does the framework give a useful signal when nothing is found |
| Conflict handling | Does it distinguish misinterpretation from factual dispute |
| Decay behaviour | Does it handle stale memories gracefully |
| Cold start latency | Cost of embedded vs external dependency |
| p50/p99 recall latency | Production-grade performance comparison |
| External deps required | Operational complexity |

**Design principle:** Every task in this benchmark is a case where frameworks
diverge. Toy retrieval (fetch a name from 10 turns ago) is not here. The
hard cases are: gap protocol, conflict resolution, temporal recall,
cross-session queries, and decay-aware retrieval.

## Benchmark contract

All benchmarks follow [contracts/benchmark-contract.md](../../contracts/benchmark-contract.md).

## Publishing

Results published at `results/memory/` after each run.
Live deployment metrics published daily starting from first production run.

See [github.com/AGenNext/Agent-Memory](https://github.com/AGenNext/Agent-Memory)
for the memory layer being benchmarked.
