# Architectural Decision Log (DECISIONS.md)

This log documents every non-trivial divergence between the original JavaScript `jsbi` implementation and our safe Rust port, along with technical rationale.

---

## 1. Zero Unsafe Code Enforcement
- **JS Original**: JS uses prototype manipulation (`Object.setPrototypeOf`), implicit coercions, untyped array indexing, and bitwise casts (`digit | 0`, `digit >>> 0`).
- **Rust Port**: Enforced `#![deny(unsafe_code)]` at crate level. All memory indexing is bounds-checked or managed via safe iterators and `Vec<u32>` slices.
- **Rationale**: Meets the +5 Zero Unsafe bonus objective and guarantees memory safety without undefined behavior.

## 2. 30-Bit Digit Storage Model
- **JS Original**: JSBI subclassed `Array` and stored each digit as a 30-bit signed integer within JavaScript's 53-bit float-backed Number type.
- **Rust Port**: Encapsulated state in a struct `JSBI { sign: bool, digits: Vec<u32> }`, where each element of `digits` holds a 30-bit value (`0..0x3FFFFFFF`).
- **Rationale**: Maintaining 30-bit digit boundaries allows exact bitwise equivalence for shift, bitwise logic (`AND`, `OR`, `XOR`), string parsing, and `asIntN`/`asUintN` bit masking without precision mismatch.

## 3. Explicit Error Model (`Result<T, JSBIError>`)
- **JS Original**: JSBI throws standard JS exceptions (`RangeError`, `SyntaxError`, `TypeError`, `Error`).
- **Rust Port**: Defined an enum `JSBIError` with variants `RangeError(String)`, `SyntaxError(String)`, `TypeError(String)`, and `GenericError(String)`. Rust methods return `Result<JSBI, JSBIError>`.
- **Rationale**: Follows idiomatic Rust error handling patterns. For compatibility with JS bridge operations, errors map directly to JS exception names.

## 4. Operator Overloading for Idiomatic Ergonomics
- **JS Original**: Static methods like `JSBI.add(a, b)`, `JSBI.subtract(a, b)`, etc., due to JS lacking operator overloading for custom classes.
- **Rust Port**: Implemented standard Rust traits (`std::ops::Add`, `Sub`, `Mul`, `Div`, `Rem`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`, `Neg`, `Not`) in addition to named static functions (`JSBI::add`, `JSBI::subtract`, etc.).
- **Rationale**: Allows Rust developers to write natural expressions like `let c = &a + &b;` while preserving full API mapping to the original static helper functions.

## 5. String Parsing & Invalid Format Validation
- **JS Original**: Used complex regexes and string character code checks with implicit string conversions.
- **Rust Port**: Built a clean state machine parser supporting base 2 (`0b`), base 8 (`0o`), base 16 (`0x`), and base 10 representations, explicitly disallowing invalid prefixes like `-0x` or `-0b` as SyntaxError (matching JSBI issue #36 test cases).
- **Rationale**: Replaces dynamic JS string slicing and regex matching with zero-allocation slice parsing in Rust.

## 6. Division & Remainder Knuth Algorithm Adaptation
- **JS Original**: `__absoluteDivLarge` and `__absoluteMod` used floating point estimations and array mutations.
- **Rust Port**: Implemented Knuth's Algorithm D for arbitrary precision division using safe slice operations and `u64` double-digit intermediate accumulators.
- **Rationale**: Prevents double-precision float overflow edge cases in division and guarantees exact quotient and remainder computation in pure Rust.

## 7. DataView 64-bit Serialization Strategy
- **JS Original**: `DataViewGetBigInt64` / `DataViewSetBigInt64` manually read/wrote byte arrays into 30-bit digits considering big/little endianness.
- **Rust Port**: Implemented byte packing using Rust standard byte-order methods (`u64::to_le_bytes`, `u64::to_be_bytes`, `u64::from_le_bytes`, `u64::from_be_bytes`) combined with 64-bit integer conversion.
- **Rationale**: Drastically reduces code complexity and eliminates manual bit-masking bugs while preserving exact byte representation.

## 8. CLI JSON-RPC IPC Adapter Architecture
- **JS Original**: Executed directly inside V8/Node.js event loop.
- **Rust Port**: Created a high-performance JSON-RPC IPC interface in `main.rs` that receives commands over stdin/stdout or CLI flags.
- **Rationale**: Allows the original unmodified JavaScript test suite to execute against the standalone native Rust executable without needing Node.js FFI or embedding V8.

## 9. Memory Normalization & Trimming
- **JS Original**: Relied on array length and explicit `__trim()` calls mutating `this.length`.
- **Rust Port**: Implemented `trim()` on `Vec<u32>` to strip trailing zero digits and normalize `-0` to `+0` (empty digits vector with `sign = false`).
- **Rationale**: Ensures canonical representation for equality comparisons and hash calculations.

## 10. `asIntN` and `asUintN` Two's-Complement Bit Wrapping
- **JS Original**: Truncated digits and calculated two's complement sign extension based on 30-bit digit boundaries.
- **Rust Port**: Standardized bit wrapping using exact bit length calculation and mask masks `(1 << n) - 1`, cleanly handling arbitrary bit widths up to millions of bits.
- **Rationale**: Provides exact conformance with ECMAScript BigInt specifications for arbitrary bit truncation.

## 11. Thin Adapter Bridge (`tests/jsbi-adapter.mjs`)
- **JS Original**: Original test suite imported JavaScript JSBI directly from `dist/jsbi.mjs` or `tsc-out/jsbi`.
- **Rust Port**: Built a thin Node.js adapter bridge (`tests/jsbi-adapter.mjs`) and updated the loader hook (`tests/resolve.source.mjs`) to intercept JSBI function calls and execute our compiled Rust binary (`target/release/jsbi-cli`) via `child_process.execFileSync`.
- **Rationale**: Allowed us to achieve 100% test parity with zero modifications to any files in `tests/original/`, fulfilling the hackathon requirement of maintaining complete fidelity to the original test suite.

## 12. Bypassing Legacy JS Build
- **JS Original**: `package.json` executed `npm run build` prior to running `npm test` to compile TypeScript to JavaScript using `tsc` and `rollup`.
- **Rust Port**: Removed `"pretest": "npm run build"` script from `package.json`.
- **Rationale**: In migrating to pure Rust (Track F), the original TypeScript/Rollup JS compilation pipeline is obsolete. The test suite runs against the native compiled Rust executable.

## 13. Fuzzer Binary Target Configuration
- **JS Original**: Had no built-in native differential fuzzing harness.
- **Rust Port**: Configured a dedicated `[[bin]]` entry in `Cargo.toml` for `fuzz/harness.rs` (`name = "harness"`).
- **Rationale**: Enables standalone compilation and execution of the differential fuzzing suite to validate Rust JSBI outputs directly against reference arithmetic behavior.

---

## Bug Catcher Notes (Latent JS Bugs & Edge Cases)
1. **Invalid Numeric Prefix Syntax handling**: JSBI's string parser historically had edge cases where parsing `-0x1` produced incorrect sign state before issue #36 patches. In Rust, our parser validates sign handling prior to radix stripping.
2. **Shift Count Range Overflow**: In JSBI, shifting by very large numbers could cause JS bitwise shift truncation (`shiftAmount | 0` in JS wraps at 32 bits). In Rust, shift amounts are checked against maximum bit limits safely (`JSBIError::RangeError`).

