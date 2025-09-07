# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.89.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app

# OS deps for building
RUN apt-get update -y && apt-get upgrade -y && \
    apt-get install --no-install-recommends -y \
      pkg-config \
      libssl-dev \
      libpq-dev \
      ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/* && \
    rustup toolchain install nightly && \
    rustup update nightly && \
    rustup default nightly

# Build only the selected example (defaults to periodic_alert)
ARG EXAMPLE_BIN=periodic_alert
ENV EXAMPLE_BIN=${EXAMPLE_BIN}

# Leverage caching by copying manifests first
COPY Cargo.toml ./
COPY rust-toolchain.toml ./
COPY greenhouse_core/Cargo.toml greenhouse_core/Cargo.toml
COPY greenhouse_macro/Cargo.toml greenhouse_macro/Cargo.toml
COPY services/auth_service/Cargo.toml services/auth_service/Cargo.toml
COPY services/data_storage_service/Cargo.toml services/data_storage_service/Cargo.toml
COPY services/device_service/Cargo.toml services/device_service/Cargo.toml
COPY services/scripting_service/Cargo.toml services/scripting_service/Cargo.toml
COPY api/web/Cargo.toml api/web/Cargo.toml
COPY api/script/Cargo.toml api/script/Cargo.toml
COPY examples/Cargo.toml examples/Cargo.toml

# Then copy sources
COPY . .

# Build the requested example from the examples crate
RUN cargo build --release -p examples --example ${EXAMPLE_BIN} && \
    cp target/release/examples/${EXAMPLE_BIN} /${EXAMPLE_BIN}

# ---- Runtime image ----
FROM debian:bookworm-slim AS final
ARG EXAMPLE_BIN=periodic_alert
ENV EXAMPLE_BIN=${EXAMPLE_BIN}

# Create non-root user
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "10001" \
  appuser

# Runtime deps
RUN apt-get update -y && apt-get upgrade -y && \
  apt-get install --no-install-recommends -y \
        libpq5 \
        ca-certificates \
        curl \
  && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy built binary
COPY --from=builder /${EXAMPLE_BIN} /usr/local/bin/${EXAMPLE_BIN}
RUN chown appuser /usr/local/bin/${EXAMPLE_BIN} && \
  chmod +x /usr/local/bin/${EXAMPLE_BIN}

USER appuser
ENV RUST_LOG="${EXAMPLE_BIN}=debug,info"

# Default workdir and config directory
WORKDIR /usr/local/bin/
RUN mkdir -p /usr/local/bin/config

# Allow passing CONFIG_PATH env to override default
# Example: docker run -e CONFIG_PATH=/usr/local/bin/config/input_output_int_saver/config.json IMAGE
ENTRYPOINT ["/bin/sh","-lc","exec /usr/local/bin/$EXAMPLE_BIN ${CONFIG_PATH:-}"]
