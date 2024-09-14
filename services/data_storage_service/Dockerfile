ARG RUST_VERSION=1.78.0

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    pkg-config=1.8.1-1 \
    libssl-dev=3.0.13-1~deb12u1 \
    libpq-dev=15.7-0+deb12u1

RUN \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --release --package data_storage_service --bin data_storage_service && \
  cp ./target/x86_64-unknown-linux-gnu/release/data_storage_service /

FROM debian:bookworm-slim AS final
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "10001" \
  appuser
  
COPY --from=builder /data_storage_service /usr/local/bin
RUN chown appuser /usr/local/bin/data_storage_service && \
  chown -R appuser /usr/local/bin/

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    libpq-dev=15.7-0+deb12u1\
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER appuser
ENV RUST_LOG="data_storage_service=debug,info"
RUN mkdir /usr/local/bin/config
WORKDIR /usr/local/bin/
ENTRYPOINT ["data_storage_service"]
EXPOSE 9091
#keep this container alive
#ENTRYPOINT [ "tail", "-f", "/dev/null" ]
