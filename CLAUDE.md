# greenhouse_backend — CLAUDE.md

AI assistant guide for the OpenGreenhouseManager backend.

## Overview

Rust workspace implementing a microservices backend for a greenhouse management system. Services expose HTTP APIs, communicate through a shared core library, and are deployed as individual Docker containers.

## Technology Stack

- **Language**: Rust (nightly toolchain — see `rust-toolchain.toml`)
- **Web framework**: Axum 0.8
- **Database ORM**: Diesel 2 + diesel-async with PostgreSQL via bb8 connection pool
- **Auth**: JWT (`jsonwebtoken`) + bcrypt passwords + HTTP-only cookies (`tower-cookies`)
- **Serialization**: serde / serde_json / serde_yaml
- **Logging/Tracing**: `tracing` + `tracing-subscriber` + Sentry error tracking
- **Task runner**: `just` (see `justfile`)
- **Container registry**: `ghcr.io/opengreenhousemanager/`

## Repository Layout

```
greenhouse_backend/
├── Cargo.toml              # Workspace manifest
├── rust-toolchain.toml     # Pins nightly channel
├── justfile                # Developer task runner
├── compose.yaml            # Local dev Docker Compose (PostgreSQL + example devices)
├── Dockerfile              # Base builder image for all services
├── greenhouse_core/        # Shared library: DTOs + smart-device interface
│   └── src/
│       ├── auth_service_dto/
│       ├── data_storage_service_dto/
│       ├── device_service_dto/
│       ├── scripting_service_dto/
│       ├── smart_device_dto/
│       ├── smart_device_interface/
│       └── http_error.rs
├── greenhouse_macro/       # Procedural macro crate
├── services/
│   ├── auth_service/       # User auth, JWT issuance, bcrypt passwords
│   ├── data_storage_service/ # Time-series / sensor data persistence
│   ├── device_service/     # Smart-device registration and management
│   └── scripting_service/  # Automation scripting engine
├── api/
│   ├── web/                # web_api — REST gateway used by the Angular frontend
│   └── script/             # scripting_api — API for the scripting subsystem
├── examples/               # Example smart-device binaries (used for local testing)
├── integration-tests/      # Workspace-level integration test crate
├── docker/                 # Supporting Docker files (e.g. postgres/init.sql)
└── scripts/                # Build helper scripts (build-image-layer.sh)
```

### Service structure (each service follows this pattern)

```
services/<name>/
├── Cargo.toml
├── Dockerfile          # Inherits from base builder image
├── config/             # YAML/JSON runtime config files
├── diesel.toml         # Diesel migration config
├── migrations/         # SQL migration files
└── src/
    ├── main.rs
    ├── lib.rs
    ├── router/         # Axum route handlers
    ├── database/       # Diesel models and queries
    └── ...             # Service-specific modules (e.g. token/)
```

## Common Development Commands

All primary tasks are in `justfile`. Run `just --list` to see all recipes.

```bash
# Start all services (logs → logs/services.log)
just run-services

# Start all API gateways (logs → logs/apis.log)
just run-apis

# Start everything
just start-all

# Start everything except specified services
just start-all-except auth_service web_api

# Stop all running services/APIs
just stop-all

# Run the full CI suite locally (lint + test + fmt)
just ci

# Individual checks
just lint    # cargo clippy --all-targets --all-features --workspace -- -D warnings
just test    # cargo test --release --workspace --all-features -- --test-threads=1
just fmt     # cargo fmt --all -- --color always

# Run example devices
just device
just device-stop
```

### Local database

```bash
# Start PostgreSQL (and example devices)
docker compose up db -d

# Run Diesel migrations for a specific service
cd services/auth_service && diesel migration run
```

## CI/CD

### Continuous Integration (`.github/workflows/rust_ci.yml`)

Triggers on every push and pull request. Runs in a matrix:

| Step | Command |
|------|---------|
| Typo check | `typos` |
| Dependency sort | `cargo sort --workspace --grouped` |
| Build | `cargo build --release --workspace` |
| Format | `cargo fmt --all -- --check --color always` |
| Lint | `cargo clippy --all-targets --all-features --workspace -- -D warnings` |
| Tests | `cargo test --release --workspace --all-features -- --test-threads=1` |

**All CI checks must pass before merging.**

### Docker publish (`.github/workflows/docker.yml`)

- **On PR to `main` that touches `Dockerfile`**: builds base image for `linux/amd64` + `linux/arm64` (no push).
- **On `v*` tag push**: builds and publishes all service images to `ghcr.io/opengreenhousemanager/`:
  - `auth_service`
  - `data_storage`
  - `device_service`
  - `scripting_service`
  - `web_api`
  - `script_api`

### Releasing

Tag the commit with `v*` (e.g. `v1.2.3`) to trigger Docker image publication. See `release.toml` for `cargo-release` configuration.

## Code Conventions

### Rust style

- **Nightly Rust** is required — `rust-toolchain.toml` pins `channel = "nightly"`.
- Clippy is run with `-D warnings` — treat every warning as an error.
- `cargo fmt` uses nightly `rustfmt`.
- Keep `Cargo.toml` dependency list sorted (`cargo sort`).
- Spell-check all source files with `typos` before committing.

### Shared DTOs (`greenhouse_core`)

- All inter-service data transfer types live in `greenhouse_core/src/<service>_dto/`.
- Add new DTOs here — never define them only inside a single service crate.
- The `smart_device_interface` module defines the trait all example devices implement.

### Database migrations

- Each service that owns a database has its own `migrations/` folder.
- Generate new migrations with `diesel migration generate <name>` from inside the service directory.
- Migrations run automatically at service startup via `diesel_migrations`.

### Error handling

- HTTP errors are standardised via `greenhouse_core/src/http_error.rs`.
- Use `tracing` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) for structured logging.
- Set `RUST_LOG=<crate>=debug,info` in environment for runtime log filtering.

### Configuration

- Services read their config from a path provided by the `CONFIG_PATH` environment variable.
- Config files are YAML/JSON under each service's `config/` directory.

## Environment Variables

| Variable | Used by | Purpose |
|----------|---------|---------|
| `RUST_LOG` | All services | Log filter (e.g. `auth_service=debug,info`) |
| `CONFIG_PATH` | All services | Path to the service YAML/JSON config file |
| `DATABASE_URL` | Services with Diesel | PostgreSQL connection string (also used by diesel CLI) |

## Docker Compose (Local Dev)

`compose.yaml` provides:
- **`db`** — PostgreSQL on port `5432` with `admin`/`password` credentials, initialised by `docker/postgres/init.sql`.
- Three **example device** containers (ports `6001`–`6003`) with health checks.

Default credentials for local dev:
- PostgreSQL user: `admin`, password: `password`

## Adding a New Service

1. Create `services/<new_service>/` following the existing service structure.
2. Add it to the `members` array in the root `Cargo.toml`.
3. Add a `Dockerfile` that uses the base builder image.
4. Add a DTO module in `greenhouse_core/src/<new_service>_dto/` and export it from `lib.rs`.
5. Add `cargo run --package <new_service>` to the relevant `just` recipes.
6. Add a publish step in `.github/workflows/docker.yml`.
