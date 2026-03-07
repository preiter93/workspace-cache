FROM rust:1.76 AS builder

WORKDIR /app

# Copy workspace-cache binary (pre-built)
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Copy only Cargo manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/worker/Cargo.toml ./crates/worker/Cargo.toml
COPY crates/common/Cargo.toml ./crates/common/Cargo.toml

# Create minimal source stubs so cargo can parse the workspace
RUN mkdir -p crates/worker/src crates/common/src && \
    echo "fn main() {}" > crates/worker/src/main.rs && \
    echo "" > crates/common/src/lib.rs

# Generate dependency workspace for worker + common only
RUN workspace-cache deps -p worker -p common

# Build dependencies (this layer gets cached!)
RUN cd .workspace-cache && cargo build --release

# Copy real source code
COPY crates/worker/src ./crates/worker/src
COPY crates/common/src ./crates/common/src

# Build the worker service (deps already cached - fast!)
RUN workspace-cache build --release -p worker

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/worker /usr/local/bin/worker
CMD ["worker"]
