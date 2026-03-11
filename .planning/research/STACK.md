# Stack Research

**Domain:** Full-stack Rust web application (Twitter/X clone)
**Researched:** 2026-03-11
**Confidence:** MEDIUM-HIGH (core framework choices are well-settled; some auth crate landscape is immature)

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| axum | 0.8.8 | HTTP server, routing, request handling | Built by the Tokio team; ergonomic extractors, compile-time type safety, native Tower middleware integration. Cleaner DX than Actix-web for learners, no actor-model baggage. The de-facto new-project choice in 2025. |
| leptos | 0.8.17 | Full-stack frontend UI (SSR + hydration) | Fine-grained reactivity (no virtual DOM), first-class SSR + WASM hydration, tight Axum integration via `leptos_axum`. Server functions let you call backend logic from UI code in pure Rust. The most ambitious and active Rust web UI framework. |
| cargo-leptos | 0.2.39 | Build coordinator for Leptos | Handles dual compilation (server binary + WASM client), hot reloading, asset pipeline. Required for Leptos SSR — not optional. |
| PostgreSQL | 16+ | Primary database | Specified by project requirements. Better ecosystem crate support in Rust than MySQL; more realistic social-graph queries than SQLite. |
| sqlx | 0.8.6 | Async database access with compile-time checked queries | Async-first (Tokio-native), compile-time SQL verification via macros, raw SQL (no ORM DSL to learn). Migrations built-in via `sqlx-cli`. The right choice when you already know SQL and want Rust safety on top. |
| tokio | 1.x | Async runtime | Required by both Axum and SQLx. The dominant Rust async runtime — there is no real alternative for this stack. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.0.228 | Serialization / deserialization | Everywhere: JSON request/response bodies, config, database row mapping. Enable `features = ["derive"]`. |
| serde_json | 1.x | JSON support | All API responses and request parsing. |
| tower-http | 0.6.8 | HTTP middleware (CORS, tracing, compression) | Enable features `["cors", "trace", "fs"]` at minimum. CORS needed when Leptos client fetches from Axum. |
| tracing | 0.1.x | Structured logging and spans | Use instead of `println!`. Integrates with tower-http trace middleware out of the box. |
| tracing-subscriber | 0.3.x | Log output formatting | Use `EnvFilter` feature to control verbosity per module. |
| argon2 | 0.5.3 | Password hashing | Argon2id is the OWASP-recommended algorithm for 2025. Use this over bcrypt for new projects. The RustCrypto `argon2` crate is the canonical implementation. |
| jsonwebtoken | 10.3.0 | JWT creation and verification | Straightforward, well-maintained. Required: choose either `aws_lc_rs` or `rust_crypto` feature. Use `rust_crypto` for simplicity on a learning project (no OpenSSL dependency). |
| uuid | 1.x | UUIDs for entity IDs | Use `features = ["v4", "serde"]`. Avoid auto-increment integers for user-facing IDs in social apps — UUIDs are non-enumerable. |
| chrono | 0.4.x | Date/time handling | Timestamps for posts, created_at, updated_at. Enable `serde` feature for JSON serialization. |
| dotenvy | 0.15.x | `.env` file loading | Development-only config (DATABASE_URL, JWT_SECRET). The maintained fork of the `dotenv` crate. |
| leptos_meta | (bundled with leptos) | `<head>` tag management in Leptos | Set page titles, meta tags per-route in SSR mode. |
| leptos_router | (bundled with leptos) | Client-side routing in Leptos | File-system-style routing for SPA navigation with SSR support. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo-leptos | Dev server with hot reload | `cargo leptos watch` — use this instead of `cargo run` during development |
| sqlx-cli | Database migration management | `cargo install sqlx-cli --features postgres`. Commands: `sqlx migrate add`, `sqlx migrate run` |
| sqlx offline mode | CI without live DB | Run `cargo sqlx prepare` to cache query metadata. Allows `cargo build` without DATABASE_URL set. |
| rust-analyzer | IDE support | The only real option. Configure with `check.command = "clippy"` for richer feedback. |
| cargo-watch | Auto-rebuild on file changes | Use for backend-only iterative development when not using cargo-leptos. |
| Tailwind CSS | Utility-first styling | Cargo-leptos has native Tailwind integration. Configure in `Cargo.toml` under `[package.metadata.leptos]`. No JS build step needed. |

