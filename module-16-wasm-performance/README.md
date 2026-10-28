# Module 16 - WASM Performance

## What You'll Learn

- Binary size
- Memos
- Batching
- Untracked reads
- Lazy rendering
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## How to Run

```bash
cd module-16-wasm-performance
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

- Batching
- Memo vs derived
- Untrack
- Lazy list
- Worker placeholder

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 15](../module-15-testing)

Next: [Module 17](../module-17-file-uploads)

## Measuring the binary size

Always measure a release build, and measure the compressed size, because that is what users download.

```bash
cd module-16-wasm-performance
trunk build --release
ls -l dist/*.wasm
gzip -9 -c dist/*.wasm | wc -c
brotli -q 11 -c dist/*.wasm | wc -c
```

One measured example (release profile with `opt-level = "z"`, LTO, one codegen unit and `panic = "abort"`, Leptos 0.8.22): about 2 MB in a debug build, about 240 KB after `wasm-bindgen`, about 140 KB after `wasm-opt -Oz`, and about 50 KB over the wire with Brotli. Your numbers will differ with your toolchain.

Run the unit tests with `cargo test -p module-16-wasm-performance`.
