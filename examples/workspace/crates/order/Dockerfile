# Stage 1: Install workspace-cache tool
FROM rust:1.94-bookworm AS base
WORKDIR /app
RUN cargo install workspace-cache@0.1.0-alpha.2


# Stage 2: Generate minimal workspace with stub sources
FROM base AS planner
COPY . .
RUN workspace-cache deps --bin order --fast

# Stage 3: Build dependencies only (cached until Cargo.toml/Cargo.lock change)
FROM base AS deps
COPY --from=planner /app/.workspace-cache .
RUN cargo build --release

# Stage 4: Build the actual binary with real source code
FROM deps AS builder
RUN rm -rf crates/user/src crates/pkg-a/src crates/pkg-b/src crates/order/src
COPY crates/user crates/user
COPY crates/pkg-a crates/pkg-a
COPY crates/pkg-b crates/pkg-b
COPY crates/order crates/order
RUN cargo clean --release -p user -p pkg_a -p pkg_b -p order
RUN cargo build --release --bin order

# Stage 5: Minimal runtime image
FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/order /usr/local/bin/order
ENTRYPOINT ["/usr/local/bin/order"]
