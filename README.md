# Agent Bench

Agent Bench owns reproducible benchmark suites, benchmark evaluation runs, result packaging, and benchmark publishing for AGenNext agentic systems.

It is the SWE-bench-style benchmarking layer for agents and agent teams.

## Responsibility

Agent Bench owns:

- reproducible benchmark task suites
- benchmark evaluation runs
- benchmark result schemas
- benchmark result packages
- public/private benchmark publishing
- benchmark reports
- leaderboard inputs
- baseline result records
- reproducibility manifests

Agent Eval owns reusable evaluation metrics, rubrics, and scoring components used by Agent Bench.

```text
Agent-Bench
  → benchmark tasks, benchmark runs, result packages, publishing

Agent-Eval
  → scoring functions, rubrics, CLEAR evaluation, quality metrics
```

## Scope

Agent Bench defines:

- benchmark task suites
- task definitions
- expected outputs
- scenario fixtures
- baseline runs
- repeatability settings
- domain task categories
- benchmark result schemas
- leaderboard publishing formats
- benchmark report formats
- reproducibility manifests

## Consumers

- Agent-Team
- Agent-Eval
- Agent-Analytics
- Model-Router
- Agent-Knowledge
- Agent-World
- future AGenNext products

## Core Principle

```text
Benchmarks provide reproducible tasks.
Evaluations score the outcomes.
Bench publishes comparable results.
Analytics tracks performance over time.
```

## Publishing model

Benchmark results should include:

- benchmark suite version
- task IDs
- model/agent/team version
- runtime version
- dataset version
- scoring version
- result artifact links
- trace/evidence links
- reproducibility manifest
- pass/fail summary
- leaderboard fields

## Initial Benchmark Domains

- enterprise search
- source-to-artifact generation
- RFP filling
- vendor comparison
- candidate screening
- product documentation
- sales deck generation
- pitch deck generation
- multi-agent handoff reliability
- tool/model routing
- security and policy compliance
- cloud architect deployment workflows
