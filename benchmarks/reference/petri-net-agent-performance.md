# Performance Evaluation of Agent-Based Systems: A Petri Net Approach — Glossary, Methods & Formulas

Reference notes distilled from:

> José Merseguer, Javier Campos, Eduardo Mena. *Performance Evaluation for the
> Design of Agent-based Systems: A Petri Net Approach.* University of Zaragoza.

**One-line summary.** Predict the performance of mobile-agent software **early**
in design — before implementation — by translating performance-annotated UML
(pa-UML) models into **Generalized Stochastic Petri Nets (GSPN)** and computing
performance indices (notably system response time) from their steady state.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **Mobile agent** | Software entity that can migrate across hosts to carry out a task (e.g. a software retrieval service). |
| **pa-UML** | Performance-annotated UML — UML diagrams (use case, sequence, statechart) annotated with system load, delays, and routing rates. |
| **Petri net** | A formal model of concurrent systems with places, transitions, and tokens. |
| **GSPN (Generalized Stochastic Petri Net)** | A Petri net with timed transitions (exponential firing rates) and immediate transitions; supports quantitative performance analysis. |
| **Timed transition** | A transition annotated with an exponential **firing rate**; models a delay/service. |
| **Immediate transition** | A zero-delay transition annotated with routing/weight probabilities. |
| **Firing rate** | Rate parameter of a timed transition; its inverse is the mean service time. |
| **Steady state** | The long-run stationary distribution of the GSPN, from which performance indices are computed. |
| **Throughput** | The steady-state firing frequency of a transition (completions per unit time). |
| **System response time** | Time to serve a user request; here, the inverse of the throughput of the `select_sw_service` transition. |
| **Performance index** | A computed measure (throughput, mean number of tokens, utilization, response time) derived from the GSPN. |
| **RPC strategy** | Remote-procedure-call design — computation stays put and communicates remotely. |
| **Travel (migration) strategy** | Mobile-agent design — the agent migrates to the data/host to compute locally. |
| **Software Retrieval Service** | The case-study system: lets users select and download new software efficiently. |
| **GreatSPN** | The tool used to compute steady-state throughput / performance indices of the GSPN. |

---

## 2. Methods

### 2.1 Design-time performance prediction
The approach is integrated into the **early** stages of development so behavior
can be predicted **without** building the full implementation, avoiding multiple
throwaway prototypes.

### 2.2 Modeling pipeline (pa-UML → GSPN → indices)
1. **Model** the system in UML (use case, sequence, statechart diagrams) and
   **annotate** it with performance information (load, delays, routing rates) →
   **pa-UML**.
2. **Translate** the pa-UML model into a **GSPN**, giving UML a formal semantics
   in terms of Petri nets. Timed transitions carry firing rates; immediate
   transitions carry routing weights.
3. **Analyze** the GSPN: compute the steady state and derive performance indices
   with **GreatSPN**.

### 2.3 Case study and comparison
The **Software Retrieval Service** is modeled, then analyzed under different
design strategies — **RPC** vs. **travel (agent migration)** — and scenarios
("intelligent browser", "expert user", "fast/slow connection") to compare
**system response time** as the number of requests grows.

### 2.4 Key findings
- Response time is obtained as the **inverse of the throughput** of the
  `select_sw_service` transition at steady state.
- As the number of concurrent requests rises, per-request response time
  increases (contention).
- RPC and travel strategies show only small differences in some scenarios; the
  model lets designers choose without implementing each prototype.
- Predicted results coincided with those obtained by the ANTARCTICA system
  designers — validating design-time prediction.

---

## 3. Formulas / metric definitions

**System response time** (the central index):
```
ResponseTime = 1 / Throughput(select_sw_service)
   where Throughput is the steady-state firing frequency of the transition,
   computed from the GSPN steady-state distribution (via GreatSPN).
```

**Mean service time of a timed transition:**
```
MeanServiceTime = 1 / (firing rate λ)
```

**General GSPN performance indices** (computed from the steady-state
distribution π over markings):
```
Throughput(t)      = Σ_{markings enabling t} π(marking) · rate(t)
Mean #tokens(p)    = Σ_markings π(marking) · tokens_in_p(marking)
Utilization(server)= fraction of steady-state time the server is busy
```

**Little's-law relationship** (queueing interpretation linking the indices):
```
Mean #requests in system = Throughput × ResponseTime
```
