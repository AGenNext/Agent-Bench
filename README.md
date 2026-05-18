# Agent Bench

Agent Bench defines reusable benchmark suites for AGenNext agentic systems.

## Responsibility

Agent Bench owns reproducible benchmark task suites.

Agent Eval owns evaluation metrics and scoring.

```text
Agent-Bench
  → benchmark tasks and datasets

Agent-Eval
  → scoring, rubrics, CLEAR evaluation, quality metrics
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

## Consumers

- Agent-Team
- Agent-Eval
- Model-Router
- Agent-Knowledge
- future AGenNext products

## Core Principle

```text
Benchmarks provide reproducible tasks.
Evaluations score the outcomes.
Analytics tracks performance over time.
```

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
