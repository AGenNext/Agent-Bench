# Structural Performance Evaluation of Multi-Agent Systems — Glossary, Methods & Formulas

Reference notes distilled from:

> Dariusz Król, Michał Zelmozer. *Structural Performance Evaluation of
> Multi-Agent Systems.* Journal of Universal Computer Science, 14(7):1154–1178, 2008.

**One-line summary.** Define lightweight **connection metrics** to evaluate the
structural performance of distributed multi-agent platforms **without
reorganizing the running system**, then validate them empirically by comparing
two Java RMI platforms — **Aglets** and **Jade** — across network configurations.

---

## 1. Glossary

| Term | Definition |
|---|---|
| **Multi-agent system (MAS)** | A distributed system of cooperating software agents, here implemented over Java RMI. |
| **Java RMI** | Java Remote Method Invocation — the distribution mechanism underlying both evaluated platforms. |
| **Aglets / Jade** | The two open-source Java mobile-agent platforms compared in the study. |
| **Structural performance** | Performance arising from the system's connection structure (host topology, network conditions), independent of application logic. |
| **Metric** | A comparison of two or more measures (per the IEEE definition); here, a function quantifying connection quality. |
| **Connection metric** | The cheap, analytical family of metrics proposed to estimate distance/cost between agents without instrumenting the live system. |
| **Connection cost metric** | The main metric: the average "distance" (round-trip messaging cost) between agents taking part in the experiment. |
| **Host latency** | Time a host takes to process received messages (to be minimized). |
| **Degree of stability** | Probability that a host stays available/connected over a time interval. |
| **Degree of security** | Probability that a host is secure over a time interval. |
| **Distance function `dt`** | A **pseudo-metric** over the set of hosts `H` combining latency, stability, and security at a given time `t`; represents the cost of connecting two hosts. |
| **Pseudo-metric** | A distance-like function that may assign zero distance to distinct points (relaxes the strict metric axioms). |
| **Round-trip exchange time** | Concrete measurement: time to send a message from an agent on one host to an agent on another and receive the reply. |
| **Network configuration** | Test topology: single host, two hosts on a LAN, two remote hosts, and remote hosts separated by larger distance. |
| **Agent broker** | Directory service each agent queries to learn where to send the next message. |

---

## 2. Methods

### 2.1 Goal
Develop connection metrics that (a) evaluate distributed MAS performance
**without reorganizing the running system**, and (b) are validated by
experiments across varied network/environment configurations.

### 2.2 The connection metric
A host belongs to a set `H`. The **connection cost metric** is modeled with a
distance function `dt: H × H → ℝ` (a pseudo-metric depending on time `t`), built
from three time-dependent functions per host:
- **latency** — message-processing time (minimize),
- **degree of stability** — probability the host stays connected,
- **degree of security** — probability the host is secure.

Time is divided into discrete intervals on a globally synchronized clock; the
study focuses on the latency component (stability/security set aside for the
core experiments).

### 2.3 Experimental protocol
- **Scenario:** a message is passed through all participating agents; on each
  receipt an agent queries the **agent broker** for the next destination.
- **Variables:** network configuration (single host → LAN → remote → distant
  remote) and number of simultaneous messages (1 → 10).
- **Comparison:** Aglets vs. Jade under identical conditions.
- **Measurement:** round-trip message-exchange time = the empirical connection
  cost, measured from send until reply.

### 2.4 Findings
- The proposed connection metrics are **adequate across diverse configurations**
  (confirmed by measurement vs. calculation agreement).
- The study characterizes Aglets' vs. Jade's behavior and the impact of network
  configuration on agent communication cost.

---

## 3. Formulas / metric definitions

**Connection cost (pseudo-metric over hosts):**
```
dt : H × H → ℝ            # depends on time t (discrete intervals)
dt(a, b) = cost of connecting hosts a and b
         = f( latency(a,b,t), stability(·,t), security(·,t) )
   pseudo-metric: dt(a,a) may be 0, symmetry & triangle-inequality relaxed
```

**Host latency (component to minimize):**
```
latency(host, t) = time the host takes to process a received message
```

**Empirical connection cost (what is measured):**
```
ConnectionCost = round-trip time
               = t_reply_received − t_message_sent
```

**Average distance between agents (the reported metric):**
```
AvgConnectionCost = (1/N) · Σ_{i=1..N} ConnectionCost_i
   over N message exchanges in a given network configuration
```

**Degrees of stability / security (probabilistic components):**
```
stability(host, t) = P(host remains connected during interval t)
security(host, t)  = P(host is secure during interval t)
```
