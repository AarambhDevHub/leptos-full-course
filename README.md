# Leptos 0.8 Full Course - Build Production Web Apps in Rust

[![Rust](https://img.shields.io/badge/Rust-1.89+-orange.svg)](https://www.rust-lang.org)
[![Leptos](https://img.shields.io/badge/Leptos-0.8.22-blue.svg)](https://leptos.dev)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![YouTube](https://img.shields.io/badge/YouTube-Aarambh_Dev_Hub-red.svg)](https://youtube.com/@AarambhDevHub)

Complete 25-module course - Leptos 0.8 from zero to production full-stack Rust web apps.

## Requirements

- Rust 1.89 or newer. Leptos 0.8.22 itself needs Rust 1.88+, and the latest `uuid` release used by the course needs 1.89.
- The `wasm32-unknown-unknown` target, `trunk` (CSR modules), and `cargo-leptos` (SSR modules).
- Docker (optional) or a local PostgreSQL 16 for modules 10, 11, and 25.

## Quick Start

```bash
rustup update stable
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install cargo-leptos --locked

# Run the first CSR module.
cd module-01-intro
trunk serve

# Run the first SSR module.
cd ../module-08-server-functions
cargo leptos watch
```

## Course Modules

| Module | Name | Key topics | Track | How to run |
|--------|------|------------|-------|------------|
| 01 | [Introduction](./module-01-intro) | What Leptos is, The view macro, Components | CSR | `cd module-01-intro && trunk serve` |
| 02 | [Components](./module-02-components) | Required props, Optional props, Children | CSR | `cd module-02-components && trunk serve` |
| 03 | [Reactivity](./module-03-reactivity) | Signals, Effects, Memos | CSR | `cd module-03-reactivity && trunk serve` |
| 04 | [Control Flow](./module-04-control-flow) | Show, For, Rust conditionals | CSR | `cd module-04-control-flow && trunk serve` |
| 05 | [Forms and Events](./module-05-forms) | Events, Controlled inputs, Validation | CSR | `cd module-05-forms && trunk serve` |
| 06 | [Routing](./module-06-routing) | Router, Routes, Params | CSR | `cd module-06-routing && trunk serve` |
| 07 | [Async and Resources](./module-07-async-resources) | Resources, Suspense, Transition | CSR | `cd module-07-async-resources && trunk serve` |
| 08 | [Server Functions](./module-08-server-functions) | Server functions, ServerAction, ActionForm | SSR | `cargo leptos watch -p module-08-server-functions` |
| 09 | [SSR and Hydration](./module-09-ssr) | Server rendering, Hydration, Streaming | SSR | `cargo leptos watch -p module-09-ssr` |
| 10 | [Database](./module-10-database) | SQLx, PgPool, Migrations | SSR | `cargo leptos watch -p module-10-database` |
| 11 | [Authentication](./module-11-auth) | Cookies, JWT, Password hashing | SSR | `cargo leptos watch -p module-11-auth` |
| 12 | [State Management](./module-12-state-management) | Local state, Context, RwSignal | CSR | `cd module-12-state-management && trunk serve` |
| 13 | [Styling and Tailwind](./module-13-styling) | Static classes, Conditional classes, Dynamic styles | CSR | `cd module-13-styling && trunk serve` |
| 14 | [Animations](./module-14-animations) | Transitions, Keyframes, Deferred removal | CSR | `cd module-14-animations && trunk serve` |
| 15 | [Testing](./module-15-testing) | Native tests, WASM tests, Validation tests | CSR | `cd module-15-testing && trunk serve` |
| 16 | [WASM Performance](./module-16-wasm-performance) | Binary size, Memos, Batching | CSR | `cd module-16-wasm-performance && trunk serve` |
| 17 | [File Uploads](./module-17-file-uploads) | File inputs, Multipart, Validation | SSR | `cargo leptos watch -p module-17-file-uploads` |
| 18 | [WebSockets](./module-18-websockets) | WebSocket API, Axum upgrade, Broadcast | SSR | `cargo leptos watch -p module-18-websockets` |
| 19 | [Server-Sent Events](./module-19-sse) | EventSource, SSE responses, Keepalive | SSR | `cargo leptos watch -p module-19-sse` |
| 20 | [Caching and Optimization](./module-20-caching) | Resource cache, SWR, HTTP caching | SSR | `cargo leptos watch -p module-20-caching` |
| 21 | [i18n and Accessibility](./module-21-i18n) | Translations, RTL, ARIA | CSR | `cd module-21-i18n && trunk serve` |
| 22 | [JS Interop](./module-22-js-interop) | wasm-bindgen, web-sys, js-sys | CSR | `cd module-22-js-interop && trunk serve` |
| 23 | [Error Handling](./module-23-error-handling) | ErrorBoundary, Server errors, Status codes | SSR | `cargo leptos watch -p module-23-error-handling` |
| 24 | [Islands Architecture](./module-24-islands) | Islands, Static HTML, Hydration scope | SSR | `cargo leptos watch -p module-24-islands` |
| 25 | [Production](./module-25-production) | Auth, CRUD, Uploads | SSR | `cargo leptos watch -p module-25-production` |

## Learning Tracks

- Foundations CSR: modules 01-07 teach browser-rendered Leptos with Trunk.
- Applied CSR: modules 12-16 and 21-22 cover state, styling, testing, performance, i18n, and JS interop.
- Full-stack SSR: modules 08-11, 17-20, and 23-25 use Axum, cargo-leptos, server functions, realtime features, and deployment patterns.

## What's New in Leptos 0.8

This course is verified against Leptos 0.8.22 (`leptos_meta` 0.8.7, `leptos_router` 0.8.17, `leptos_axum` 0.8.10). 0.8.22 is a patch release: it fixes hydration picking up the wrong data after hydration has finished, and updates `rstml` to 0.13.1 for a security advisory. No course code needed API changes.

- The stable API centers on `use leptos::prelude::*`, `signal(...)`, `RwSignal::new(...)`, `Memo::new(...)`, and `Effect::new(...)`.
- Signal reads and writes use explicit `.get()`, `.set()`, `.update()`, `.read()`, and `.write()` methods in this course.
- The router uses the 0.8 route components and path APIs.
- Full-stack modules follow the 0.8 Axum starter layout with `src/app.rs`, `src/main.rs`, and `src/lib.rs` hydration entry points.

## Docker and PostgreSQL

```bash
cp .env.example .env
docker compose up postgres
```

Database-focused modules include local migrations and expect `DATABASE_URL` from `.env.example`.

## Community

- YouTube: https://youtube.com/@AarambhDevHub
- GitHub: https://github.com/aarambh-darshan
- Discord: add your community invite here
- Buy Me a Coffee: https://buymeacoffee.com/aarambhdevhub

Made with care by Aarambh Dev Hub for the Rust community.
