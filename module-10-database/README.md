# Module 10 - Database

## What You'll Learn

- SQLx
- PgPool
- Migrations
- CRUD
- Optimistic UI
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## Database Setup

This module needs PostgreSQL 16 (for example `docker compose up postgres` from the repository root). The default connection string is `postgres://postgres:postgres@localhost:5432/leptos_course`; override it with `DATABASE_URL`. The server creates the tables it needs from `migrations/0001_init.sql` when it starts, so no manual setup is required.

## How to Run

```bash
cd module-10-database
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

- List notes
- Create note
- Edit note
- Delete note
- Rollback

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 09](../module-09-ssr)

Next: [Module 11](../module-11-auth)
