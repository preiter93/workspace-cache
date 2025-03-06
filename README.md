# workspace-cache

A Rust CLI tool that optimizes dependency caching for Rust workspaces, primarily for Docker builds.

## Installation

```sh
cargo install --path .
```

## Usage

### Commands

#### `workspace-cache deps`

Generates a minimal workspace copy with stub source files, preserving the original workspace structure. This allows Cargo to build and cache all dependencies.

```sh
# Cache dependencies for entire workspace
workspace-cache deps

# Cache dependencies for specific package(s) only
workspace-cache deps -p api
```

This creates a `.workspace-cache/` directory containing:
- `Cargo.toml` - Copy of workspace manifest (filtered to selected members)
- `Cargo.lock` - Copy of the original lock file (if present)
- `crates/*/Cargo.toml` - Original member manifests
- `crates/*/src/*.rs` - Stub source files

#### `workspace-cache build`

Builds the real workspace using the already cached dependencies.

```sh
# Build entire workspace
workspace-cache build

# Build specific package(s)
workspace-cache build -p api

# Release build
workspace-cache build --release -p api
```

#### `workspace-cache resolve`

Resolves which workspace members a package depends on (transitively). The `deps` command does this automatically, but this is useful for scripting or understanding your dependency graph.

```sh
workspace-cache resolve -p api
# api
# common
# utils
```

#### `workspace-cache dockerfile`

Generates a Dockerfile for a package, automatically including all required workspace dependencies.

```sh
# Output to stdout
workspace-cache dockerfile -p api

# Output to file
workspace-cache dockerfile -p api -o Dockerfile

# Custom base and runtime images
workspace-cache dockerfile -p api \
    --base-image rust:1.80-alpine \
    --runtime-image alpine:3.20
```

## How It Works

1. **Dependency Phase**: The `deps` command creates a mirror of your workspace with stub source files. This minimal workspace has the same structure and dependencies as your real workspace, allowing Cargo to compile and cache all dependencies.

2. **Build Phase**: The `build` command runs `cargo build` on the real workspace. Since dependencies are already compiled and cached, only your application code needs to be built.

This approach preserves your exact workspace structure, including:
- Workspace dependencies (`[workspace.dependencies]`)
- Path dependencies between members
- All features and configuration

## Docker Example

Generate a Dockerfile automatically:

```sh
workspace-cache dockerfile -p api -o Dockerfile
```

This produces:

```dockerfile
FROM rust:1.87-bookworm AS base
WORKDIR /app
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Prepare minimal workspace
FROM base AS planner
COPY . .
RUN workspace-cache deps -p api

# Build dependencies
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build --release

# Build the binary
FROM deps AS builder
RUN rm -rf crates/api/src crates/common/src crates/utils/src
COPY crates/api crates/api
COPY crates/common crates/common
COPY crates/utils crates/utils
RUN cargo build --release -p api

# Runtime
FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/api /usr/local/bin/api
ENTRYPOINT ["/usr/local/bin/api"]
```

**Stages:**
1. **base** - installs workspace-cache
2. **planner** - generates `.workspace-cache/` with filtered workspace
3. **deps** - builds dependencies (cached as long as deps don't change)
4. **builder** - copies real source, builds binary
5. **runtime** - minimal image with just the binary

## Testing

```sh
cargo test
```

## Example Workspace

See `example-workspace/` for a working microservice example:

```
example-workspace/
├── Cargo.toml
└── crates/
    ├── api/
    │   ├── Cargo.toml
    │   ├── Dockerfile
    │   └── src/main.rs
    ├── worker/
    │   ├── Cargo.toml
    │   ├── Dockerfile
    │   └── src/main.rs
    ├── common/
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── utils/
        ├── Cargo.toml
        └── src/lib.rs
```

Test it:

```sh
# Build workspace-cache
cargo build --release

# Go to example workspace
cd example-workspace

# Generate deps for api (auto-resolves common and utils)
../target/release/workspace-cache deps -p api

# Build dependency cache
cd .workspace-cache && cargo build --release && cd ..

# Build the api (deps already cached - very fast!)
../target/release/workspace-cache build --release -p api

# Run it
./target/release/api
```
