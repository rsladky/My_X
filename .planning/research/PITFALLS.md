# Pitfalls Research

**Domain:** Full-stack Rust Twitter clone (Axum + Leptos + PostgreSQL + SQLx + JWT)
**Researched:** 2026-03-11
**Confidence:** MEDIUM-HIGH (cross-verified against official docs, GitHub issues, and community forums)

---

## Critical Pitfalls

### Pitfall 1: Blocking the Tokio Runtime with CPU-Intensive Work

**What goes wrong:**
Password hashing (Argon2id, bcrypt) takes 100–500ms of pure CPU work. Running it directly inside an `async fn` handler blocks the Tokio worker thread for that entire duration. While one login is hashing, every other request waiting on that thread stalls — the server appears to freeze under any concurrent load.

**Why it happens:**
Developers write `let hash = argon2.hash_password(...)` directly in the handler, treating it like a normal await. The mistake is invisible during solo testing but surfaces immediately under any concurrent traffic (two browser tabs logging in at once).

**How to avoid:**
Always wrap CPU-heavy work in `tokio::task::spawn_blocking`:

```rust
let hash = tokio::task::spawn_blocking(move || {
    Argon2::default().hash_password(password.as_bytes(), &salt)
})
.await??;
```

Use `argon2` crate from RustCrypto, not `bcrypt`, because Argon2id is OWASP-recommended and the Rust implementation is well-maintained.

**Warning signs:**
- Login/register endpoints take 200-500ms even with 1 user
- Requests to unrelated endpoints (e.g., GET feed) stall during a simultaneous login
- `tokio-console` shows task poll durations spiking on auth handlers

**Phase to address:** Auth phase (user registration and login implementation)

---

### Pitfall 2: `std::sync::Mutex` Held Across `.await` Points

**What goes wrong:**
When sharing state between handlers using `Arc<Mutex<T>>`, holding a `std::sync::Mutex` lock while calling `.await` produces a `!Send` future. Axum requires all handlers to return `Send` futures (it uses a multi-threaded runtime). The compiler error is cryptic and the `#[debug_handler]` attribute is the only practical rescue.

**Why it happens:**
Developers copy state-sharing patterns from synchronous Rust code. `std::sync::Mutex` is the obvious import. The error message from Axum about trait bounds not being satisfied does not directly identify the mutex as the cause.

**How to avoid:**
- Use `tokio::sync::Mutex` when the lock must be held across `.await`
- Prefer scoping the lock so it drops before any `.await`:
  ```rust
  let value = {
      let guard = state.lock().unwrap();
      guard.clone() // drop guard here
  };
  do_async_work(value).await;
  ```
- For database state, use `sqlx::PgPool` directly — it is `Clone + Send + Sync` and handles connection pooling internally. Do not wrap it in a `Mutex`.
- Always add `#[axum::debug_handler]` to handlers during development to get readable error messages.

**Warning signs:**
- Compiler error mentioning `std::future::Future` not implementing `Send`
- Handler that compiles individually but fails when registered with `Router`
- State wrapped in `Arc<std::sync::Mutex<PgPool>>` (red flag — pool should never be behind a Mutex)

**Phase to address:** Project scaffolding / backend foundation phase

---

### Pitfall 3: SQLx Compile-Time Checking Breaks CI Without Offline Mode

**What goes wrong:**
SQLx's `query!` macro connects to a live PostgreSQL database at compile time to validate SQL. Without proper offline mode setup, the project cannot be compiled without a running database. CI systems, fresh clones, and teammates without local Postgres all break.

**Why it happens:**
Developers discover `query!` macro, love the compile-time verification, add it everywhere, then never set up `cargo sqlx prepare`. The first CI run or fresh clone fails with an opaque error about DATABASE_URL.

**How to avoid:**
1. Run `cargo sqlx prepare` after every schema or query change to generate `.sqlx/` metadata files
2. Commit the `.sqlx/` directory to version control
3. Set `SQLX_OFFLINE=true` in `.env` for development (prevents accidental live DB compilation)
4. In CI, set `SQLX_OFFLINE=true` as an environment variable

**Warning signs:**
- `error: DATABASE_URL must be set to use query macros` in CI
- Build fails for a teammate who hasn't run the migrations yet
- `.sqlx/` directory is in `.gitignore` (wrong — it must be committed)

**Phase to address:** Project scaffolding phase — set this up on day one before any query macros are written

---

### Pitfall 4: Leptos SSR Hydration Mismatches

**What goes wrong:**
When running Leptos in SSR+hydration mode, the server renders HTML and sends it to the browser, then the WASM bundle "hydrates" — attaching event listeners to the existing DOM. If the client renders different HTML than the server did (even one extra or missing node), hydration panics or silently breaks interactivity.

