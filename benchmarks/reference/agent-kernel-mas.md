# Agent-Kernel (Social-Simulation MAS) — Glossary, Methods & Formulas

Reference notes distilled from:

> Mao, Liu, Wang, Ding, Miao, et al. *Agent-Kernel: A MicroKernel Multi-Agent
> System Framework for Adaptive Social Simulation Powered by LLMs.* Technical
> Report V1.0, Zhejiang University et al. arXiv:2512.01610v1, Dec 2025.
> Code: https://github.com/ZJU-LLMs/Agent-Kernel

> **Not to be confused with** `akg-kernel-agent.md` (compute-kernel synthesis).
> This Agent-Kernel is a *society-centric microkernel MAS framework* for LLM
> social simulation.

**One-line summary.** A **society-centric modular microkernel** architecture for
LLM social simulation: decouple core system functions from simulation logic, and
**Agent / Environment / Action** from each other, so a society's population and
profiles can change at runtime. Validated on Universe 25 (mouse-utopia, dynamic
population) and a 10,000-agent ZJU campus simulation.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **Social simulation** | Using LLM role-play to model human behavior and societal dynamics. |
| **Microkernel architecture** | A small stable **core system** + extensible **plugins**; general-purpose infra is pre-built, users write only plugins. |
| **Society-centric (vs. agent-centric)** | Models environments, rules, and actions as independent society-level entities — not embedded inside each agent. |
| **Agent–Environment–Action decoupling** | Separating an agent's *cognition* from the shared *environment* and *actions*, so each can update independently at runtime. |
| **Core modules (5)** | **Agent, Environment, Action, Controller, System** — the stable kernel. |
| **Agent module** | Holds only cognition: profile + dynamic state + the cognitive loop. |
| **Environment module** | Single source of truth for the objective world (Space, Relation plugins). |
| **Action module** | Shared, discoverable capabilities agents can invoke (Communication, Tools, Other-Actions). |
| **Controller** | Stateless central mediator: validates every action request and enables runtime intervention; horizontally scalable. |
| **System module** | Global services: **Timer**, **Messager**, **Recorder**. |
| **Cognitive loop** | The BDI-inspired routine **Perceive → Plan → Invoke → State → Reflect** (configurable order). |
| **Plugin** | A hot-swappable unit implementing scenario logic; conforms to abstract interfaces (`init()`, `execute()`). |
| **Database-per-Plugin** | Each plugin owns its store (graph DB for relations, vector DB for memory) — heterogeneous persistence + fault isolation. |
| **Timer / tick** | Global clock; simulation advances in **ticks** (e.g. 48 ticks = 24h), preventing causal inversion. |
| **Messager** | Asynchronous message routing decoupling senders/receivers, avoiding deadlocks. |
| **Recorder** | Unified logging → traceable, reproducible audit trail. |
| **MasPod** | Deployment unit: a group of agents + local Environment/Action/Controller (Ray actor). |
| **PodManager** | Orchestrates MasPods, balances load by assigning new agents to the least-loaded pod. |
| **Four design dimensions** | **Adaptability, Configurability, Reliability, Reusability** — the goals the architecture optimizes. |
| **PCG** | Procedural Content Generation tool to batch-create initial agent/relation/spatial data. |
| **SocietyHub / Society-Panel** | Community model-sharing hub; visual deploy/config/monitor panel. |

---

## 2. Methods

### 2.1 Why microkernel + society-centric
Pipeline and layered MAS frameworks bake agents' actions/environment into static
agent profiles → adding one agent cascades updates to all peers. Agent-Kernel
instead treats environment/actions as shared society-level entities, so
population can grow/shrink (birth, death, migration) by mere registration.

### 2.2 The five core modules
- **Agent** — cognition only; runs the Perceive→Plan→Invoke→State→Reflect loop.
- **Environment** — authoritative world (Space, Relation plugins).
- **Action** — shared capability registry (Communication, Tools, Other-Actions);
  `@AgentCall` annotations gate which methods agents may call (permission control).
- **Controller** — stateless mediator; **all** messages route through it for
  validation + runtime intervention (and rollback / what-if analysis).
- **System** — Timer (deterministic time), Messager (async, deadlock-free),
  Recorder (unified audit log).

### 2.3 Plugins & data
Plugins are hot-swappable behind abstract interfaces; **Database-per-Plugin**
gives heterogeneous persistence and fault isolation (a schema change stays
local). Implemented with the Mediator + Component-Plugin + Facade patterns.

### 2.4 Distributed mode (scale)
K8s-inspired: agents are packed into **MasPods** (Ray actors); a **PodManager**
handles lifecycle, inter-pod messaging, and adaptive load balancing.

### 2.5 Validation
- **Universe 25**: population 8 → peak 313 over 1,729 ticks (73 days), 447 life
  cycles, 126,265 events — runtime tracks population (adaptability).
- **ZJU Campus**: 10,000 agents over 50 pods, 336 ticks (7 days), 1.26B tokens,
  uniform memory (CV 4.86%) — large-scale load balancing.

### Relevance to Agent-Bench
A reference for the **`agent_coordination`** benchmark category and for
*system-level* performance metrics (throughput per tick, token usage, memory
balance) alongside behavioral-fidelity scoring — distinct from single-agent
task benchmarks.

---

## 3. Formulas / metric definitions

The paper is architectural; its quantitative metrics are simulation
performance/fidelity measures.

**Tick ↔ wall-clock mapping:**
```
sim_time = ticks × (sim_seconds_per_tick)      # e.g. 48 ticks = 24h
```

**Per-tick execution time** dominated by agent execution:
```
tick_time ≈ T_agent_exec + T_msg_dispatch + T_status_update + T_eval
   (ZJU: agent exec ≈ 93.0%, dispatch ≈ 6.37%)
```

**Capability-rating coefficient of variation** (load-balance quality):
```
CV = σ / μ           # ZJU memory: μ=1.36 GB, σ=67.71 MB → CV ≈ 4.86%
```
Low CV ⇒ uniform pod load (good balancing).

**Behavior proportion** per day (behavioral-pattern fidelity):
```
proportion_c(day) = (events in category c that day) / (total events that day)
```

**Net population change:**
```
ΔN(t) = births(t) − deaths(t)
```

**Adaptive pod assignment** (PodManager):
```
target_pod = argmin_p  agent_count(p)      # new agent → least-loaded pod
```

**Token throughput:**
```
tokens_per_tick = total_tokens / ticks     # ZJU: 1.26e9 / 336 ≈ 3.75e6
```
