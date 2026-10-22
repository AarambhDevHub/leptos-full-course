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
- **Module 04:** todo ids were made with `items.len() + 1`, so deleting a todo and then adding one reused an id and the new row silently never appeared in the keyed `<For/>` list. Ids now come from a counter that only goes up.
- **Module 08:** the sample todos were re-added whenever the list was empty, so deleting every todo brought them back. They are now seeded only once.
- **Module 09:** with default streaming, the per-article `Meta` description inside `Suspense` never reached the `<head>`, and the home page had two `description` tags. The article route now uses `SsrMode::Async` (title and description are in the first response) and the duplicate app-level description was removed.
- **Module 10:** the note preview sliced the body by bytes (`&body[..50]`), so a long Hindi (or emoji) note panicked and killed the whole WebAssembly app. It now counts characters, with three unit tests. Also: after saving an edit the card kept showing the old title (the keyed row displayed a captured copy), `update_note` accepted a blank title, and the tables are created at startup (see above).
- **Module 11 and 25:** every password was hashed with the same fixed salt, so two users with the same password had identical hashes. Each password now gets a fresh random salt (existing users can still log in, because the salt is stored inside the hash). A duplicate email no longer returns the raw database error (it leaked the constraint name); it returns "that email is already registered" and logs the real error on the server.
- **Module 12:** the cart list showed a stale quantity (a keyed row displayed a captured copy, so "x1" stayed while the summary said 3 items), and notification ids used `len() + 1`, so a new toast could get the id of a live one and never appear. The row now reads its quantity from the cart, and ids come from a counter (`NextNotificationId` newtype in context).
- **Module 13:** the module was titled "Styling and Tailwind" but contains no Tailwind. It is now simply "Styling".
- **Module 14:** a quick second click on the fade toggle let the first timer unmount the box (fixed with a generation number, and the fade now also fades in); two animated-counter loops fought over one signal and the number zigzagged (the newest loop now cancels older ones); the Replay button changed an attribute and restarted nothing (the list is now recreated with a keyed `For`); the bouncing ball's interval used `.forget()` and could never stop (now cleaned up with `on_cleanup`).
- **Module 17:** uploads over 2 MB (Axum's default limit) were silently dropped while the server answered `ok`. The route now allows 10 MB (matching `.env.example`) and returns real errors (`413` for oversized files).

## How this was verified

- Type-checked all 25 modules with `cargo check` (CSR, `ssr` and `hydrate`): no errors, no warnings.
- Built all 25 modules to `wasm32-unknown-unknown`.
- Ran the 14 CSR modules in jsdom: all mount and survive clicking every button; modules 01, 05, 06 and 07 have behavior tests (counter, form validation, routing, async resources).
- Ran modules 12, 13 and 14 in a simulated browser: cart, toasts, localStorage, wizard and theme; class and style directives, CSS variables and dark mode; fade, counter (every DOM update recorded), replay and ball.
- Hydrated modules 10 and 11 against PostgreSQL 16 and exercised the notes page (edit, optimistic delete, non-English text) and the register, login, logout and admin flows.
- Hydrated the server-rendered modules 08 and 09 in a simulated browser and exercised real server functions, client-side navigation and head tags.
- Ran the 11 server modules and exercised their pages and endpoints, against PostgreSQL 16 for modules 10, 11 and 25.
- `cargo test -p module-15-testing` (3 tests) passes.

Not covered: real-browser rendering and CSS, the `wasm_bindgen_test` tests in module 15 (they need a browser), and `cargo leptos` / `trunk` builds (the checks used `cargo` and `wasm-bindgen` directly).
