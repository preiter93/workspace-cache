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
