# Project Research Summary

**Project:** My_X — Full-stack Twitter/X clone
**Domain:** Social media application (Rust learning project)
**Researched:** 2026-03-11
**Confidence:** MEDIUM-HIGH

## Executive Summary

My_X is a Twitter/X clone built entirely in Rust — server, database layer, and browser UI. The research validates a full-stack Rust architecture as achievable and educational: Axum handles the HTTP server, Leptos provides SSR + WASM hydration for the frontend, SQLx gives compile-time-verified SQL, and PostgreSQL stores all social graph data. The distinguishing characteristic of this project is not the feature set (it is intentionally minimal) but the depth of Rust learning it enables: fine-grained reactivity, async database access, JWT middleware as Tower layers, and shared types across server and client — all in one language.

The recommended scope is deliberately narrow. Auth, create post, follow/unfollow, view profiles, and a chronological home feed constitute the entire v1. Likes, replies, retweets, media uploads, real-time updates, and search are all explicitly deferred. This is the correct call: each deferred feature represents a separate non-trivial subsystem (recursive data structures, WebSocket lifecycle, file storage, full-text indexing) that would obscure the core social graph patterns the project exists to teach.

The primary risks are tooling-related and front-loaded. Leptos SSR + hydration is the least mature part of the stack — there are fewer production examples than for comparable JS frameworks, and hydration mismatches produce cryptic errors. SQLx compile-time query checking will break CI if offline mode is not configured on day one. Argon2 password hashing must be wrapped in `spawn_blocking` or it silently degrades the async runtime under concurrent load. None of these risks are blockers — they are specific, well-documented, and preventable with the right setup steps in Phase 1.

---

## Key Findings

### Recommended Stack

The stack is settled and well-reasoned. Axum (0.8.8) is the consensus choice for new Rust web servers in 2025 — ergonomic, Tokio-native, Tower-compatible, with no actor-model overhead. Leptos (0.8.17) is the correct frontend choice for a project that wants full Rust SSR + WASM hydration; it has first-class Axum integration via `leptos_axum` and `cargo-leptos` handles the dual compilation (server binary + WASM client). SQLx (0.8.6) is the right database layer for a learning project: raw SQL (no ORM DSL), compile-time query validation, async-first, migrations included. Version compatibility between leptos/leptos_axum (must match exactly) and axum/tower-http (0.8/0.6 required) are the only critical version constraints.

**Core technologies:**
- Axum 0.8.8: HTTP server and routing — ergonomic, Tower-native, de-facto standard in 2025
- Leptos 0.8.17: Full-stack frontend (SSR + WASM hydration) — fine-grained reactivity, tight Axum integration
- cargo-leptos 0.2.39: Build coordinator — required for dual server/WASM compilation, not optional
- PostgreSQL 16+: Primary database — better Rust ecosystem support than MySQL, handles social graph queries well
- SQLx 0.8.6: Database access — async, compile-time SQL verification, raw SQL (no ORM DSL)
- argon2 0.5.3 + jsonwebtoken 10.3.0: Auth — OWASP-recommended hashing, explicit JWT middleware for learning value

