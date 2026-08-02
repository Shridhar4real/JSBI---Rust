# JSBI Rust Port (Port Mortem 2026 - Track F: JavaScript to Rust)

An idiomatic, pure-Rust standalone port of [`GoogleChromeLabs/jsbi`](https://github.com/GoogleChromeLabs/jsbi) - the JavaScript BigInt polyfill. This implementation completely eliminates the Node.js/V8 runtime dependency while maintaining **100% behavioral equivalence**.

---

## 1. Single-Step Build Command 

This project strictly satisfies the **"Standalone & Runnable"** hackathon requirement. The entire project (both the library and standalone CLI binary) compiles in one step using the release profile:

```bash
cargo build --release
```

- **Compiled Outputs**:
  - Library: `target/release/libjsbi.rlib`
  - Standalone Binary: `target/release/jsbi-cli` (or `target/release/jsbi-cli.exe` on Windows)

---

## 2. Test Execution Instructions

### A. Native Rust Integration Tests
To run the pure-Rust test suite verifying arithmetic, bitwise shifts, string parsing, and DataView operations:
```bash
cargo test
```

### B. Original JavaScript Test Parity
To run the original JavaScript test suite against our compiled Rust binary:
```bash
npm test
```
*Note*: This executes the original, **unmodified JavaScript test suite** through our custom Node.js thin adapter bridge (`tests/resolve.source.mjs` and `tests/jsbi-adapter.mjs`), which intercepts JSBI method calls and routes them directly to the compiled Rust binary.

---

## 3. Migration Rationale & Performance Enhancements

- **Core Architectural Win**: Intercepted the Node.js module loader to achieve **100% test parity** without modifying a single line of code in the original test files inside [`tests/original/`](./tests/original/).
- **Honest Benchmarks** (from [`bench/results.json`](./bench/results.json)):
  - **Startup Time**: **19.84x faster startup** (`13.98 ms` Rust vs `277.33 ms` JS).
  - **Memory Footprint (RSS)**: **92.8% reduction in peak memory** (`3.80 MB` Rust vs `52.76 MB` JS).
  - **Latency**: Up to **6.67x latency reduction** in arbitrary-precision operations (`add`, `multiply`, `divide`, `bitwise_and`).
- For a comprehensive log of all architectural design decisions and technical rationale, see [`DECISIONS.md`](./DECISIONS.md). For detailed profiling methodology and confounder analysis, see [`bench/methodology.md`](./bench/methodology.md).

---

## 4. Bonus Points Claimed

### 🛡️ Zero Unsafe 
- **Status**: **CLAIMED**
- **Verification**: Enforced crate-wide via `#![deny(unsafe_code)]` at the root of `src/lib.rs` and `src/main.rs`. Zero `unsafe` blocks exist in the entire codebase.

### ⚡ Differential Fuzz Survivor 
- **Status**: **CLAIMED**
- **Verification**: The port survived **65 continuous seconds** of intensive differential fuzzing, processing **486,013 randomized test cases** across arithmetic, bitwise, and edge-case inputs against the reference JS BigInt engine with **ZERO divergences**.
- **Proof**: See [`fuzz/log.txt`](./fuzz/log.txt) and execute `cargo run --release --bin harness` to verify live.

---

## 5. Project Structure Overview


- [`src/`](./src/): Safe, idiomatic Rust implementation (`lib.rs`, `bigint.rs`, `error.rs`, `main.rs`).
- [`tests/original/`](./tests/original/): The 100% untouched original JavaScript test suite.
- [`fuzz/`](./fuzz/): Differential fuzzing harness (`harness.rs`) and verified 65-second success log (`log.txt`).
- [`bench/`](./bench/): Honest benchmark suite (`run_benchmarks.mjs`), results (`results.json`), and methodology breakdown (`methodology.md`).
- [`DECISIONS.md`](./DECISIONS.md): Complete architectural decision log documenting all non-trivial technical design choices.
- [`.port-mortem.toml`](./.port-mortem.toml): Track F configuration and kickoff hash metadata.
