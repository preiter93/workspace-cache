FROM rust:1.94-bookworm AS base
WORKDIR /app
COPY --from=workspace-cache /workspace-cache /usr/local/bin/workspace-cache

# Prepare dependencies
FROM base AS planner
COPY . .
RUN workspace-cache deps -p api

# Build dependencies
FROM base AS builder
COPY --from=planner /app/.workspace-cache ./.workspace-cache
COPY --from=planner /app/Cargo.lock ./Cargo.lock
RUN cd .workspace-cache && cargo build --release

# Build the binary
COPY Cargo.toml Cargo.lock ./
COPY crates/api crates/api
COPY crates/common crates/common
COPY crates/utils crates/utils
RUN workspace-cache build --release -p api

# Runtime
FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/api /usr/local/bin/api
ENTRYPOINT ["/usr/local/bin/api"]
