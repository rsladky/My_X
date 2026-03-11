# Architecture Research

**Domain:** Full-stack Rust Twitter clone (social graph — posts, follows, feed)
**Researched:** 2026-03-11
**Confidence:** MEDIUM-HIGH (Axum+Leptos patterns well-documented; social graph schema is established; Leptos SSR/server-function integration has fewer real-world examples at scale)

---

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Browser Layer                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │   Leptos WASM (hydrated, reactive)                       │   │
│  │   Routes: / /feed /profile/:id /login /register          │   │
│  └──────────────────────┬──────────────────────────────────┘   │
│                          │ Server Functions (POST /api/...)      │
└──────────────────────────┼──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                        Axum Server                              │
│  ┌─────────────────┐  ┌────────────────┐  ┌─────────────────┐  │
│  │  Leptos SSR     │  │  REST handlers │  │  JWT middleware │  │
│  │  (HTML render)  │  │  (JSON API)    │  │  (Tower layer)  │  │
│  └────────┬────────┘  └───────┬────────┘  └────────┬────────┘  │
│           │                   │                     │           │
│  ┌────────▼───────────────────▼─────────────────────▼────────┐ │
│  │                     Service Layer                          │ │
│  │  AuthService  PostService  FeedService  UserService        │ │
│  └────────────────────────┬───────────────────────────────────┘ │
│                           │                                      │
│  ┌────────────────────────▼───────────────────────────────────┐ │
│  │                   Repository Layer                         │ │
│  │  UserRepo  PostRepo  FollowRepo  FeedRepo                  │ │
│  └────────────────────────┬───────────────────────────────────┘ │
└──────────────────────────┬┴─────────────────────────────────────┘
                           │ SQLx async queries
┌──────────────────────────▼──────────────────────────────────────┐
│                       PostgreSQL                                │
│  users  posts  follows  (feed generated via JOIN at read time)  │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|----------------|-------------------|
| Leptos frontend (WASM) | Reactive UI, client-side routing, hydrated interactivity | Axum server via server functions (POST) |
| Leptos SSR (server-side) | Initial HTML render, SEO-friendly page load | Axum router, service layer (directly, not over HTTP) |
| Axum router | HTTP routing, middleware composition, serving static assets | Leptos SSR renderer, REST handlers, Tower middleware |
| JWT middleware (Tower layer) | Extract + validate Bearer token on protected routes | Injects `Claims` into request extensions |
| AuthService | Password hashing (argon2), JWT minting and validation | UserRepo, jsonwebtoken crate |
| PostService | Create post, delete post, fetch post | PostRepo |
| FeedService | Generate timeline for a user (pull-on-read from follows) | PostRepo, FollowRepo |
| UserService | Register, fetch profile, follow/unfollow | UserRepo, FollowRepo |
| Repository layer | SQL queries via SQLx, maps rows to domain types | PostgreSQL connection pool (PgPool) |
| PostgreSQL | Persistent storage for all domain data | Repository layer only |

---

## Recommended Project Structure

Two valid approaches exist. For this learning project, a **single Cargo workspace** is recommended — simpler than splitting into multiple crates while still enforcing layer boundaries through modules.

```
my_x/
├── Cargo.toml                  # workspace or single crate
├── .env                        # DATABASE_URL, JWT_SECRET
├── migrations/                 # SQLx migration files
│   ├── 20240101_create_users.sql
│   ├── 20240102_create_posts.sql
│   └── 20240103_create_follows.sql
└── src/
    ├── main.rs                 # Axum server bootstrap, router assembly
    ├── config.rs               # Env vars, app config struct
    ├── error.rs                # AppError enum implementing IntoResponse
    │
    ├── domain/                 # Pure domain types — no framework deps
    │   ├── mod.rs
    │   ├── user.rs             # User, UserId, NewUser structs
    │   ├── post.rs             # Post, PostId, NewPost structs
    │   └── follow.rs           # Follow relationship type
    │
    ├── db/                     # Repository implementations (SQLx)
    │   ├── mod.rs
    │   ├── user_repo.rs
    │   ├── post_repo.rs
    │   └── follow_repo.rs
    │
    ├── services/               # Business logic, orchestration
    │   ├── mod.rs
    │   ├── auth.rs             # AuthService: hash, verify, JWT
    │   ├── post.rs             # PostService: create, fetch
    │   ├── feed.rs             # FeedService: timeline query
    │   └── user.rs             # UserService: profile, follow/unfollow
    │
    ├── handlers/               # Thin Axum handlers (HTTP ↔ service)
    │   ├── mod.rs
    │   ├── auth.rs             # POST /auth/register, POST /auth/login
    │   ├── posts.rs            # POST /posts, GET /posts/:id
    │   ├── feed.rs             # GET /feed
    │   └── users.rs            # GET /users/:id, POST /users/:id/follow
    │
    ├── middleware/
    │   └── auth.rs             # JWT Tower middleware layer
    │
    └── frontend/               # Leptos components (compile to WASM)
        ├── mod.rs
        ├── app.rs              # Root component + leptos-router setup
        ├── pages/
        │   ├── login.rs
        │   ├── register.rs
        │   ├── feed.rs
        │   └── profile.rs
        └── components/
            ├── post_card.rs
            ├── post_form.rs
            └── user_card.rs
```

