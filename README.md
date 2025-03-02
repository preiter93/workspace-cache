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
workspace-cache deps -p api -p common
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

#### `workspace-cache deps-of`

Shows which workspace members a package depends on (transitively). Useful for determining which packages to pass to `-p`.

```sh
# Show all workspace members that api depends on
workspace-cache deps-of -p api
# Output:
# api
# common

# Use it with deps command
workspace-cache deps $(workspace-cache deps-of -p api | xargs -I{} echo "-p {}")
```

## How It Works

1. **Dependency Phase**: The `deps` command creates a mirror of your workspace with stub source files (`fn main() {}` or `// stub`). This minimal workspace has the same structure and dependencies as your real workspace, allowing Cargo to compile and cache all dependencies.

2. **Build Phase**: The `build` command runs `cargo build` on the real workspace. When using a shared target directory, dependencies are already compiled and cached, so only your application code needs to be built.

This approach preserves your exact workspace structure, including:
- Workspace dependencies (`[workspace.dependencies]`)
- Path dependencies between members
- All features and configuration

## Microservice Example

Consider a workspace with multiple microservices:

```
my-platform/
├── Cargo.toml
├── Cargo.lock
└── crates/
    ├── api/           # HTTP API service (depends on axum)
    ├── worker/        # Background worker (depends on tokio)
    └── common/        # Shared library (depends on serde)
```

### Building only the API service

```sh
# First, find which workspace members api depends on
workspace-cache deps-of -p api
# Output: api, common

# Generate deps for api and its workspace dependencies
workspace-cache deps -p api -p common

# Build the dependency cache (use shared target dir)
cd .workspace-cache && CARGO_TARGET_DIR=../target cargo build --release && cd ..

# Build only the api binary (deps already cached!)
workspace-cache build --release -p api
```

## Docker Example

The tool eliminates manual dummy file creation in Dockerfiles:

```dockerfile
FROM rust:1.76 AS builder

WORKDIR /app

# Copy workspace-cache binary
COPY --from=workspace-cache-image /workspace-cache /usr/local/bin/workspace-cache

# Copy the entire workspace (manifests + source)
COPY . .

# Generate minimal workspace for api + common
RUN workspace-cache deps -p api -p common

# Build dependencies only (this layer gets cached!)
RUN cd .workspace-cache && cargo build --release

# Build the real api service
RUN workspace-cache build --release -p api

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/api /usr/local/bin/api
CMD ["api"]
```

### Optimized Docker Caching

For better layer caching, copy only manifests first:

```dockerfile
FROM rust:1.76 AS builder

WORKDIR /app

COPY --from=workspace-cache-image /workspace-cache /usr/local/bin/workspace-cache

# Copy only Cargo files first (for better caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/common/Cargo.toml ./crates/common/Cargo.toml

# Create minimal source stubs (required for cargo to parse workspace)
RUN mkdir -p crates/api/src crates/common/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    echo "" > crates/common/src/lib.rs

# Generate dependency workspace and build deps
RUN workspace-cache deps -p api -p common
RUN cd .workspace-cache && cargo build --release

# Now copy real source and build
COPY crates/api/src ./crates/api/src
COPY crates/common/src ./crates/common/src
RUN workspace-cache build --release -p api

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/api /usr/local/bin/api
CMD ["api"]
```

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
    └── common/
        ├── Cargo.toml
        └── src/lib.rs
```

Test it:

```sh
# Build workspace-cache
cargo build --release

# Go to example workspace
cd example-workspace

# Generate deps for worker service only
../target/release/workspace-cache deps -p worker -p common

# Build dependency cache (with shared target dir)
cd .workspace-cache && CARGO_TARGET_DIR=../target cargo build --release && cd ..

# Build the worker (deps already cached - very fast!)
../target/release/workspace-cache build --release -p worker

# Run it
./target/release/worker
```