**Do not use:** Actix-web (actor-model overhead), Diesel (sync-first fights Tokio), Rocket (async maturity lag), SeaORM (abstracts away what you're learning), the unmaintained `dotenv` crate.

### Expected Features

The feature scope is deliberately minimal and grounded in the project's learning objective. Every deferred feature is deferred for a specific reason — it would teach a different domain, not deepen the current one.

**Must have (v1 — table stakes):**
- User registration with email + password — root dependency for everything
- Login / logout with JWT — enables all authenticated routes
- Create a text post (280-char max) — the core product action
- View own profile with post history — closes the author loop
- View another user's profile — enables social discovery
- Follow / unfollow a user — the social graph primitive
- Home feed (posts from followed users, chronological) — the payoff for following
- Delete own post — basic content control

**Should have (v1.x — after core loop works):**
- Edit profile (display name, bio)
- Follower/following counts on profiles
- Cursor-based pagination on feed and profile post lists

**Defer to v2+:**
- Likes/reactions — separate engagement mechanic, no new Rust patterns
- Replies/threads — recursive data model, standalone learning topic
- Real-time feed updates (WebSockets) — separate async/concurrency topic
- Media uploads — file storage/object storage, separate domain
- Search — full-text indexing, separate subsystem

### Architecture Approach

The architecture is a Rust monolith in a single Cargo workspace with enforced layer boundaries via modules: domain types (no framework deps), repository layer (all SQL lives here), service layer (all business logic), thin Axum handlers (HTTP extraction only), Tower JWT middleware layer (applied to protected route groups), and Leptos frontend components (compile to WASM, also render on server for SSR). Feed generation uses pull-on-read (JOIN at query time) rather than fan-out-on-write — correct for this scale, eliminates background workers and cache invalidation complexity. Server functions (`#[server]` macro) are the primary frontend-backend integration pattern: they auto-generate type-safe HTTP stubs for WASM while running as direct function calls during SSR.

**Major components:**
1. Axum router + Tower middleware — HTTP routing, JWT validation as a layer applied to protected route groups
2. Leptos SSR + WASM — server-rendered initial HTML, client-side hydration and reactivity
3. Service layer (AuthService, PostService, FeedService, UserService) — all business rules live here
4. Repository layer (UserRepo, PostRepo, FollowRepo, FeedRepo) — all SQLx queries, maps rows to domain types
5. PostgreSQL — three core tables (users, posts, follows); feed generated via JOIN at read time

**Key patterns:**
- Leptos server functions for all frontend-backend data flow (not a separate REST client)
- JWT as Tower middleware layer, not per-handler logic
- Pull-on-read feed: single JOIN query, no precomputed feed table
- `PgPool` directly in Axum `State` (never behind a Mutex — it's already `Clone + Send + Sync`)

### Critical Pitfalls

1. **Blocking the Tokio runtime with Argon2** — wrap all password hashing/verification in `tokio::task::spawn_blocking`. Without this, concurrent login requests freeze the entire server.
2. **Leptos SSR hydration mismatches** — never use `cfg!(target_arch = "wasm32")` for conditional rendering, always include `<tbody>` in tables, gate browser-only APIs inside `Effect::new`. Verify with DevTools console on first page load.
3. **SQLx offline mode not configured** — run `cargo sqlx prepare` before any query macros are written, commit `.sqlx/` directory, set `SQLX_OFFLINE=true` in `.env`. Do this on day one or CI will be permanently broken.
4. **N+1 queries on feed** — write feed as an explicit JOIN (posts + follows + users) in a single query. Never fetch author data inside a loop over post results.
5. **Offset-based pagination producing duplicates** — use cursor-based keyset pagination `(created_at, id)` from the start. `OFFSET` pagination produces duplicate posts when new content is inserted between pages.

**Security non-negotiables:** JWT secret from environment variable only (panic at startup if missing), user ID from JWT claims only (never from request body), Argon2id with per-user salts, identical error messages for "user not found" vs "wrong password".

---

## Implications for Roadmap

Based on the dependency graph from FEATURES.md and the build order from ARCHITECTURE.md, a 5-phase structure emerges. Phases 1-2 are blocking gates; nothing else can proceed without them.

### Phase 1: Foundation — Project Scaffold + Database
**Rationale:** DB schema and offline mode setup must exist before any code can compile with SQLx query macros. `std::sync::Mutex` across `.await` errors must be prevented at the router level from the start.
**Delivers:** Runnable Axum server, PostgreSQL schema (users/posts/follows with indexes), SQLx offline mode configured, `#[axum::debug_handler]` on all handlers, domain types crate, `AppError` implementing `IntoResponse`
**Addresses:** Table stakes groundwork — all subsequent features depend on this
**Avoids:** SQLx offline mode pitfall (day-one setup), `std::sync::Mutex` anti-pattern, missing DB indexes on foreign keys
**Research flag:** Standard patterns — well-documented, skip phase research

### Phase 2: Auth — Registration, Login, JWT Middleware
**Rationale:** Auth is the root dependency for every other feature. JWT middleware must be in place before protected routes can be built. Password hashing pitfall must be solved here before it can propagate.
**Delivers:** POST /auth/register, POST /auth/login, Tower JWT middleware layer applied to protected route group, persistent login via localStorage token
**Addresses:** Registration, login/logout, persistent login table stakes
**Avoids:** Argon2 blocking Tokio runtime (`spawn_blocking`), hardcoded JWT secret (env var only), account enumeration via error messages, trusting user_id from request body
**Research flag:** Standard patterns — JWT + Argon2 in Axum is well-documented

### Phase 3: Core Social Features — Posts + Profiles
**Rationale:** Posts must exist before the follow graph is meaningful. Profile viewing depends on posts existing. These two features share the same data access patterns and belong in the same phase.
**Delivers:** POST /posts (create), DELETE /posts/:id, GET /users/:id (profile with post history), Leptos SSR scaffolding (page shell, routing), LoginPage, RegisterPage, ProfilePage components
**Addresses:** Create post, delete post, view own profile, view another user's profile
**Avoids:** Leptos SSR hydration mismatches (validate with trivial page first), SQL queries in handlers anti-pattern, separate Axum/Leptos project anti-pattern
**Research flag:** Leptos SSR/hydration integration is the highest-uncertainty area — consider a focused spike on the SSR shell before building full components

### Phase 4: Social Graph — Follow + Home Feed
**Rationale:** Follow depends on users existing (Phase 3). Feed depends on both follow data and posts (Phases 2-3). This is the payoff phase — the feature that makes the product feel like a social network.
**Delivers:** POST /users/:id/follow, DELETE /users/:id/follow, GET /feed (chronological, pull-on-read JOIN), FeedPage and follow button components with reactive updates
**Addresses:** Follow/unfollow, home feed table stakes
**Avoids:** N+1 feed queries (explicit JOIN from the start), fan-out-on-write anti-pattern, offset pagination (cursor-based from day one), follow idempotency (`ON CONFLICT DO NOTHING`), follow/unfollow requiring full page reload (Leptos `<ActionForm>`)
**Research flag:** Feed SQL query design should be validated against the DB schema before implementation — the JOIN pattern is established but the cursor pagination needs careful API design

### Phase 5: Polish — UX + v1.x Improvements
**Rationale:** Core loop is working end-to-end. Polish features improve usability without adding new Rust patterns.
**Delivers:** Edit profile, follower/following counts, Leptos `Suspense` loading states, cursor-based pagination on feed and profile post lists, post creation redirect to feed
**Addresses:** v1.x should-have features from FEATURES.md
**Avoids:** Feed loading without loading state (blank page), profile with all posts at once (paginate from start)
**Research flag:** Standard patterns — skip phase research

### Phase Ordering Rationale

- **Auth before posts:** You cannot author a post without a user identity. JWT middleware must be in place before any protected route is tested.
- **Posts before follows:** The follow graph is meaningless without content to show. Building profiles forces the SSR pattern to be validated before the more complex feed page.
- **Leptos SSR introduced in Phase 3, not Phase 1:** The DB and auth can be built and tested with curl/httpie. Adding Leptos SSR to Phase 1 would couple two complex systems before either is understood.
- **Cursor pagination decided in Phase 4:** The feed is the only paginated surface in v1. Committing to cursor-based pagination here prevents a painful rewrite if the project grows.
- **No v2 features in this roadmap:** Likes, replies, real-time, media, search are all conscious exclusions. The roadmap should not include them even as stretch goals — they reset the learning objective.

### Research Flags

Phases needing deeper research during planning:
- **Phase 3 (Leptos SSR):** The Axum + Leptos SSR + WASM hydration integration is the most complex and least-documented part of the stack. A focused spike (trivial SSR page compiling and hydrating without errors) should be the first deliverable, gating the rest of Phase 3. Worth a `/gsd:research-phase` focused on `leptos_axum` integration patterns and `cargo-leptos` configuration.

Phases with standard patterns (skip research-phase):
- **Phase 1:** DB setup, Axum scaffolding, SQLx migrations — extensively documented, official starter template exists
- **Phase 2:** JWT + Argon2 in Axum — well-documented community patterns, official examples available
- **Phase 4:** Feed SQL JOIN pattern is established; cursor pagination is documented but needs careful design (not research)
- **Phase 5:** Standard CRUD polish — no novel patterns

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All version numbers verified against docs.rs stable releases; version compatibility constraints confirmed against official changelogs |
| Features | HIGH | Core social features are well-established; scoping decisions grounded in project learning objective; competitor analysis corroborates scope |
| Architecture | MEDIUM-HIGH | Axum patterns are well-documented; Leptos SSR + server functions have fewer real-world production examples; pull-on-read feed schema is established |
| Pitfalls | MEDIUM-HIGH | Top pitfalls cross-verified against official docs, GitHub issues, and community references; Leptos hydration pitfalls based on official book guidance |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **Leptos SSR in production-scale examples:** Most Leptos tutorials show toy apps. The hydration mismatch failure modes are documented but the exact configuration for a multi-page app with authentication state is sparse. Validate with a spike before committing to the pattern in Phase 3.
- **Leptos server function auth context:** Extracting JWT claims inside a `#[server]` function (which runs on the server during SSR and over HTTP during WASM) requires careful use of `use_context()` vs `axum::extract`. The exact pattern needs validation against current leptos_axum 0.8 docs before Phase 3 implementation.
- **cargo-leptos Tailwind integration:** Documented in the Leptos book but less tested at the project scaffolding level. Confirm the `[package.metadata.leptos]` Tailwind config works with the starter template before relying on it.

---

## Sources

### Primary (HIGH confidence)
- [axum 0.8.8 — docs.rs](https://docs.rs/crate/axum/latest) — version, API patterns
- [leptos 0.8.17 — docs.rs](https://docs.rs/crate/leptos/latest) — version, SSR patterns
- [leptos-rs/start-axum — GitHub](https://github.com/leptos-rs/start-axum) — official starter template
- [Leptos Book — Server Functions](https://book.leptos.dev/server/25_server_functions.html) — server function patterns
- [Leptos Book — Hydration Bugs](https://book.leptos.dev/ssr/24_hydration_bugs.html) — hydration mismatch prevention
- [Leptos Book — cargo-leptos](https://book.leptos.dev/ssr/21_cargo_leptos.html) — build tool documentation
- [sqlx 0.8.6 — docs.rs](https://docs.rs/crate/sqlx/latest) — version, offline mode
- [jsonwebtoken 10.3.0 — docs.rs](https://docs.rs/crate/jsonwebtoken/latest) — version, crypto backend requirement
- [argon2 0.5.3 — docs.rs](https://docs.rs/crate/argon2/latest) — version, RustCrypto canonical impl
- [tower-http 0.6.8 — docs.rs](https://docs.rs/crate/tower-http/latest) — version, feature flags
- [axum middleware docs](https://docs.rs/axum/latest/axum/middleware/index.html) — Tower layer patterns
- [leptos_axum — docs.rs](https://docs.rs/leptos_axum/latest/leptos_axum/) — Axum integration API
- [SQLx offline mode — launchbadge/sqlx](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) — offline mode setup
- [Tokio blog — axum 0.8.0 announcement](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — release confirmation

### Secondary (MEDIUM confidence)
- [realworld-axum-sqlx — launchbadge](https://github.com/launchbadge/realworld-axum-sqlx) — reference implementation patterns
- [axum-social-with-tests — sqlx examples](https://github.com/launchbadge/sqlx/tree/main/examples/postgres/axum-social-with-tests) — social app pattern reference
- [rust-axum-leptos-wasm — hvalfangst](https://github.com/hvalfangst/rust-axum-leptos-wasm) — working Axum + Leptos integration example
- [LogRocket: Best Way to Structure Rust Web Services](https://blog.logrocket.com/best-way-structure-rust-web-services/) — layered service architecture patterns
- [JWT Authentication in Rust using Axum — codevoweb.com](https://codevoweb.com/jwt-authentication-in-rust-using-axum-framework/) — JWT middleware pattern
- [Password auth in Rust — Luca Palmieri](https://lpalmieri.com/posts/password-authentication-in-rust/) — Argon2 + spawn_blocking pattern
- [Keyset cursors vs offset pagination — Sequin](https://blog.sequinstream.com/keyset-cursors-not-offsets-for-postgres-pagination/) — cursor pagination rationale
- [System Design Twitter feed architecture — namastedev](https://namastedev.com/blog/system-design-of-twitter-feed/) — pull-on-read vs fan-out-on-write
- [Full Stack Rust with Leptos — benw.is](https://benw.is/posts/full-stack-rust-with-leptos) — practical full-stack pattern
- [Handling synchronous blocking in async Rust — Leapcell](https://leapcell.io/blog/handling-synchronous-blocking-in-asynchronous-rust-web-services) — spawn_blocking pattern
- [Axum debug_handler issue — tokio-rs/axum #438](https://github.com/tokio-rs/axum/issues/438) — Mutex across .await pitfall
- [Semgrep: JWT mistakes](https://semgrep.dev/blog/2020/hardcoded-secrets-unverified-tokens-and-other-common-jwt-mistakes/) — JWT security pitfalls

---
*Research completed: 2026-03-11*
*Ready for roadmap: yes*
