ARG RUST_VERSION=1.86.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    pkg-config=1.8.1-1 \
    libssl-dev=3.0.16-1~deb12u1

RUN \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --release --package example-hybrid-device --bin example-hybrid-device && \
  cp "./target/${TARGETPLATFORM}/release/example-hybrid-device" /

FROM debian:bookworm-slim AS final
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "10001" \
  appuser
  
COPY --from=builder /example-hybrid-device /usr/local/bin
RUN chown appuser /usr/local/bin/example-hybrid-device && \
  chown -R appuser /usr/local/bin/

USER appuser
ENV RUST_LOG="example-hybrid-device=debug,info"
RUN mkdir /usr/local/bin/config
WORKDIR /usr/local/bin/
ENTRYPOINT ["example-hybrid-device"]
