FROM rust:1.76 AS builder

WORKDIR /app

# Install workspace-cache
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Copy only manifest files first
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/common/Cargo.toml ./crates/common/Cargo.toml

# Create dummy source files so cargo can parse manifests
RUN mkdir -p crates/api/src crates/common/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    echo "" > crates/common/src/lib.rs

# Generate dependency cache for api (includes common as path dep)
RUN workspace-cache deps -p api -p common

# Build dependencies only
RUN cd .workspace-cache && cargo build --release

# Remove dummy sources
RUN rm -rf crates/api/src crates/common/src

# Copy real source code
COPY crates/api/src ./crates/api/src
COPY crates/common/src ./crates/common/src

# Build the api service
RUN workspace-cache build --release -p api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api /usr/local/bin/api
CMD ["api"]
