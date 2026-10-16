# Update to Leptos 0.8.22

## Versions

| Crate | Before | Now |
|-------|--------|-----|
| `leptos` | 0.8.19 | 0.8.22 |
| `leptos_meta` | 0.8.6 | 0.8.7 |
| `leptos_router` | 0.8.13 | 0.8.17 |
| `leptos_axum` | 0.8.9 | 0.8.10 |

Minimum Rust is now documented as 1.89 (Leptos 0.8.22 needs 1.88, and the latest `uuid` needs 1.89). The READMEs previously said 1.78.

No course code needed API changes for 0.8.22.

## Fixes made while verifying

- **Module 10 and 11:** the tables were never created, so a fresh database returned `relation "notes" does not exist` (module 10) or failed on register (module 11). Both now run their idempotent `migrations/0001_init.sql` at startup. They intentionally do not use SQLx's migration tracker, because modules 10, 11 and 25 share one database and all have a migration numbered `0001`.
- **Module 17:** uploads over 2 MB (Axum's default limit) were silently dropped while the server answered `ok`. The route now allows 10 MB (matching `.env.example`) and returns real errors (`413` for oversized files).

## How this was verified

- Type-checked all 25 modules with `cargo check` (CSR, `ssr` and `hydrate`): no errors, no warnings.
- Built all 25 modules to `wasm32-unknown-unknown`.
- Ran the 14 CSR modules in jsdom: all mount and survive clicking every button; modules 01, 05, 06 and 07 have behavior tests (counter, form validation, routing, async resources).
- Ran the 11 server modules and exercised their pages and endpoints, against PostgreSQL 16 for modules 10, 11 and 25.
- `cargo test -p module-15-testing` (3 tests) passes.

Not covered: real-browser rendering and CSS, the `wasm_bindgen_test` tests in module 15 (they need a browser), and `cargo leptos` / `trunk` builds (the checks used `cargo` and `wasm-bindgen` directly).