### Structure Rationale

- **domain/:** Zero framework dependencies. Types that both services and frontend can share without import cycles.
- **db/:** All SQLx queries live here. Services never write raw SQL — they call repo functions that return domain types.
- **services/:** The only layer that knows business rules (e.g., "you can't follow yourself"). Handlers are kept dumb.
- **handlers/:** Extract request data, call one service method, return response. No SQL, no auth logic here.
- **middleware/:** JWT validation runs as a Tower layer applied to protected route groups, not inside handlers.
- **frontend/:** Leptos components and server functions. When `cargo-leptos` builds, this module compiles to WASM for the browser and also runs on the server for SSR.

---

## Architectural Patterns

### Pattern 1: Leptos Server Functions (Isomorphic Calls)

**What:** The `#[server]` macro marks a function that exists in the Leptos component tree but executes only on the server. The client automatically POSTs arguments; the server returns a serialized result. No manual REST client code needed.

**When to use:** All data fetching and mutations from frontend components. Feed loading, posting a tweet, follow/unfollow.

**Trade-offs:** Simplest DX for this project. Tightly couples frontend and backend in one codebase — fine for a monolith, harder to extract later if you want a standalone mobile API.

**Example:**
```rust
#[server(CreatePost, "/api")]
pub async fn create_post(body: String) -> Result<Post, ServerFnError> {
    // Only runs on server. Has access to AppState via use_context().
    let state = expect_context::<AppState>();
    let claims = extract_claims()?;          // from request extensions
    state.post_service.create(claims.user_id, body).await
        .map_err(ServerFnError::from)
}
```

### Pattern 2: JWT as Tower Middleware Layer

**What:** A Tower `Layer` + `Service` that intercepts requests, extracts the `Authorization: Bearer <token>` header, validates the JWT, and injects parsed `Claims` into request extensions. Protected routes are grouped under `Router::layer(auth_layer)`.

**When to use:** Any route that requires authentication. Applied at the router level, not per-handler.

**Trade-offs:** Clean separation — handlers don't know about token parsing. Unprotected routes (login, register, SSR page shell) are excluded from the layer.

**Example:**
```rust
// router assembly in main.rs
let protected = Router::new()
    .route("/feed", get(feed_handler))
    .route("/posts", post(create_post_handler))
    .layer(auth_middleware_layer);        // Tower layer validates JWT

let public = Router::new()
    .route("/auth/login", post(login_handler))
    .route("/auth/register", post(register_handler));

let app = Router::new()
    .merge(protected)
    .merge(public)
    .with_state(app_state);
```

### Pattern 3: Pull-on-Read Feed (Fan-out at Read Time)

**What:** Instead of precomputing a feed table on every post write, the feed is generated at read time with a SQL JOIN across `posts` and `follows`. Simple, correct, appropriate for a learning project with small user counts.

**When to use:** Always for this project. Fan-out-on-write (precomputed feed table) only becomes necessary at tens of thousands of active users with viral accounts — far beyond scope.

**Trade-offs:** Feed query is slightly slower per request but requires no background workers, no cache invalidation, no extra tables. Correctness is trivial.

**Example:**
```sql
-- FeedRepo: get_feed_for_user
SELECT p.id, p.author_id, p.body, p.created_at, u.username
FROM posts p
JOIN follows f ON f.followee_id = p.author_id
JOIN users u   ON u.id = p.author_id
WHERE f.follower_id = $1
ORDER BY p.created_at DESC
LIMIT 50;
```

