# JSBI Rust Port (Port Mortem 2026 - Track F)

This repository contains an idiomatic, pure-Rust port of [`GoogleChromeLabs/jsbi`](https://github.com/GoogleChromeLabs/jsbi) - the JavaScript BigInt polyfill.

## Single Build Command
To compile the release binary and library:
```bash
cargo build --release
```

## Migration Rationale & Key Divergences
The primary objective of this port is **strict behavioral equivalence** with the original JSBI implementation while embracing Rust's safety, strict typing, and memory model.

Key architectural choices:
1. **Zero Unsafe Code**: Enforced via `#![deny(unsafe_code)]` at the crate root.
2. **Explicit Type Safety & Error Handling**: Rather than throwing JavaScript runtime exceptions (`RangeError`, `SyntaxError`, `TypeError`), all Rust methods return `Result<JSBI, JSBIError>` or safe fallbacks.
3. **30-bit Digit Storage**: Retained the 30-bit radix representation (`0..0x3FFFFFFF`) to ensure 100% bit-for-bit algorithmic alignment with JSBI's division, bitwise shifts, string conversion, and `asIntN`/`asUintN` semantics.
4. **No Node.js / V8 Dependency**: The Rust library and CLI executable are built as standalone native binaries requiring zero external JavaScript runtimes.

For a detailed breakdown of all non-trivial design choices, consult [DECISIONS.md](./DECISIONS.md).

## Project Structure
- `src/`: Safe, idiomatic Rust code (`lib.rs`, `bigint.rs`, `error.rs`, `main.rs`).
- `tests/original/`: Unmodified JavaScript test suite.
- `tests/port/`: Rust integration and unit tests.
- `fuzz/`: Differential fuzzing harness (`harness.rs`) and log (`log.txt`).
- `bench/`: Latency, RSS, and startup time benchmark methodology (`methodology.md`) and results (`results.json`).
- `.port-mortem.toml`: Hackathon track F metadata and kickoff hash.
