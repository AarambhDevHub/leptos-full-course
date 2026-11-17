# Module 21 - i18n and Accessibility

## What You'll Learn

- Translations
- RTL
- ARIA
- Focus
- Keyboard navigation
- How the concepts fit into a production Leptos 0.8 app

## Prerequisites

- Rust 1.89+
- `wasm32-unknown-unknown` target
- `trunk` for CSR modules or `cargo-leptos` for SSR modules

## How to Run

```bash
cd module-21-i18n
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

- Language switcher (English, Spanish, Hindi, Arabic) with saved choice and browser detection
- Right-to-left layout (`lang` and `dir` on the page)
- Accessible form
- Live region that announces only real saves
- Keyboard menu with arrow keys

## Try It

1. Run the module.
2. Open the browser at the printed local URL.
3. Change the interactive examples and inspect the Rust code in `src/`.
4. Modify one example, rebuild, and compare the behavior.

Previous: [Module 20](../module-20-caching)

Next: [Module 22](../module-22-js-interop)
