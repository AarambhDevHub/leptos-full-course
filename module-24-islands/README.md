# Module 24 - Islands Architecture

## What You'll Learn

- Islands
- Static HTML
- Hydration scope
- Search island
- Theme island
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## How to Run

```bash
cd module-24-islands
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

- Blog layout (server-only component, no WebAssembly)
- Search island (a real `#[island]`, the full list is in the server HTML)
- Newsletter island (server-side address validation, status region)
- Theme island (head script applies the saved theme before paint, CSS variables)

This module uses the Leptos `islands` feature: `#[island]` components, `HydrationScripts ... islands=true` and `hydrate_islands()`. The feature is marked experimental. To see the difference, build with `cargo leptos build --release` and compare the size of `target/site/pkg/*.wasm` with the same page hydrated as a whole app. The old table of sizes in the page was never measured, so it was removed.

Run the unit tests with `cargo test -p module-24-islands`.

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 23](../module-23-error-handling)

Next: [Module 25](../module-25-production)
