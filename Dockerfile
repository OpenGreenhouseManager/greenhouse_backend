ARG RUST_VERSION=1.78.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install -y \
    pkg-config \
    libssl-dev

RUN \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --release --package example-hybrid-device --bin example-hybrid-device && \
  cp ./target/x86_64-unknown-linux-gnu/release/example-hybrid-device /
RUN cp ./docker/hybrid_device/config.json /

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
COPY --from=builder /config.json /usr/local/bin
RUN chown appuser /usr/local/bin/example-hybrid-device
RUN chown -R appuser /usr/local/bin/
USER appuser
ENV RUST_LOG="hello_rs=debug,info"
WORKDIR /usr/local/bin/
ENTRYPOINT ["example-hybrid-device"]
EXPOSE 9092/tcp