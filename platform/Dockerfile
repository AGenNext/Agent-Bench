# Multi-stage build for the Agent-Bench platform API.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY platform/Cargo.toml platform/Cargo.toml
COPY platform/src platform/src
COPY platform/migrations platform/migrations
WORKDIR /src/platform
# Build the full server (embedded + remote SurrealDB engines).
RUN cargo build --release --features server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/platform/target/release/agentbench-platform /usr/local/bin/
EXPOSE 8080
ENV AGENTBENCH_ADDR=0.0.0.0:8080
ENTRYPOINT ["/usr/local/bin/agentbench-platform"]
