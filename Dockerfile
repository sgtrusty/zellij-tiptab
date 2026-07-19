# ==========================================
# STAGE 0: Base
# ==========================================
FROM rust:1.96.1-trixie AS base

RUN rustup target add wasm32-wasip1

# ==========================================
# STAGE 1: Non-Root Builder
# ==========================================
FROM base AS builder

# 1. Create the non-root user and group
RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -m -s /bin/bash appuser

# 2. Tell rustup to use the system-wide toolchain, but let cargo write to the user's home
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/home/appuser/.cargo \
    PATH=/usr/local/cargo/bin:/home/appuser/.cargo/bin:$PATH

# 3. Switch to the non-root user
USER appuser
WORKDIR /app

# 4. Copy the crate (now at the repo root)
COPY --chown=appuser:appgroup Cargo.toml Cargo.lock ./
COPY --chown=appuser:appgroup src ./src
COPY --chown=appuser:appgroup .cargo ./.cargo

# 5. Build for WASI in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry,uid=10001,gid=10001 \
    --mount=type=cache,target=/app/target,uid=10001,gid=10001 \
    cargo build --release --target wasm32-wasip1 && \
    cp target/wasm32-wasip1/release/zellij-tiptab.wasm /app/zellij-tiptab.wasm

# ==========================================
# STAGE 3: Lint
# ==========================================
FROM base AS lint

RUN rustup component add clippy

WORKDIR /app
COPY --from=builder /app/Cargo.toml /app/Cargo.lock ./
COPY --from=builder /app/src ./src
COPY --from=builder /app/.cargo ./.cargo

RUN cargo clippy --target wasm32-wasip1 -- -D warnings

# ==========================================
# STAGE 2: Non-Root Export
# ==========================================
FROM scratch AS export

COPY --from=builder /app/zellij-tiptab.wasm /