---

## Installation

```toml
# Cargo.toml — server binary
[dependencies]
# Web server
axum = { version = "0.8", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "fs"] }

# Frontend (SSR mode)
leptos = { version = "0.8", features = ["ssr"] }
leptos_axum = { version = "0.8" }
leptos_meta = { version = "0.8" }
leptos_router = { version = "0.8", features = ["ssr"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }

# Auth
argon2 = "0.5"
jsonwebtoken = { version = "10", features = ["rust_crypto"] }

# Utilities
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
# (none required beyond cargo-leptos for this project)
```

```toml
# Cargo.toml — WASM client binary (hydrate features)
[dependencies]
leptos = { version = "0.8", features = ["hydrate"] }
leptos_meta = { version = "0.8" }
leptos_router = { version = "0.8" }
```

```bash
# Install dev tools
cargo install cargo-leptos
cargo install sqlx-cli --features postgres

# Bootstrap from official starter
cargo leptos new --git https://github.com/leptos-rs/start-axum
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Leptos | Yew | If you need a React-like virtual DOM mental model and don't care about SSR. Yew is more mature and has more stars (30.5k vs 18.5k) but lacks server-side rendering integration — it's purely a WASM SPA framework. For a learning project that wants full-stack Rust, Leptos wins. |
| Leptos | Dioxus | If you want cross-platform (desktop, mobile, web) from one codebase. Dioxus has a gentler learning curve for React developers but is less focused on web SSR. Not the right pick when your goal is web-specific Rust learning. |
| SQLx | Diesel | If you want a full ORM with a Rust DSL for queries and are comfortable with more complex setup. Diesel has stronger compile-time guarantees but is sync-first (async feels bolted on) and requires a separate DSL to learn. SQLx lets you write SQL you already know — better for a learning project. |
| SQLx | SeaORM | If you want an async ORM with a query builder API similar to ActiveRecord/Eloquent. More feature-rich than SQLx but adds abstraction layers that obscure what SQL is being sent — counterproductive for learning. |
| argon2 | bcrypt | Only if Argon2 turns out to be too slow on constrained hardware. For local dev, argon2 is fine. OWASP ranks Argon2id above bcrypt. |
| jsonwebtoken | axum-login + tower-sessions | If you want session cookies instead of JWTs. Axum-login is a real option but adds more moving parts (session store, cookie management). For a learning project, explicit JWT middleware is more educational. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Actix-web | Actor model adds conceptual overhead not needed here. Middleware system is split between Actix-native and Tower middleware, creating confusion for beginners. Performance advantage is negligible for a local learning project. | axum |
| Rocket | Still lags behind Axum on async maturity and ecosystem size. Requires nightly for some features. Smaller community. | axum |
| Diesel (as the primary DB library) | Sync-first architecture fights against Tokio async runtime. Requires a separate connection pool bridging strategy. DSL is expressive but adds an extra layer to learn. | sqlx |
| `dotenv` crate | Unmaintained since 2020. | dotenvy (maintained fork) |
| Virtual DOM (Yew-style) for this project | Learning fine-grained reactivity in Leptos maps better to Rust's ownership model. Virtual DOM erases that understanding. | leptos |
| Raw `tokio-postgres` | No compile-time query checking, more boilerplate for type mapping. Only justified when you need capabilities SQLx doesn't expose. | sqlx |
| `bcrypt` crate | Argon2id is strictly better for new projects: more memory-hard, resists GPU attacks more effectively. | argon2 |

---

## Stack Patterns by Variant

**If choosing pure SPA (no SSR):**
- Use Leptos in CSR mode with `features = ["csr"]`
- Axum becomes a pure REST API backend
- Loses SEO benefits and initial page-load performance
- Not recommended: SSR + hydration is the point of Leptos

**If you want session cookies instead of JWTs:**
- Add `tower-sessions` + `tower-sessions-sqlx-store` for DB-backed sessions
- Replace JWT middleware with session extraction in handlers
- Sessions are stateful (requires DB or Redis lookup per request) — educational but more complex

**If starting with just the API (no frontend yet):**
- Use the starter template but ignore Leptos initially
- Build and test all Axum routes with curl/httpie first
- Add Leptos SSR integration after routes are validated

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| leptos 0.8 | leptos_axum 0.8 | Must match exactly — leptos and leptos_axum versions are coupled. |
| axum 0.8 | tower-http 0.6 | tower-http 0.5 is incompatible with axum 0.8's updated Tower integration. |
| sqlx 0.8 | tokio 1.x | Requires `features = ["runtime-tokio"]` — do not use `runtime-async-std`. |
| jsonwebtoken 10 | Must pick crypto backend | Add either `features = ["aws_lc_rs"]` or `features = ["rust_crypto"]`. Missing this causes a compile error. |
| argon2 0.5 | Works standalone | No external C dependencies. Pure Rust. |

---

## Sources

- [Announcing axum 0.8.0 - Tokio blog](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — confirmed 0.8.x release date and API changes (HIGH confidence)
- [axum 0.8.8 on docs.rs](https://docs.rs/crate/axum/latest) — confirmed current stable version (HIGH confidence)
- [leptos 0.8.17 on docs.rs](https://docs.rs/crate/leptos/latest) — confirmed current version (HIGH confidence)
- [leptos-rs/start-axum GitHub](https://github.com/leptos-rs/start-axum) — official Leptos + Axum starter template (HIGH confidence)
- [sqlx 0.8.6 on docs.rs](https://docs.rs/crate/sqlx/latest) — confirmed current stable (0.9.0-alpha.1 exists but not stable) (HIGH confidence)
- [jsonwebtoken 10.3.0 on docs.rs](https://docs.rs/crate/jsonwebtoken/latest) — confirmed current version and crypto backend requirement (HIGH confidence)
- [argon2 0.5.3 on docs.rs](https://docs.rs/crate/argon2/latest) — confirmed version; 0.6.0-rc available but not stable (HIGH confidence)
- [tower-http 0.6.8 on docs.rs](https://docs.rs/crate/tower-http/latest) — confirmed current version (HIGH confidence)
- [RustCrypto/password-hashes GitHub](https://github.com/RustCrypto/password-hashes) — confirmed argon2 is the canonical RustCrypto password hashing crate (HIGH confidence)
- [Axum vs Actix-web 2025 - Medium](https://medium.com/@indrajit7448/axum-vs-actix-web-the-2025-rust-web-framework-war-performance-vs-dx-17d0ccadd75e) — ecosystem comparison (MEDIUM confidence — single source)
- [Leptos vs Yew vs Dioxus comparison 2026 - Reintech](https://reintech.io/blog/leptos-vs-yew-vs-dioxus-rust-frontend-framework-comparison-2026) — frontend framework comparison (MEDIUM confidence)
- [Diesel vs SQLx vs SeaORM comparison - Reintech](https://reintech.io/blog/diesel-vs-sqlx-vs-seaorm-rust-database-library-comparison-2026) — DB library tradeoffs (MEDIUM confidence)
- [Leptos book - cargo-leptos](https://book.leptos.dev/ssr/21_cargo_leptos.html) — official build tool documentation (HIGH confidence)
- [Full Stack Rust with Leptos - benw.is](https://benw.is/posts/full-stack-rust-with-leptos) — practical full-stack pattern confirmation (MEDIUM confidence)

---
*Stack research for: Full-stack Rust Twitter/X clone (My_X)*
*Researched: 2026-03-11*
