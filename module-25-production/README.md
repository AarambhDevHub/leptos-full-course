# Module 25 - Production

The capstone: one application that has accounts, private data in a database,
an honest activity log, health checks, hardening, and a deployment recipe.

## What You'll Learn

- Authentication: Argon2 password hashes, signed session tokens, an HttpOnly and Secure cookie
- Protected pages that are correct in the server-rendered HTML, with no "login required" flash
- Notes CRUD where every query is limited to the signed-in user
- An activity feed and dashboard numbers that come from the database
- Input rules as small pure functions with unit tests (`src/guard.rs`)
- Operational endpoints (`/health`, `/ready`), structured logs, graceful shutdown
- Docker and Docker Compose deployment

## Prerequisites

- Rust 1.89+, the `wasm32-unknown-unknown` target and `cargo-leptos`
- PostgreSQL 16 (or Docker)

## How to Run

```bash
# Postgres, for example:
docker run -d --name leptos-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=leptos_course -p 5432:5432 postgres:16

export JWT_SECRET=$(openssl rand -hex 32)   # required: the server refuses to start without it
export COOKIE_SECURE=false                  # local http only; keep the default (true) behind https
cd module-25-production
cargo leptos watch
```

Run the unit tests with `cargo test -p module-25-production`.

## Routes and Endpoints

| Path | Purpose |
|------|---------|
| `/`, `/login`, `/register` | Public pages |
| `/dashboard`, `/notes`, `/profile`, `/settings`, `/activity` | Pages for signed-in users |
| `/health` | The process is alive (always `200 ok`) |
| `/ready` | The database answers (`200`, or `503`) |
| `/api/*` | Leptos server functions |

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Required. At least 32 characters and not an example value |
| `COOKIE_SECURE` | `false` removes the Secure cookie flag, for local http only |
| `LEPTOS_SITE_ADDR` | Server bind address |
| `LEPTOS_OUTPUT_NAME`, `LEPTOS_SITE_ROOT`, `LEPTOS_SITE_PKG_DIR` | Needed when you run the bare binary (the Dockerfile sets them) |
| `RUST_LOG` | Tracing filter (defaults to `info`) |

## Deploy with Docker Compose

```bash
cp .env.example .env     # in the repository root
# edit .env: set JWT_SECRET (openssl rand -hex 32) and POSTGRES_PASSWORD,
# and COOKIE_SECURE=false if you test over plain http
docker compose up --build
```

Note: the Docker files in this repository were reviewed and the server binary
was run the way the container runs it, but the image itself has not been built
by the course author. Build it once and watch the first start.

## What this module does not do

These are real production needs that are left out so the lesson stays focused:

- Email confirmation and password reset
- A CSRF token (the cookie is `SameSite=Lax`, which blocks most cross-site posts)
- Revoking a token before it expires (a session lasts twelve hours)
- A Content-Security-Policy (Leptos injects an inline script, so a strict policy needs a nonce)
- A login limiter shared between several servers (this one is in memory, per process)
- File uploads and live events: those are modules 17 to 19

## Production Checklist

- Serve behind TLS (Caddy, nginx, Fly.io or Railway) and keep `COOKIE_SECURE` on.
- Keep PostgreSQL private; do not publish its port.
- Store `JWT_SECRET` in your platform's secret store, never in the image.
- Point the platform's health check at `/health` and the readiness check at `/ready`.
- Collect the JSON logs and watch latency, error rate and database pool use.
- Back up the database and test a restore.
- Build with the `wasm-release` profile for smaller client bundles.

## Deployment Notes

### Fly.io

```toml
app = "leptos-course-production"
primary_region = "sin"

[http_service]
internal_port = 3000
force_https = true
auto_stop_machines = true
auto_start_machines = true
```

### Railway

```json
{
  "build": { "builder": "DOCKERFILE" },
  "deploy": { "restartPolicyType": "ON_FAILURE" }
}
```

Previous: [Module 24](../module-24-islands)

Next: Finish
