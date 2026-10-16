# Module 23 - Error Handling

## What You'll Learn

- ErrorBoundary
- Server errors
- Status codes
- Retry
- Rollback
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## How to Run

```bash
cd module-23-error-handling
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

- Retry resource
- Partial failure
- Rollback
- Global boundary
- Error log

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 22](../module-22-js-interop)

Next: [Module 24](../module-24-islands)
