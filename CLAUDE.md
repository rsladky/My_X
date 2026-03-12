# My_X — Claude Context

## What This Is

A full-stack Twitter/X clone built entirely in Rust as a learning project. Backend: Axum. Frontend: Leptos (SSR + WASM hydration). Database: PostgreSQL via SQLx. The goal is understanding Rust ownership, async, and real-world patterns — not production deployment.

## Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| HTTP server | axum | 0.8.8 |
| Frontend | leptos + leptos_axum | 0.8.17 |
| Build tool | cargo-leptos | 0.2.39 |
| Database | PostgreSQL | 16+ |
| DB access | sqlx | 0.8.6 |
| Auth | jsonwebtoken (rust_crypto feature) + argon2 | 10.3.0 / 0.5.3 |
| Async runtime | tokio | 1.x |

## Dev Commands

```bash
cargo leptos watch          # dev server with hot reload
cargo test --features ssr   # run tests
cargo build --features ssr  # server build
cargo build                 # WASM/hydrate build (must also pass)
sqlx migrate run            # apply DB migrations
```

## Established Patterns

**Server functions only** — all frontend↔backend data flow uses `#[leptos::server]` functions. No manual fetch, no REST client.

**Feature-gate server-only crates** — sqlx, jsonwebtoken, argon2, uuid are all under `[ssr]` in Cargo.toml. WASM compilation will fail otherwise.

**Shared types must compile in WASM** — structs shared between server and client (e.g. `PostWithAuthor`, `AuthUser`) must be defined OUTSIDE `#[cfg(feature = "ssr")]`.

**PgPool via context** — pool is provided via `leptos_routes_with_context`, not Axum State. Access with `use_context::<PgPool>()` inside server functions.

**Auth state** — `RwSignal<Option<AuthUser>>` provided via context. Access with `use_context::<RwSignal<Option<AuthUser>>>()` in components.

**Inline styles** — no CSS framework, no Tailwind. All styling is inline.

**`navigate()` after `spawn_local`** — call `navigate()` inside `spawn_local` after `validate_token` resolves to avoid flash of unauthenticated state.

**`AuthUser`** carries `user_id: i32` + `username: String`.

## Authorization Pattern

`delete_post`: use `WHERE id = $1 AND author_id = $2` + check `rows_affected() == 0` → error. The DB is authoritative; user_id comes from auth context signal.

## Current State

- Phase 1 (Foundation): complete
- Phase 2 (Auth): complete — register, login, JWT, persist, logout all working
- **Phase 3 (Posts + Profiles): next up** — see TODO.md

## Out of Scope (v1)

Likes, replies, retweets, media uploads, real-time updates, search, notifications, DMs, deployment.
