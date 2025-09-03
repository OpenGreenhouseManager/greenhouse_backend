ARG RUST_VERSION=1.89.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    pkg-config \
    libssl-dev

COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

# Build the application
RUN sh /tmp/build-image-layer.sh apps && \
  find "./target/release/" -maxdepth 1 -type f -exec test -x {} \; -exec cp {} / \;

FROM debian:bookworm-slim AS final
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "10001" \
  appuser
  
COPY --from=builder /periodic_alert /usr/local/bin
RUN chown appuser /usr/local/bin/periodic_alert && \
  chown -R appuser /usr/local/bin/

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
        libpq-dev=15.13-0+deb12u1\
          ca-certificates=20230311\
  && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER appuser
ENV RUST_LOG="periodic_alert=debug,info"
RUN mkdir /usr/local/bin/config
WORKDIR /usr/local/bin/
ENTRYPOINT ["periodic_alert"]
