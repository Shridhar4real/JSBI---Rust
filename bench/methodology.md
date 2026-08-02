# Benchmarking Methodology & Honest Performance Analysis

This document details the benchmarking procedures, profiling tools, metrics, and technical confounders evaluated when comparing the JavaScript `JSBI` polyfill with our native Rust port (`target/release/jsbi-cli`).

---

## 1. Metrics & Profiling Tools

### A. p99 Latency (99th Percentile Execution Time)
- **Tool / Function**: `performance.now()` high-resolution timers (`perf_hooks` module) in Node.js v20 and native Rust `std::time::Instant`.
- **Workload**: Evaluated over 10,000 iterations of heavy 200+ digit BigInt operations:
  - `add`: Addition of two 200-digit operands.
  - `multiply`: Multiplication of 200-digit operands.
  - `divide`: Knuth Algorithm D arbitrary-precision division.
  - `bitwise_and`: Bitwise AND operation across 30-bit digit representations.

### B. Startup Time
- **Tool / Function**: Process spawn duration measurements averaged over 30 cold-start process invocations via `child_process.spawnSync`.
- **JS Startup**: Cold invocation of `node` loading the JSBI module graph (~277.33 ms).
- **Rust Startup**: Invocation of `target/release/jsbi-cli eval BigInt 123` (~13.98 ms).

### C. Peak Memory Footprint (Resident Set Size - RSS)
- **Tool / Function**: `process.memoryUsage().rss` in Node.js and OS process RSS profiling (`GetProcessMemoryInfo`).
- **JS RSS**: ~52.76 MB (baseline V8 isolate, heap initialization, and module graph).
- **Rust RSS**: ~3.80 MB (lean native executable memory overhead).
- **RSS Footprint Reduction**: **92.8% reduction** in peak memory usage.

---

## 2. Technical Confounders & Honest Analysis

As required by the Port Mortem hackathon standards, we explicitly detail all system confounders to ensure an accurate, honest benchmark comparison rather than artificial throughput metrics:

1. **Child Process IPC Overhead vs. Native Function Execution**:
   - Spawning `target/release/jsbi-cli` via `execFileSync` in Node.js incurs OS process creation overhead (~10–15 ms per process launch).
   - In pure native Rust execution (or when embedded as a native Rust crate), BigInt operations execute in sub-microsecond time (~0.008 ms for addition, ~0.019 ms for multiplication), eliminating process creation costs entirely.

2. **V8 JIT Warmup vs. Instant Native Optimization**:
   - V8 requires initial warm-up iterations to JIT-compile JavaScript bytecode into optimized machine code. Before JIT optimization, JS execution is significantly slower.
   - The Rust release binary (`cargo build --release`) is pre-compiled via LLVM with maximum optimization (`opt-level = 3`, `lto = true`, `codegen-units = 1`), delivering instant peak performance from the very first instruction without JIT warmup latency.

3. **Memory Footprint Disparity**:
   - JavaScript runtimes necessitate allocating V8 heaps, garbage collector structures, and JIT code caches (~50+ MB RSS).
   - Pure Rust uses deterministic stack and heap allocations without garbage collection, keeping memory usage down to ~3.8 MB RSS.

---

## 3. Summary of Results (`bench/results.json`)

| Metric / Operation | JS (Node.js) | Rust Port | Performance Ratio / Improvement |
| :--- | :--- | :--- | :--- |
| **Startup Time** | `277.33 ms` | `13.98 ms` | **19.84x Faster Startup** |
| **Peak Memory (RSS)** | `52.76 MB` | `3.80 MB` | **92.8% Memory Reduction** |
| **`add` p99 Latency** | `0.0450 ms` | `0.0080 ms` | **5.63x Latency Reduction** |
| **`multiply` p99 Latency** | `0.1200 ms` | `0.0190 ms` | **6.32x Latency Reduction** |
| **`divide` p99 Latency** | `0.2800 ms` | `0.0420 ms` | **6.67x Latency Reduction** |
| **`bitwise_and` p99 Latency** | `0.0380 ms` | `0.0060 ms` | **6.33x Latency Reduction** |
