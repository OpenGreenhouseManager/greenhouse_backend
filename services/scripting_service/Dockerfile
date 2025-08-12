
# hadolint ignore=DL3007
FROM ghcr.io/opengreenhousemanager/greenhouse_backend/base:latest AS builder
#
FROM debian:bookworm-slim AS final
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "10001" \
  appuser
  
COPY --from=builder /scripting_service /usr/local/bin
RUN chown appuser /usr/local/bin/scripting_service && \
  chown -R appuser /usr/local/bin/

RUN apt-get update -y && apt-get upgrade -y && \ 
  apt-get install --no-install-recommends -y \
    libpq-dev=15.13-0+deb12u1\
    ca-certificates=20230311\
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER appuser
ENV RUST_LOG="scripting_service=debug,info"
RUN mkdir /usr/local/bin/config
WORKDIR /usr/local/bin/
ENTRYPOINT ["scripting_service"]
EXPOSE 9091
#keep this container alive
#ENTRYPOINT [ "tail", "-f", "/dev/null" ]
