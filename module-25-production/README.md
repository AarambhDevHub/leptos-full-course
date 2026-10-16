# Module 25 - Production

## What You'll Learn

- Auth
- CRUD
- Uploads
- SSE
- Deployment
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## How to Run

```bash
cd module-25-production
cargo leptos watch
```

## Routes and Endpoints

| Path | Purpose |
|------|---------|
| `/` | Main lesson dashboard |
| `/api/*` | Leptos server functions |

## Key Concepts

```rust
use leptos::prelude::*;

let (value, set_value) = signal(0);
let doubled = Memo::new(move |_| value.get() * 2);
set_value.update(|n| *n += 1);
```

## Working Examples

- Home
- Dashboard
- Notes
- Profile
- Settings

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 24](../module-24-islands)

Next: Finish


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

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Signing key for auth demos |
| `LEPTOS_SITE_ADDR` | Server bind address |
| `RUST_LOG` | Tracing filter |

### Production Checklist

- Run migrations before accepting traffic.
- Serve behind TLS with Caddy, nginx, Fly.io, or Railway.
- Enable structured tracing and collect logs centrally.
- Monitor latency, error rate, database pool saturation, and memory.
- Build with the `wasm-release` profile for smaller client bundles.