**Why it happens:**
Common causes:
- Rendering content conditionally based on `cfg!(target_arch = "wasm32")` — server and client see different branches
- Using `<table>` without an explicit `<tbody>` — browsers insert one automatically, creating a DOM mismatch
- Calling browser-only APIs (gloo, web-sys) during server render, causing a panic
- Depending on server-only crates (filesystem, process) in WASM, which won't compile

**How to avoid:**
- Never use `cfg!(target_arch = "wasm32")` to conditionally render different element counts
- Always include `<tbody>` explicitly in `<table>` elements
- Gate browser-only code inside `Effect::new(|_| { ... })` — effects only run client-side
- Mark server-only Cargo dependencies as `optional = true` and enable them only under the `ssr` feature
- Enable only ONE of `csr`, `hydrate`, or `ssr` features per build target

**Warning signs:**
- Browser console shows hydration errors or panics on page load
- Page renders correctly server-side but loses interactivity after WASM loads
- Compiler errors mentioning `mio` when building for WASM (server-only dep leaking into client build)
- rust-analyzer showing errors for code that actually compiles (feature flag configuration issue)

**Phase to address:** Frontend scaffolding phase — get SSR/hydration working with a trivial page before building any real UI

---

### Pitfall 5: N+1 Queries on Feed and Profile Pages

**What goes wrong:**
The feed query fetches a list of post IDs, then the code loops over them fetching each post's author separately — one query per post. For a feed of 20 posts, that's 21 database round-trips. At any real-world scale this is catastrophic, but it's also slow enough to be noticeable even locally.

**Why it happens:**
SQLx is not an ORM. It has no automatic relationship loading. Developers write `query_as!` to fetch posts, then reach for a helper function like `get_user_by_id()` inside a loop, not realizing each call is a separate SQL query.

**How to avoid:**
Write feed queries as explicit JOINs that fetch everything in one shot:

```sql
SELECT
    p.id, p.content, p.created_at,
    u.id AS author_id, u.username, u.display_name
FROM posts p
JOIN users u ON u.id = p.user_id
JOIN follows f ON f.followed_id = p.user_id
WHERE f.follower_id = $1
ORDER BY p.created_at DESC
LIMIT $2
```

For cases where JOINs are impractical, use `ANY($1::uuid[])` to batch-fetch by IDs in a single query rather than one query per ID.

**Warning signs:**
- EXPLAIN ANALYZE shows many sequential small queries instead of one join
- Feed endpoint latency scales linearly with the number of posts in the feed
- Database logs show repeated `SELECT * FROM users WHERE id = $1` with different IDs

**Phase to address:** Feed feature phase — design the SQL upfront, don't refactor it after the fact

---

### Pitfall 6: Offset-Based Feed Pagination Producing Duplicate or Missing Posts

**What goes wrong:**
Implementing feed pagination with `LIMIT 20 OFFSET 40` causes posts to appear twice or be skipped when new posts are inserted between page requests. A user scrolling through their feed sees the same tweet twice on page 2.

**Why it happens:**
Offset pagination is the intuitive first approach. `OFFSET N` tells Postgres to skip N rows, but "row 40" shifts whenever new content is inserted — what was row 41 becomes row 42, appearing again on page 2.

**How to avoid:**
Use cursor-based keyset pagination from the start. The cursor is the `(created_at, id)` pair of the last seen post:

```sql
SELECT p.*, u.username FROM posts p
JOIN follows f ON f.followed_id = p.user_id
WHERE f.follower_id = $1
  AND (p.created_at, p.id) < ($2, $3)  -- cursor
ORDER BY p.created_at DESC, p.id DESC
LIMIT 20
```

The compound `(created_at, id)` cursor is safe because `id` (UUID or serial) breaks ties when multiple posts share the same timestamp. Return the cursor values of the last post in each response for the client to use in the next request.

**Warning signs:**
- Duplicate posts appear when scrolling from page 1 to page 2 while another user is actively posting
- Pagination is implemented with `?page=2&per_page=20` instead of `?cursor=<value>`
- SQL query contains `OFFSET` without an immutable sort key

