FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Extra CLI/build tooling commonly useful for Rust development and debugging.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*
