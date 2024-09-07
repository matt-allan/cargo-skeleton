FROM docker.io/rust:1.80.1-slim-bookworm AS base

ENV CARGO_INCREMENTAL=0

WORKDIR /usr/src/cargo-skeleton

FROM base AS skeleton-builder

COPY --from=cargo-skeleton:latest /cargo-skeleton /usr/local/cargo/bin/

COPY . .

RUN cargo skeleton

FROM base AS builder

COPY --from=skeleton-builder /usr/src/cargo-skeleton/skeleton.tar /usr/src/cargo-skeleton/skeleton.tar

RUN tar xf skeleton.tar

RUN cargo build --release --locked

COPY . .

# ENV CARGO_LOG=cargo::core::compiler::fingerprint=trace

RUN cargo install --locked --path .

ENTRYPOINT ["cargo", "skeleton"]