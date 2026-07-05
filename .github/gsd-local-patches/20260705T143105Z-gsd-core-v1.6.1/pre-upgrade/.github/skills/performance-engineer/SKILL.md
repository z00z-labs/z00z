---
name: performance-engineer
description: Auto-invoked when user wants to investigate slowness, latency spikes, throughput limits, memory pressure, expensive queries, hot paths, regressions, or scalability risks, and needs a performance review, profiling plan, optimization strategy, or observability-driven bottleneck analysis. Also triggers on profiling, flame graph, p95, p99, CPU hotspot, memory leak, load test, caching, performance budget, and capacity planning.
---

# Performance Engineer

Review a system, service, feature, query path, or frontend flow through a
performance lens so bottlenecks, wasted work, scaling limits, and missing
measurement become explicit before optimization work starts.

## When to Use

- User asks why something is slow, regressed, overloaded, or not scaling.
- The task is to review latency, throughput, resource usage, tail behavior, or
  performance risk before a release or major change.
- The user wants a profiling plan, optimization strategy, load-test design, or
  observability-based bottleneck analysis.
- A code review or architecture review needs a dedicated performance pass.
- The artifact under review may have been written by a human, AI assistant, or
  mixed workflow. This skill is coder-agnostic: evaluate the workload,
  evidence, and bottlenecks, not the author.

## When Not to Use

- The task is feature delivery with no stated performance goal or symptom.
- There is no code, runtime behavior, or measurable workload to inspect.
- The request is purely about security, correctness, or architecture with no
  performance question.
- The user only wants generic tuning tips without tying them to a real system.

## How It Works

1. **Define the performance question**
   - State what is slow or risky: endpoint, batch job, render path, database
     query, startup path, background worker, or system-wide workload.
   - Record the important metric: latency, p95 or p99, throughput, CPU, memory,
     I/O, network cost, queue depth, or error rate under load.
   - Anchor the review to user impact or business impact, not just raw numbers.

2. **Establish the baseline and workload shape**
   - Capture current behavior, relevant traffic shape, data size, concurrency,
     and environment assumptions.
   - Distinguish one-off slowness from steady-state bottlenecks and from load
     amplification problems.
   - Do not compare numbers across different environments as if they were one
     benchmark.

3. **Trace the critical path**
   - Map the path from entry to response or completion, including storage,
     caches, queues, network hops, serialization, and external calls.
   - Identify where time, memory, allocations, blocking, retries, or fan-out
     accumulate.
   - Separate the dominant path from secondary noise.

4. **Find the bottleneck class**
   - Classify the main issue as compute-bound, memory-bound, I/O-bound,
     network-bound, lock-contention, query-shape, cache-miss, N+1, cold-start,
     over-serialization, excess allocations, or load-distribution problem.
   - Collapse repeated symptoms into one root bottleneck when they share the
     same cause.
   - Prefer root-shape findings over a long list of local micro-issues.

5. **Check measurement quality**
   - Prefer traces, profiles, flame graphs, query plans, metrics, targeted load
     tests, and reproducible timings over intuition.
   - Mark missing observability as a finding when it blocks confident triage.
   - Distinguish `measured bottleneck`, `likely bottleneck`, and
     `needs instrumentation`.

6. **Evaluate optimization options**
   - Propose the narrowest change that removes the highest-cost bottleneck.
   - Compare options such as algorithm change, batching, indexing, caching,
     parallelism, reduced allocations, query rewrite, payload reduction,
     backpressure, or architecture change.
   - State expected impact, trade-offs, and regression risks instead of giving
     generic advice.

7. **Guard against false wins**
   - Do not treat synthetic microbench improvements as proof of user-visible
     gains if the critical path is elsewhere.
   - Watch for shifted cost: lower CPU but higher memory, better average but
     worse tail latency, better local latency but worse downstream pressure.
   - Treat cache-based fixes as incomplete unless invalidation, warmup, and
     hit-rate assumptions are explicit.

8. **Turn findings into a validation plan**
   - Define what to benchmark, profile, trace, or load-test after the change.
   - Recommend regression guards such as performance budgets, dashboards,
     alerts, benchmark fixtures, or targeted tests.
   - End with the smallest next step that reduces uncertainty fastest.

## Review Lenses

- **Hot-path lens**: where end-to-end time is actually spent.
- **Data-shape lens**: how input size, cardinality, and fan-out change cost.
- **Resource lens**: CPU, memory, I/O, network, connection pools, and locks.
- **Tail-latency lens**: p95 and p99 behavior, not just averages.
- **Scalability lens**: what breaks or degrades as traffic or data grows.
- **Observability lens**: whether the current telemetry is good enough to prove
  the bottleneck and the fix.

## Evidence Rules

- Do not call something a performance problem without naming the metric and the
  path where the cost appears.
- Do not recommend optimization before checking whether the bottleneck is
  measured, inferred, or still unproven.
- Do not pad the review with generic tuning tips unrelated to the observed
  workload.
- If evidence comes from a profile, trace, query plan, benchmark, or load test,
  say so. If it comes from code inspection only, say that too.
- Prefer one high-confidence bottleneck with a clear fix over many weak
  optimization guesses.

## Review Output

When using this skill, structure the output around these fields:

- Scope and workload
- Performance goal or symptom
- Baseline and evidence
- Findings ordered by severity or expected impact
- Bottleneck classification
- Optimization options and trade-offs
- Validation plan and regression guards

## Examples

### Example 1: Slow API Path

```text
User: Review this API flow. It is slow under load.
Assistant: First define the latency target and workload shape, trace the
critical path through storage, caches, and downstream calls, then report the
measured or likely bottleneck with the narrowest fix and validation plan.
```

### Example 2: Frontend Regression

```text
User: Our frontend got sluggish after the last feature. Do a performance pass.
Assistant: Anchor the review to the user-visible slowdown, inspect the render
path, payload size, and network behavior, then separate measured regressions
from optimization guesses before proposing fixes.
```

### Example 3: Mixed Human And AI Changes

```text
User: Some of this system was written by AI and some by people. Review it for
performance.
Assistant: Keep the review coder-agnostic, evaluate only the workload, code,
telemetry, and bottleneck evidence, and report performance findings with clear
confidence levels.
```

## Notes

- Performance work starts with measurement, not with optimization folklore.
- Tail latency and workload shape matter more than isolated average timings.
- Missing telemetry can itself be the highest-priority performance finding.
- If the user asks for a review, bottlenecks and trade-offs come first and
  broad performance theory stays secondary.