# Cargo Skeleton

⚠️ WIP ⚠️

Cargo Skeleton is a Cargo subcommand that generates the bare minimum directory tree ("skeleton") necessary to compile a workspace's dependencies.

This is useful when building Docker images.

See [Cargo issue #2644](https://github.com/rust-lang/cargo/issues/2644) for a description of the problem this solves and a long list of workarounds.

## Installation

The command is distributed via a Docker image. The image is only used to distribute the binary, and cannot be used to execute it. To use the binary, copy it into your Docker image of choice.

The binary is available under `/cargo-skeleton` and should be added to the cargo bin directory. For example:

```dockerfile
FROM docker.io/rust:1.80.1-slim-bookworm
COPY --from=docker.io/cargo-skeleton:latest /cargo-skeleton /usr/local/cargo/bin/
```

## Usage

The following Dockerfile demonstrates how to build a single package's binary using Cargo Skeleton:

```dockerfile
FROM docker.io/rust:1.80.1-slim-bookworm AS base

# Initial setup
ENV CARGO_INCREMENTAL=0
WORKDIR /usr/src/app

# Create a new stage to build the skeleton.
# This stage will run a lot so don't put anything else in here.
FROM base AS skeleton-builder

# Copy the `cargo skeleton` command from the distribution image
COPY --from=cargo-skeleton:latest /cargo-skeleton /usr/local/cargo/bin/

# Copy the sources into the workspace.
COPY . .

# Create the skeleton archive.
# This will create a file `skeleton.tar` in the `WORKDIR` by default.
RUN cargo skeleton create

# Start a new builder stage.
FROM base AS builder

# Copy the `cargo skeleton` command from the distribution image
COPY --from=cargo-skeleton:latest /cargo-skeleton /usr/local/cargo/bin/
# Copy the skeleton archive from the previous stage
COPY --from=skeleton-builder /usr/src/app/skeleton.tar /usr/src/app/skeleton.tar

# Unpack the skeleton into the current directory.
# This will overwrite any existing files!
RUN cargo skeleton unpack

# Build the skeleton, compiling any dependencies.
# Anything after `--` is passed directly to `cargo build`.
# Put the `--package`, `--exclude`, and `--all` flags before the `--`.
RUN cargo skeleton build -- --release --locked

# Copy in the source files.
# This will invalidate the docker cache when any file changes.
COPY . .

# Now execute the actual Cargo build.
# Any flags used to build the skeleton should be passed here too.
RUN cargo build --release --locked

# Any subsequent steps here.
# Typically you would start a third stage for the runtime image.
```

If you are using this with buildx caching, make sure to pass [`mode=max`](https://docs.docker.com/build/cache/backends/#cache-mode).

### How it works

The skeleton is just a tar archive. Every file that affects compilation is added as-is. Every target (`src/lib.rs`, `src/main.rs`, etc.) is replaced with a "stub" file. The stub will cause a compilation error if it's actually compiled; it only exists to make Cargo happy. The archive will have the same checksum unless one of the files changes.

The skeleton archive is built in a Docker stage, then copied to the next stage where it is unpacked. The stage that builds the archive will have to run any time a source file changes, but the archive will still have the same checksum. Since the checksum does not change, the instructions in the next stage are still cached.

After unpacking the archive the dependencies for a given package are built. This uses a custom command because Cargo does not have a flag to only build dependencies. All the command does is pass a bunch of `--package` flags to `cargo build`. The list of dependencies is saved to a `Skeleton.lock` file when the archive is first built and read by the build command.

### Alternatives

#### Manual stubs

Like [this](https://stackoverflow.com/questions/58473606/cache-rust-dependencies-with-docker-build). It sort of works, but it's very hard to maintain in a workspace with multiple packages. The dummy files can confuse the Cargo cache mtime detection, resulting in a successful build of an empty binary.

#### Docker cache mounts

If you are always building on the same machine, you can use cache mounts instead. Using cache mounts lets you use Cargo like you normally would, and you don't need to do any of this.

The big downside of cache mounts is they don't work with the [buildx cache](https://docs.docker.com/build/cache/backends/). In a CI environment you don't typically get to use the same machine twice, and you need to use a remote cache for fast builds.

#### Cargo Chef

[Cargo Chef](https://github.com/LukeMathWalker/cargo-chef) inspired the design of Cargo Skeleton, so the approaches are very similar. This project was created to try a more "hands off" approach, letting Cargo do more of the work, in hopes that doing so would avoid edge cases that break compilation. For example:

- Any file that affects compilation is added to the archive as-is
- Build args are forward to Cargo verbatim
- Cargo Metadata is used for as much resolution as possible; manifests are never parsed
- Stub files are never compiled so that Cargo's cache doesn't get confused

Cargo Chef has been used for much longer with a wide variety of projects and will likely handle many edge cases that this project does not. There are situations where Cargo Skeleton will re-compile dependencies but Cargo Chef would not.