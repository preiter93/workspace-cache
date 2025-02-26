# workspace-cache

A Rust CLI tool that optimizes dependency caching for Rust workspaces, primarily for Docker builds.

## Installation

```sh
cargo install --path .
```

## Usage

### Commands

#### `workspace-cache deps`

Generates a minimal workspace that contains only dependency information so Cargo builds dependencies but not the application code.

```sh
workspace-cache deps
```

This creates a `.workspace-cache/` directory containing:
- `Cargo.toml` - Minimal manifest with all external dependencies
- `Cargo.lock` - Copy of the original lock file (if present)
- `src/main.rs` - Dummy main file

#### `workspace-cache build`

Builds the real workspace using the already cached dependencies.

```sh
workspace-cache build
workspace-cache build --release
```

## Docker Example

```dockerfile
FROM rust:1.76 AS builder

WORKDIR /app

# Install workspace-cache
COPY workspace-cache /usr/local/bin/workspace-cache

# Copy manifests only (for dependency caching layer)
COPY Cargo.toml Cargo.lock ./
COPY crates/app/Cargo.toml ./crates/app/Cargo.toml

# Create dummy source files so cargo can parse the workspace
RUN mkdir -p crates/app/src && echo "fn main() {}" > crates/app/src/main.rs

# Generate minimal workspace and build dependencies
RUN workspace-cache deps
RUN cd .workspace-cache && cargo build --release

# Remove dummy sources
RUN rm -rf crates/app/src

# Copy actual source code
COPY crates/app/src ./crates/app/src

# Build the real workspace (dependencies already cached)
RUN workspace-cache build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/app
CMD ["app"]
```

## How It Works

1. **Dependency Phase**: The `deps` command reads `cargo metadata` to extract all external dependencies from workspace members. It generates a minimal single-crate project that depends on all these crates, allowing Cargo to build and cache them.

2. **Build Phase**: The `build` command simply runs `cargo build --workspace` on the real workspace. Since dependencies are already compiled and cached, only the actual application code needs to be built.

This two-phase approach allows Docker to cache the dependency layer separately from the source code layer, dramatically speeding up rebuilds when only source files change.

## Example Workspace

See `example-workspace/` for a working example:

```
example-workspace/
├── Cargo.toml
├── Dockerfile
└── crates/
    └── app/
        ├── Cargo.toml
        └── src/
            └── main.rs
```

Test it:

```sh
cd example-workspace
../target/release/workspace-cache deps
cd .workspace-cache
cargo build
```
