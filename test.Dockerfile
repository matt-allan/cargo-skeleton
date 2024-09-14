FROM docker.io/rust:1.80.1-slim-bookworm AS base

ENV CARGO_INCREMENTAL=0

WORKDIR /usr/src/cargo-skeleton

FROM base AS skeleton-builder

COPY --from=cargo-skeleton:latest /cargo-skeleton /usr/local/cargo/bin/

COPY . .

ENV RUST_LOG=debug

RUN cargo skeleton create

FROM base AS builder

COPY --from=cargo-skeleton:latest /cargo-skeleton /usr/local/cargo/bin/
COPY --from=skeleton-builder /usr/src/cargo-skeleton/skeleton.tar /usr/src/cargo-skeleton/skeleton.tar

RUN cargo skeleton unpack

RUN cargo skeleton build

COPY . .

# ENV CARGO_LOG=cargo::core::compiler::fingerprint=trace

RUN cargo build --release --locked

ENTRYPOINT ["cargo", "run" "--"]