# Module 15 - Testing

## What You'll Learn

- Native tests
- WASM tests
- Validation tests
- State tests
- Component tests
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## How to Run

```bash
cd module-15-testing
trunk serve
```

## Routes and Endpoints

| Path | Purpose |
|------|---------|
| `/` | Main lesson dashboard |

## Key Concepts

```rust
use leptos::prelude::*;

let (value, set_value) = signal(0);
let doubled = Memo::new(move |_| value.get() * 2);
set_value.update(|n| *n += 1);
```

## Working Examples

- Counter tests
- Form validation
- Todo state
- Server fn integration
- Render checks

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 14](../module-14-animations)

Next: [Module 16](../module-16-wasm-performance)

## Running the tests

```bash
# Native tests: pure logic, run anywhere, finish in a fraction of a second
cargo test -p module-15-testing

# Browser tests (wasm-bindgen-test): need Chrome; wasm-pack downloads the driver
cd module-15-testing && wasm-pack test --headless --chrome
```

The nine native tests include six edge-case tests (reused todo ids, email edge cases, and file-size rounding at unit boundaries). They were written first and failed against the original helpers, so they are a good example of the red, green, refactor workflow.
