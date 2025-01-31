ARG RUST_VERSION=1.82.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    pkg-config=1.8.1-1 \
    libssl-dev=3.0.15-1~deb12u1

RUN \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --release --package web_api --bin web_api && \
  cp ./target/x86_64-unknown-linux-gnu/release/web_api /

FROM debian:bookworm-slim AS final
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "10001" \
  appuser
  
COPY --from=builder /web_api /usr/local/bin
RUN chown appuser /usr/local/bin/web_api && \
  chown -R appuser /usr/local/bin/

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
        libpq-dev=15.10-0+deb12u1\
         ca-certificates=20230311\
  && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER appuser
ENV RUST_LOG="web_api=debug,info"
RUN mkdir /usr/local/bin/config
WORKDIR /usr/local/bin/
ENTRYPOINT ["web_api"]
#keep this container alive
#ENTRYPOINT [ "tail", "-f", "/dev/null" ]
