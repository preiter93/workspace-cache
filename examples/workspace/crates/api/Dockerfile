FROM rust:1.76 AS builder

WORKDIR /app

# Copy workspace-cache binary (pre-built)
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Copy only Cargo manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/common/Cargo.toml ./crates/common/Cargo.toml

# Create minimal source stubs so cargo can parse the workspace
RUN mkdir -p crates/api/src crates/common/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    echo "" > crates/common/src/lib.rs

# Generate dependency workspace for api + common only
RUN workspace-cache deps -p api -p common

# Build dependencies (this layer gets cached!)
RUN cd .workspace-cache && cargo build --release

# Copy real source code
COPY crates/api/src ./crates/api/src
COPY crates/common/src ./crates/common/src

# Build the api service (deps already cached - fast!)
RUN workspace-cache build --release -p api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api /usr/local/bin/api
CMD ["api"]
