ARG RUST_VERSION=1.87.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .

ARG TARGETARCH

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    pkg-config=1.8.1-1 \
    libssl-dev=3.0.16-1~deb12u1 \
    libpq-dev=15.13-0+deb12u1  && \
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
