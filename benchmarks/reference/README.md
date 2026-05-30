# Agent Evaluation & Benchmarking — Reference Library

Distilled reference notes for the literature and tooling that inform Agent
Bench's benchmark suites and evaluation contracts. Each document follows the
same structure:

1. **Glossary** — key terms and concepts, defined.
2. **Methods** — the approach, design, and findings.
3. **Formulas** — the metric definitions and equations.

These are study notes for benchmark design, not reproductions of the sources.
Each file links back to its original.

## Index

| Reference | Focus | Source |
|---|---|---|
| [efficient-benchmarking-ai-agents.md](efficient-benchmarking-ai-agents.md) | Reducing benchmark cost via mid-difficulty task selection; rank vs. score prediction | Ndzomga, arXiv:2603.23749 |
| [clear-enterprise-evaluation.md](clear-enterprise-evaluation.md) | CLEAR: Cost, Latency, Efficacy, Assurance, Reliability — enterprise agent evaluation | Mehta, arXiv:2511.14136 |
| [agent-as-a-judge.md](agent-as-a-judge.md) | Agent-as-a-Judge framework & the DevAI benchmark (process-level evaluation) | Zhuge et al., arXiv:2410.10934 |
| [agentboard.md](agentboard.md) | Fine-grained progress-rate metric for multi-turn LLM agents; 9 tasks across embodied/game/web/tool | Ma et al., NeurIPS 2024 D&B |
| [akg-kernel-agent.md](akg-kernel-agent.md) | Multi-agent kernel synthesis; KernelBench correctness + speedup (geomean, fast_p) | Du et al., arXiv:2512.23424 |
| [agent-kernel-mas.md](agent-kernel-mas.md) | Society-centric microkernel MAS for social simulation; system-level perf (throughput, CV, ticks) | Mao et al., arXiv:2512.01610 |
| [mlflow-agent-evaluation.md](mlflow-agent-evaluation.md) | Scorers, LLM judges, traces, trajectory metrics, judge calibration | MLflow practical guide + docs |
| [aws-bedrock-agentcore-evaluations.md](aws-bedrock-agentcore-evaluations.md) | Built-in/custom evaluators, ground-truth & trajectory scoring, online vs. on-demand | AWS Bedrock AgentCore docs |
| [petri-net-agent-performance.md](petri-net-agent-performance.md) | Design-time performance prediction via pa-UML → GSPN | Merseguer, Campos & Mena |
| [structural-mas-performance.md](structural-mas-performance.md) | Connection-cost metrics for distributed multi-agent platforms (Aglets vs. Jade) | Król & Zelmozer, J.UCS 14(7) |

## Themes across the library

- **Rankings over absolute scores.** Leaderboards consume orderings; rankings
  survive distribution shift better than calibrated scores
  (*efficient-benchmarking*).
- **Beyond accuracy.** Cost, latency, reliability (pass@k), and policy/safety
  are first-class for production deployment (*CLEAR*, *AWS AgentCore*, *MLflow*).
- **Process, not just outcome.** Score the trajectory — tool calls, steps,
  intermediate requirements, progress toward subgoals — not only the final
  answer (*Agent-as-a-Judge*, *AgentBoard*, *MLflow*, *AWS AgentCore*).
- **Design-time & structural performance.** Classical MAS work predicts
  performance from formal models and connection metrics before deployment
  (*Petri net*, *structural MAS*).

## Related repo artifacts

- [../../contracts/benchmark-contract.md](../../contracts/benchmark-contract.md) — the benchmark definition contract.
- [../memory/](../memory/) — the AMB-001 agent-memory benchmark suite.
