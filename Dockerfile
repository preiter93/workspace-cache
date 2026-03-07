FROM rust:1.94-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM scratch AS export
COPY --from=builder /app/target/release/workspace-cache /workspace-cache
