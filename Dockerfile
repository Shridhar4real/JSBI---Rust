# Multi-stage Docker build for JSBI Rust Port
FROM rust:1.75-slim as builder

WORKDIR /usr/src/jsbi-rs
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /usr/src/jsbi-rs/target/release/jsbi-cli /usr/local/bin/jsbi-cli

ENTRYPOINT ["jsbi-cli"]
