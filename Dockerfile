# This Dockerfile builds the distribution image for cargo-skeleton.
# It is not an example of how to use cargo-skeleton.
FROM --platform=$BUILDPLATFORM docker.io/rust:1.80.1-slim-bookworm AS build

RUN apt-get update && apt-get install -y \
  g++-x86-64-linux-gnu libc6-dev-amd64-cross \
  g++-aarch64-linux-gnu libc6-dev-arm64-cross && \
  rm -rf /var/lib/apt/lists/*

RUN rustup target add \
  x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

RUN rustup toolchain install \
  stable-x86_64-unknown-linux-gnu stable-aarch64-unknown-linux-gnu

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
  CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
  CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++ \
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
  CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
  CARGO_INCREMENTAL=0

WORKDIR /usr/src/cargo-skeleton

COPY . .

FROM --platform=$BUILDPLATFORM build AS build-amd64

RUN cargo install --target x86_64-unknown-linux-gnu --path .
 
FROM --platform=$BUILDPLATFORM build AS build-arm64

RUN cargo install --target aarch64-unknown-linux-gnu --path .

FROM --platform=amd64 scratch AS final-amd64

COPY --from=build-amd64 /usr/local/cargo/bin/cargo-skeleton /

FROM --platform=arm64 scratch AS final-arm64

COPY --from=build-arm64 /usr/local/cargo/bin/cargo-skeleton /

FROM final-${TARGETARCH}