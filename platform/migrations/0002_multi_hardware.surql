-- Migration 0002 — multi-kernel / multi-hardware fields.
-- Agents are benchmarked per hardware backend and DSL; perf benchmarks
-- (KernelBench-style) add correctness + baseline latency for speedup scoring.

DEFINE FIELD hardware ON run TYPE string DEFAULT "";  -- e.g. gpu-a100, npu, cpu
DEFINE FIELD dsl      ON run TYPE string DEFAULT "";  -- e.g. triton, cuda-c, tilelang

DEFINE FIELD scores.perf                 ON run TYPE option<object>;
DEFINE FIELD scores.perf.correctness     ON run TYPE option<float>;
DEFINE FIELD scores.perf.speedup_geomean ON run TYPE option<float>;
DEFINE FIELD scores.perf.fast_1          ON run TYPE option<float>;

DEFINE FIELD correct             ON task_result TYPE bool  DEFAULT true;
DEFINE FIELD baseline_latency_ms ON task_result TYPE float DEFAULT 0.0;

-- Index for hardware-sliced leaderboards.
DEFINE INDEX run_hardware ON run FIELDS benchmark, hardware;
