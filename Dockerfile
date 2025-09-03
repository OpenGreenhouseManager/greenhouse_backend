ARG RUST_VERSION=1.89.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .

ARG TARGETARCH

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    pkg-config \
    libssl-dev \
    libpq-dev  && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    rustup toolchain install nightly && \
    rustup update nightly && \
    rustup default nightly


COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

# Build the application
RUN sh /tmp/build-image-layer.sh apps && \
  find "./target/release/" -maxdepth 1 -type f -exec test -x {} \; -exec cp {} / \;
