FROM rust:1.76 AS builder

WORKDIR /app

# Copy workspace-cache binary (pre-built)
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Copy only manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/common/Cargo.toml ./crates/common/Cargo.toml
COPY crates/worker/Cargo.toml ./crates/worker/Cargo.toml

# Create dummy source files so cargo can parse the workspace
RUN mkdir -p crates/common/src && echo "pub struct Message;" > crates/common/src/lib.rs
RUN mkdir -p crates/worker/src && echo "fn main() {}" > crates/worker/src/main.rs

# Generate minimal workspace for worker's dependencies only
RUN workspace-cache deps -p worker -p common

# Build dependencies (this layer gets cached)
RUN cd .workspace-cache && cargo build --release

# Remove dummy sources
RUN rm -rf crates/common/src crates/worker/src

# Copy actual source code
COPY crates/common/src ./crates/common/src
COPY crates/worker/src ./crates/worker/src

# Build only the worker package
RUN workspace-cache build --release -p worker

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/worker /usr/local/bin/worker
CMD ["worker"]