---

## Data Flow

### Request Flow: Feed Page Load (SSR + Hydration)

```
Browser navigates to /feed
    ↓
Axum receives GET /feed (no WASM yet)
    ↓
Leptos SSR renders FeedPage component on server
    ↓
FeedPage calls get_feed() server function (runs directly, not over HTTP)
    ↓
FeedService.get_feed(user_id) → FeedRepo.query(user_id) → PostgreSQL
    ↓
HTML with embedded feed data sent to browser
    ↓
WASM hydrates HTML, component becomes reactive
    ↓
User scrolls / posts → subsequent calls go over HTTP as server functions
```

### Request Flow: Create Post (Client Action)

```
User submits post form (Leptos Action)
    ↓
Leptos serializes args, POSTs to /api/CreatePost
    ↓
Axum router receives POST /api/CreatePost
    ↓
JWT middleware validates Bearer token, injects Claims into extensions
    ↓
create_post server fn runs: extracts Claims, calls PostService
    ↓
PostService validates (body not empty, length ≤ 280) → PostRepo.insert()
    ↓
SQLx executes INSERT INTO posts... RETURNING *
    ↓
Post struct returned → serialized as JSON response
    ↓
Leptos frontend receives Ok(Post), updates reactive signal, rerenders list
```

### Request Flow: Login

```
User submits login form
    ↓
POST /auth/login (public route, no JWT middleware)
    ↓
login_handler extracts Json<LoginRequest>
    ↓
AuthService.login(email, password):
    - UserRepo.find_by_email(email)
    - argon2::verify(password, stored_hash)
    - jsonwebtoken::encode(Claims { user_id, exp }, JWT_SECRET)
    ↓
Returns Json { token: "eyJ..." }
    ↓
Frontend stores token (localStorage or cookie)
    ↓
Subsequent server function calls include token in Authorization header
```

### Key Data Flows Summary

1. **Feed load:** Browser → Axum SSR → FeedService → DB → HTML → browser hydrates
2. **Post create:** WASM Action → POST /api → JWT layer → PostService → DB → reactive update
3. **Follow user:** WASM Action → POST /api → JWT layer → UserService → FollowRepo insert → reactive update
4. **Profile view:** SSR render on navigate → UserService + PostService queries → HTML

---

## Database Schema

```sql
-- Core tables. No feed table needed for v1 (pull-on-read).

CREATE TABLE users (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username   TEXT NOT NULL UNIQUE,
    email      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    bio        TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE posts (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body       TEXT NOT NULL CHECK (char_length(body) <= 280),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    followee_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (follower_id, followee_id),
    CHECK (follower_id != followee_id)   -- can't follow yourself
);

-- Performance indexes
CREATE INDEX idx_posts_author_created ON posts(author_id, created_at DESC);
CREATE INDEX idx_follows_follower     ON follows(follower_id);
CREATE INDEX idx_follows_followee     ON follows(followee_id);
```

---

## Suggested Build Order

Dependencies between components determine what must be built first.

| Phase | Component | Why This Order |
|-------|-----------|---------------|
| 1 | DB schema + migrations | Everything depends on tables existing |
| 2 | Domain types (user, post, follow) | Repos and services depend on these types |
| 3 | Repository layer (SQLx queries) | Services depend on repos; can be tested with a real DB |
| 4 | AuthService + JWT middleware | Other services depend on knowing who the user is |
| 5 | REST handlers (auth: register/login) | Validates the auth stack end-to-end before adding more handlers |
| 6 | PostService + handlers | Depends on auth being functional |
| 7 | FollowService + handlers | Depends on users existing |
| 8 | FeedService + feed query | Depends on posts + follows being populated |
| 9 | Leptos frontend (SSR shell + routing) | Depends on all API endpoints being stable |
| 10 | Leptos components per feature | Each component wires to existing server functions |

---

## Scaling Considerations

This is a local learning project. These notes are for awareness, not action.

| Scale | Architecture Adjustment |
|-------|-------------------------|
| 0-1k users | Current monolith is fine. Pull-on-read feed is fast enough. |
| 1k-100k users | Add Redis cache for feed. Index on `follows(follower_id)` already in place. Connection pool tuning. |
| 100k+ users | Fan-out-on-write feed table. Read replicas. Likely move to microservices — far beyond scope. |