**Phase to address:** Feed feature phase — commit to cursor pagination in the initial schema design

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `query!` without `cargo sqlx prepare` | Faster initial development | CI breaks, fresh clones fail | Never — set up offline mode from day one |
| `unwrap()` on database errors in handlers | Less boilerplate | Panics crash the whole server, no useful error messages | Never in handlers — use `?` with `IntoResponse` impl |
| Storing JWT secret in source code | Zero config friction | Security vulnerability — token forgery if code is ever shared | Never — even for a learning project, use `.env` |
| `OFFSET`-based pagination | Simpler to implement | Duplicate posts for users, rewrite required when noticed | Only if you will never add new posts during a session |
| Skipping `#[axum::debug_handler]` | N/A | Unreadable trait bound errors that take hours to diagnose | Never — always add it during development |
| Not wrapping Argon2 in `spawn_blocking` | Slightly less boilerplate | Starves the async runtime, server freezes under concurrent auth load | Never |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| SQLx + Axum State | Wrapping `PgPool` in `Arc<Mutex<PgPool>>` | `PgPool` is already `Clone + Send + Sync` — put it directly in Axum `State` |
| Leptos + Axum (full-stack) | Running Leptos server functions on a separate port or process | Leptos server functions must be served by the same Axum router — register them with `leptos_axum::LeptosRoutes` |
| SQLx offline mode + DATABASE_URL | Having `DATABASE_URL` set overrides `SQLX_OFFLINE=true` | Unset `DATABASE_URL` when running `cargo sqlx prepare` in offline mode if you hit conflicts |
| JWT + Argon2 crates | Using the `jsonwebtoken` crate with a weak or hardcoded secret | Generate a strong secret via `openssl rand -base64 32` and load it from `.env` at startup, never default to a hardcoded fallback |
| Leptos features + Cargo workspaces | Enabling `ssr` globally instead of per-target | Configure features per `[[bin]]` target in Cargo.toml or via `cargo leptos` build tool |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| No index on `follows(follower_id)` | Feed query scans the entire follows table | Add `CREATE INDEX ON follows(follower_id)` in migrations | After ~1000 follow relationships |
| No index on `posts(user_id, created_at DESC)` | Feed JOIN becomes a full table scan | Composite index on `(user_id, created_at DESC)` | After ~10,000 posts |
| SQLx connection pool too small | Requests queue waiting for a connection, timeouts | Set pool size to at least `(num_cpu_cores * 2) + 1`, typically 10 for local dev | Under any concurrent load |
| Fetching full user rows to display feed author | Unnecessary data transfer (avatar URLs, bios, etc.) for a simple username | SELECT only the columns actually needed in each query | Noticeable at any scale — it's just waste |
| Leptos full page re-renders on every interaction | UI feels slow and flickery | Use fine-grained reactivity signals — update only the specific DOM nodes that change | Immediately noticeable in the feed with 20+ items |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Hardcoded JWT secret in source code | Anyone who reads the source (or it leaks) can forge tokens for any user | Load secret from environment variable, panic at startup if not set — no fallback default |
| Using HS256 with a short or guessable secret | JWT can be brute-forced offline using a known token | Use at least 256 bits of entropy; `openssl rand -base64 32` generates a suitable secret |
| Not verifying JWT expiration (`exp` claim) | Stolen tokens work forever | Always validate `exp` claim when decoding; the `jsonwebtoken` crate validates it by default if `Validation::new()` is used correctly |
| Storing password in plain text or with MD5/SHA | Passwords exposed on DB breach | Use `argon2` crate (Argon2id variant) with per-user salts |
| Returning different error messages for "user not found" vs "wrong password" | Account enumeration — attacker can discover valid usernames | Return identical error message: "Invalid email or password" for both cases |
| No rate limiting on login endpoint | Credential stuffing attacks | Acceptable to skip for a local learning project, but document the gap |
| Trusting `user_id` from the request body instead of the JWT | User A can post as User B | Extract the authenticated user's ID exclusively from the validated JWT claims, never from request body/query params |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Feed shows no loading state while fetching | Page appears blank/frozen during data fetch | Use Leptos `Suspense` component to show a skeleton loader while resources resolve |
| Follow/unfollow requires full page reload | Jarring interaction for a social app | Use a Leptos server action (`<ActionForm>`) that updates the follow button reactively without navigation |
| No optimistic UI on follow | Button feels laggy even on fast connections | Show the new state immediately, revert on error |
| Posting a tweet redirects to a blank page | Confusing — user doesn't know if it worked | After successful post creation, redirect to feed or show the new post inline |
| Profile page loads all tweets at once | Slow initial load for users with many posts | Paginate the profile tweet history from the start |

---

## "Looks Done But Isn't" Checklist

