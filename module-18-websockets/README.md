# Module 18 - WebSockets

## What You'll Learn

- WebSocket API
- Axum upgrade
- Broadcast
- Reconnects
- JSON protocol
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## How to Run

```bash
cd module-18-websockets
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

- Connection status
- Message list
- Input
- Users
- Server room

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 17](../module-17-file-uploads)

Next: [Module 19](../module-19-sse)
