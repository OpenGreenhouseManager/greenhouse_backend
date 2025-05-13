ARG RUST_VERSION=1.86.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .
ARG TARGETARCH

# Install build requirements
RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    pkg-config=1.8.1-1 \
    libssl-dev=3.0.15-1~deb12u1 \
    libpq-dev=15.12-0+deb12u2  && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*


COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

RUN \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  sh /tmp/build-image-layer.sh apps && \
  cp "./target/${TARGETPLATFORM}/release/" /