- [ ] **Auth:** JWT validation checks the `exp` claim — a token that "works" may never expire if validation is misconfigured
- [ ] **Auth:** Password hashing runs inside `spawn_blocking` — it compiles and works without it, but blocks the runtime
- [ ] **Feed:** Feed query uses a JOIN, not a loop — verify by checking the number of database queries per request in logs
- [ ] **Feed:** Pagination uses cursor-based approach — `OFFSET` appears to work until new posts are inserted during a session
- [ ] **Follow:** Follow and unfollow are idempotent — double-clicking follow must not create duplicate rows or 500 errors (use `ON CONFLICT DO NOTHING`)
- [ ] **Leptos SSR:** Hydration works without client-side panics — open DevTools console on first page load and check for errors
- [ ] **SQLx offline mode:** CI can build without a live database — verify by running `cargo build` with `SQLX_OFFLINE=true` and no DATABASE_URL
- [ ] **Database:** All foreign keys have indexes — Postgres does NOT automatically create indexes on foreign key columns
- [ ] **Security:** JWT secret is loaded from env at startup — if `JWT_SECRET` env var is missing, the app should refuse to start
- [ ] **Security:** User ID in post creation comes from JWT claims, not request body

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Offset pagination baked into frontend and backend | HIGH | Redesign API response shape (add cursor field), update all pagination call sites, coordinate frontend changes |
| N+1 queries discovered after feature is shipped | MEDIUM | Rewrite affected queries as JOINs; no schema changes needed, but query logic must be audited throughout |
| JWT secret hardcoded and code shared publicly | HIGH | Rotate secret immediately (all existing tokens invalidated, all users logged out), move to env var |
| Hydration mismatches cause broken UI | MEDIUM | Bisect which component causes mismatch; usually one `cfg!` block or one invalid HTML nesting |
| `std::sync::Mutex` causing `!Send` errors | LOW | Swap import to `tokio::sync::Mutex` or restructure to drop lock before `.await`; mechanical change |
| Missing `cargo sqlx prepare` setup | LOW | Run `cargo sqlx prepare`, commit `.sqlx/`, set `SQLX_OFFLINE=true` in `.env`; takes ~30 minutes |
| Argon2 blocking the runtime | LOW | Wrap existing hash/verify calls in `spawn_blocking`; find-and-replace pattern |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Tokio runtime blocking (Argon2) | Auth implementation phase | Verify with two concurrent login requests — neither should block the other |
| `std::sync::Mutex` across `.await` | Project scaffolding phase | Add `#[axum::debug_handler]` to all handlers; compile with warnings-as-errors |
| SQLx offline mode | Project scaffolding phase (day one) | Run `SQLX_OFFLINE=true cargo build` with no DATABASE_URL in environment |
| Leptos SSR hydration | Frontend scaffolding phase | Open browser DevTools console — zero hydration errors on initial page load |
| N+1 queries on feed | Feed feature phase | EXPLAIN ANALYZE the feed query — must show a single join, not repeated index scans |
| Offset pagination | Feed feature phase | Verify API returns a `cursor` field, no `offset` parameter exists |
| Follow idempotency | Follow feature phase | Call follow endpoint twice in a row — must return 200, not 500 or constraint error |
| JWT secret in source | Auth implementation phase | `grep -r "JWT_SECRET\|secret" src/` must show only env var reads, never string literals |
| Trusting user_id from request | Auth implementation phase | Attempt to post as another user by sending their ID in the body — must be rejected |
| Missing DB indexes | Database schema phase | Run `\d follows` and `\d posts` in psql — verify indexes exist on foreign key columns |

---

## Sources

- [Axum debug_handler and Mutex issue — tokio-rs/axum #438](https://github.com/tokio-rs/axum/issues/438)
- [SQLx offline mode documentation — launchbadge/sqlx](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- [SQLx SQLX_OFFLINE=true in .env bug — Issue #3836](https://github.com/launchbadge/sqlx/issues/3836)
- [Leptos hydration bugs — official book](https://book.leptos.dev/ssr/24_hydration_bugs.html)
- [Password auth in Rust from scratch — Luca Palmieri (Zero To Production author)](https://lpalmieri.com/posts/password-authentication-in-rust/)
- [Handling synchronous blocking in async Rust — Leapcell](https://leapcell.io/blog/handling-synchronous-blocking-in-asynchronous-rust-web-services)
- [Hardcoded secrets, unverified tokens, JWT mistakes — Semgrep](https://semgrep.dev/blog/2020/hardcoded-secrets-unverified-tokens-and-other-common-jwt-mistakes/)
- [Keyset cursors vs offset pagination — Sequin](https://blog.sequinstream.com/keyset-cursors-not-offsets-for-postgres-pagination/)
- [SQLx lifetime errors with generic executor — Rust Forum](https://users.rust-lang.org/t/meaning-of-lifetime-errors-with-generic-sqlx-function/123859)
- [Realworld Axum SQLx reference implementation — launchbadge](https://github.com/launchbadge/realworld-axum-sqlx)
- [Axum social app with tests (SQLx official example)](https://github.com/launchbadge/sqlx/tree/main/examples/postgres/axum-social-with-tests)
- [Full Stack Rust with Leptos — Ben Wishovich](https://benw.is/posts/full-stack-rust-with-leptos)

---
*Pitfalls research for: Full-stack Rust Twitter clone (Axum + Leptos + PostgreSQL + SQLx + JWT)*
*Researched: 2026-03-11*
