# workspace-cache

[![Crate IO](https://img.shields.io/crates/v/workspace-cache?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44)](https://crates.io/crates/workspace-cache) ![Crates.io Downloads](https://img.shields.io/crates/d/workspace-cache?style=flat-square) [![Continuous Integration](https://github.com/preiter93/workspace-cache/actions/workflows/ci.yml/badge.svg)](https://github.com/preiter93/workspace-cache/actions/workflows/ci.yml) [![Deps Status](https://deps.rs/repo/github/preiter93/workspace-cache/status.svg?style=flat-square)](https://deps.rs/repo/github/preiter93/workspace-cache) [![License](https://img.shields.io/crates/l/workspace-cache?style=flat-square&color=09bd66)](./LICENSE)

Like [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) but focused on **Rust workspaces with multiple binaries** (microservices). Generates optimized Dockerfiles with proper layer caching.

## Installation

```sh
cargo install workspace-cache
```

Or install from git for the latest development version:

```sh
cargo install --git https://github.com/preiter93/workspace-cache workspace-cache
```

## Quick Start

Generate a Dockerfile for your service:

```sh
workspace-cache dockerfile --bin user -o Dockerfile
```

This produces an optimized multi-stage Dockerfile:

```dockerfile
# Stage 1: Install workspace-cache tool
FROM rust:1.94-bookworm AS base
WORKDIR /app
RUN cargo install workspace-cache@0.1.0

# Stage 2: Generate minimal workspace with stub sources
FROM base AS planner
COPY . .
RUN workspace-cache deps --bin user

# Stage 3: Build dependencies only (cached until Cargo.toml/Cargo.lock change)
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build --release

# Stage 4: Build the actual binary with real source code
FROM deps AS builder
RUN rm -rf crates/user crates/common
COPY crates/user crates/user
COPY crates/common crates/common
RUN cargo clean --release -p user -p common
RUN cargo build --release --bin user

# Stage 5: Minimal runtime image
FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/user /usr/local/bin/user
ENTRYPOINT ["/usr/local/bin/user"]
```

## Build & Run

```sh
docker build -f Dockerfile -t user .
docker run --rm user
```

## How It Works

1. **planner** - Creates a minimal workspace with stub sources for dependency resolution
2. **deps** - Builds dependencies (cached until Cargo.toml/Cargo.lock changes)
3. **builder** - Copies real source and builds binary
4. **runtime** - Minimal image with just the binary

When source files change but dependencies don't, Docker skips the `deps` stage entirely.

### Workspace Optimization

workspace-cache analyzes your workspace dependency graph and includes only the necessary members for your binary. This enables isolated builds where changes to unrelated workspace members don't invalidate your cache. For example, if you have `user`, `order`, and `payment` services, changing the `payment` service won't trigger rebuilds of `user` or `order` containers.

This optimization applies to both the dependency build stage and the binary build stage. The generated Dockerfile only copies workspace members that your binary depends on, ensuring that changes to independent members don't invalidate Docker's layer cache at any stage of the build.

## Comparison with cargo-chef

| Feature | cargo-chef | workspace-cache |
|---------|------------|-----------------|
| Cached dependencies | ✅ | ✅ |
| Workspace optimization (dependencies) | ✅ | ✅ |
| Fast dependency build | ❌ | ✅ ([`--fast`](#fast-mode)) |
| Workspace optimization (binary) | ❌ | ✅ ([`Optimization`](#workspace-optimization)) |
| Dockerfile generation | ❌ | ✅ ([`Dockerfile`](#usage)) |
| GitHub Actions | ❌ | ✅ ([`CI Usage`](#ci-usage-without-docker)) |
| Get dependent workspace members | ❌ | ✅ ([`members`](#show-workspace-members)) |

## Usage

The main command is `dockerfile`. It generates an optimized Dockerfile for your binary:

```sh
workspace-cache dockerfile --bin <binary> [OPTIONS]
```

Options:
- `--bin <binary>` - Binary to build (required)
- `--profile <profile>` - Build profile: `release` or `debug` (default: `release`)
- `-o, --output <path>` - Output path (default: stdout)
- `--base-image <image>` - Base image for build stages (default: `rust:1.94-bookworm`)
- `--runtime-image <image>` - Runtime image (default: `debian:bookworm-slim`)
- `--tool-version <version>` - Version of workspace-cache to install (default: current version)
- `--from-git` - Install workspace-cache from git instead of crates.io
- `--fast` - Fast mode: skip dependency resolution for faster generation

Examples:
```sh
# Generate release Dockerfile (default)
workspace-cache dockerfile --bin user -o Dockerfile

# Generate debug Dockerfile
workspace-cache dockerfile --bin user --profile debug -o Dockerfile.debug

# Use a specific version
workspace-cache dockerfile --bin user --tool-version 0.1.0 -o Dockerfile

# Install from git (latest dev version)
workspace-cache dockerfile --bin user --from-git -o Dockerfile

# Custom base image
workspace-cache dockerfile --bin user --base-image rust:1.80-alpine -o Dockerfile
```

### Fast Mode

Use `--fast` to skip dependency resolution. This results in a less optimized cache, but speeds up Docker builds as long as no dependencies have changed.

```sh
workspace-cache dockerfile --bin user --fast -o Dockerfile
```

## CI Usage (without Docker)

### GitHub Action (Recommended)

The simplest way to use workspace-cache in CI is with the provided composite actions:

```yaml
- name: Install workspace-cache
  uses: preiter93/workspace-cache/.github/actions/install-workspace-cache@main

- name: Build
  uses: preiter93/workspace-cache/.github/actions/build-workspace@main
  with:
    binary: user
    working-directory: services

- name: Run tests
  working-directory: services
  env:
    CARGO_TARGET_DIR: .workspace-cache/target
  run: cargo test -p user --verbose

- name: Clippy
  working-directory: services
  env:
    CARGO_TARGET_DIR: .workspace-cache/target
  run: cargo clippy -p user -- -D warnings
```

**Important:** Set `CARGO_TARGET_DIR: .workspace-cache/target` when running cargo commands after build to reuse artifacts.

See the action READMEs for more options:
- [install-workspace-cache](.github/actions/install-workspace-cache/README.md)
- [build-workspace](.github/actions/build-workspace/README.md)

### Complete Example with Matrix Strategy

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: [user, order, payment]
    steps:
      - uses: actions/checkout@v6
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      
      - name: Install workspace-cache
        uses: preiter93/workspace-cache/.github/actions/install-workspace-cache@main
      
      - name: Build
        uses: preiter93/workspace-cache/.github/actions/build-workspace@main
        with:
          binary: ${{ matrix.service }}
          working-directory: services
      
      - name: Run tests
        working-directory: services
        env:
          CARGO_TARGET_DIR: .workspace-cache/target
        run: cargo test -p ${{ matrix.service }} --verbose
      
      - name: Clippy
        working-directory: services
        env:
          CARGO_TARGET_DIR: .workspace-cache/target
        run: cargo clippy -p ${{ matrix.service }} -- -D warnings
```

### Manual Setup

You can also set up workspace-cache manually for more control:

```yaml
- name: Install workspace-cache
  run: cargo install workspace-cache

- name: Generate minimal workspace
  run: workspace-cache deps --bin user

- name: Get cache key for dependencies
  id: cache-key
  run: |
    HASH="${{ hashFiles('.workspace-cache/Cargo.lock') }}"
    echo "key=${{ runner.os }}-workspace-cache-deps-${HASH}" >> $GITHUB_OUTPUT

- name: Cache dependencies
  uses: actions/cache@v5
  with:
    path: .workspace-cache/target
    key: ${{ steps.cache-key.outputs.key }}
    restore-keys: |
      ${{ runner.os }}-workspace-cache-deps-

- name: Build dependencies (cached when Cargo.lock unchanged)
  run: cargo build --release
  working-directory: .workspace-cache

- name: Copy real sources
  run: |
    workspace-cache members --bin user | while read path name; do
      rm -rf .workspace-cache/$path
      cp -r $path .workspace-cache/$path
    done

- name: Build binary
  run: |
    PACKAGES=$(workspace-cache members --bin user | awk '{print "-p " $2}' | tr '\n' ' ')
    cargo clean --release $PACKAGES
    cargo build --release --bin user
  working-directory: .workspace-cache
```

The cache key is based on the generated `.workspace-cache/Cargo.lock`, so dependencies are only rebuilt when they change. On cache hits, the dependency build step completes in seconds.

## Other Commands

The following commands are mainly for debugging or understanding how the tool works internally.

### Generate Minimal Workspace

```sh
workspace-cache deps [OPTIONS]
```

Options:
- `--bin <binary>` - Only include dependencies for specific binary/binaries
- `-o, --output <dir>` - Output directory (default: `.workspace-cache`)
- `--fast` - Fast mode: skip dependency resolution

Examples:
```sh
# Generate for all workspace binaries
workspace-cache deps

# Generate for a specific binary
workspace-cache deps --bin user

# Generate for multiple binaries
workspace-cache deps --bin user --bin order

# Custom output directory
workspace-cache deps --bin user -o my-cache
```

### Show Workspace Members

```sh
workspace-cache members --bin <binary>
```

Shows which workspace members a binary depends on, with their paths and names:

```
$ workspace-cache members --bin user
crates/pkg-a pkg_a
crates/pkg-b pkg_b
crates/user user
```

This output can be used in scripts to dynamically copy sources or generate package lists.

### Build Workspace

```sh
workspace-cache build [OPTIONS]
```

Options:
- `--bin <binary>` - Only build specific binary/binaries
- `--release` - Build in release mode

Examples:
```sh
# Build all binaries
workspace-cache build

# Build specific binary in release mode
workspace-cache build --bin user --release
```

## Testing

Run unit tests:

```sh
cargo test
```

Test locally in your workspace:

```sh
# Generate minimal workspace for a binary
workspace-cache deps --bin user

# Build dependencies
cd .workspace-cache
cargo build --release

# Copy real sources and build (deps are cached)
rm -rf crates/user crates/common
cp -r ../crates/user crates/user
cp -r ../crates/common crates/common
cargo build --release --bin user
```

Note: This mirrors how the generated Dockerfile works. The key is building
the final binary from within `.workspace-cache/` after copying real sources.

## License

MIT
