# Benchmarking Methodology

This document defines the benchmarking procedure comparing the JavaScript JSBI polyfill implementation with the native Rust port.

## Metrics Measured
1. **p99 Latency**: 99th percentile execution time for 1,000,000 iterations of core BigInt arithmetic operations (`add`, `multiply`, `divide`, `bitwise_and`, `signed_right_shift`).
2. **RSS (Resident Set Size)**: Peak memory footprint in megabytes during heavy computation workloads.
3. **Startup Time**: Cold-start initialization duration from process invocation to execution completion.

## Environment & Testing Rig
- **Hardware Target**: x86_64 multi-core environment.
- **Node.js Benchmark Execution**: Native `benchmarks/*.mjs` scripts executed under Node.js v18+.
- **Rust Benchmark Execution**: Native release binary binary compiled with `cargo build --release` under `opt-level = 3` and `lto = true`.