**First bottleneck:** Feed query (JOIN across posts + follows grows linearly with follows count). Fix: cache per-user feed in Redis with TTL, invalidate on new post from followee.

**Second bottleneck:** Connection pool saturation under high concurrency. Fix: tune `PgPoolOptions::max_connections`, add a read replica.

---

## Anti-Patterns

### Anti-Pattern 1: SQL Queries in Handlers

**What people do:** Write `sqlx::query!(...)` directly inside Axum handler functions.
**Why it's wrong:** Handlers become untestable, business logic is scattered, SQL leaks into HTTP layer, impossible to reuse queries.
**Do this instead:** Handlers call services; services call repositories; repositories own all SQL.

### Anti-Pattern 2: Two Separate Projects (Axum API + Leptos SPA)

**What people do:** Build Axum as a pure REST API and Leptos as a separate SPA talking to it via fetch.
**Why it's wrong:** Loses the main benefit of Leptos (server functions, SSR, shared types). Doubles the effort and removes type safety across the boundary.
**Do this instead:** Use `cargo-leptos` with a single workspace. Leptos server functions call service layer directly on the server — no HTTP roundtrip for SSR, automatic type-safe client stubs for WASM.

### Anti-Pattern 3: JWT in Cookie Without CSRF Protection

**What people do:** Store JWT in an HttpOnly cookie for convenience (avoids JS access), but forget to add CSRF tokens.
**Why it's wrong:** Cookie-based auth is vulnerable to CSRF attacks. Simplest fix for a learning project is to use `Authorization: Bearer` header with localStorage — no CSRF risk.
**Do this instead:** For v1, store JWT in localStorage and send as `Authorization: Bearer <token>` header. Accept the XSS tradeoff; it's fine for a local learning project.

### Anti-Pattern 4: Fan-out-on-Write for v1

**What people do:** Pre-build a feed table on every post insert (one row per follower).
**Why it's wrong:** Complex to implement correctly, needs background workers or triggers, no benefit at small scale, a major distraction from the learning goal.
**Do this instead:** Pull-on-read feed with a single JOIN. Add precomputation only if performance data shows it's needed.

---

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Handler ↔ Service | Direct Rust function call via `AppState` | Service injected into handler via `State<AppState>` extractor |
| Service ↔ Repository | Direct Rust function call | Repo takes `&PgPool` from AppState |
| Leptos component ↔ Backend | `#[server]` function (POST over HTTP when in WASM; direct call when SSR) | Serialization is automatic via serde |
| JWT middleware ↔ Handler | Axum request extensions | Middleware inserts `Claims`; handler extracts with `Extension<Claims>` |

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| PostgreSQL | SQLx PgPool, compile-time checked queries | Run locally via Docker or system install |
| argon2 (password hashing) | Synchronous call in AuthService | Block with `spawn_blocking` if on async thread |
| jsonwebtoken | Called in AuthService.mint/verify | Symmetric HS256 with env-var secret for v1 |

---

## Sources

- [Leptos Book — Server Functions](https://book.leptos.dev/server/25_server_functions.html) — HIGH confidence (official docs)
- [leptos_axum crate docs](https://docs.rs/leptos_axum/latest/leptos_axum/) — HIGH confidence (official)
- [LogRocket: Best Way to Structure Rust Web Services](https://blog.logrocket.com/best-way-structure-rust-web-services/) — MEDIUM confidence (community, verified with patterns)
- [JWT Authentication in Rust using Axum Framework](https://codevoweb.com/jwt-authentication-in-rust-using-axum-framework/) — MEDIUM confidence (community tutorial, 2025)
- [axum middleware docs](https://docs.rs/axum/latest/axum/middleware/index.html) — HIGH confidence (official)
- [rust-axum-leptos-wasm example](https://github.com/hvalfangst/rust-axum-leptos-wasm) — MEDIUM confidence (real working project demonstrating the pattern)
- [System Design Twitter feed architecture](https://namastedev.com/blog/system-design-of-twitter-feed/) — MEDIUM confidence (industry-standard pattern, pull-on-read vs fan-out-on-write)
- [Twitter database schema design](https://drawsql.app/templates/twitter) — MEDIUM confidence (community reference)

---
*Architecture research for: Full-stack Rust Twitter clone (My_X)*
*Researched: 2026-03-11*
