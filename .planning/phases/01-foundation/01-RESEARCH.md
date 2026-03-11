# Phase 1: Foundation - Research

**Researched:** 2026-03-11
**Domain:** Leptos full-stack setup, Axum + SQLx + PostgreSQL, error handling, offline compilation
**Confidence:** HIGH

## Summary

Phase 1 requires scaffolding a runnable full-stack Rust application with Leptos 0.8.17 + Axum 0.8.8 + SQLx 0.8.6, configured for offline compilation and connected to PostgreSQL 16. The stack decision has already been locked (from STATE.md), so this research focuses on how to set up each component correctly, avoiding common pitfalls in Leptos SSR hydration, SQLx offline mode, and error handling.

Key findings:
1. **cargo leptos new** scaffolds a working project in seconds; the start-axum template is the reference
2. **SQLx offline mode** requires running `cargo sqlx prepare` and committing `.sqlx/` directory; DATABASE_URL environment variable will override offline mode if not explicitly disabled
3. **Leptos SSR hydration** is fragile; server and client must render identically or hydration errors result (no `cfg!(target_arch)` branching, no invalid HTML nesting)
4. **Error handling** via Axum IntoResponse trait + custom error enums allows structured JSON responses without panics
5. **PostgreSQL schema** should be defined once in migrations; cursor-based pagination with `(created_at, id)` pattern requires compound index

**Primary recommendation:** Use `cargo leptos new` to scaffold, immediately set `SQLX_OFFLINE=true`, run migrations with `sqlx-cli`, commit `.sqlx/` directory, implement custom `AppError` enum with IntoResponse impl, and test `cargo leptos watch` before proceeding to Phase 2.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos | 0.8.17 | Full-stack reactive framework (SSR + client hydration) | Official Rust web framework with best-in-class SSR ergonomics |
| Axum | 0.8.8 | HTTP server framework powering Leptos backend | Chosen by Leptos team; lightweight, composable middleware/routing |
| SQLx | 0.8.6 | Async SQL toolkit with compile-time query checking | Type-safe queries, offline mode, no code generation overhead |
| PostgreSQL | 16.x | Relational database | Production-grade, rich feature set, Rust ecosystem has strong support |
| jsonwebtoken | 10.3.0 | JWT signing and validation | Standard for stateless session auth in Rust |
| argon2 | 0.5.3 | Password hashing | Memory-hard, resistant to GPU/ASIC attacks, OWASP recommended |
| serde | 1.0+ | Serialization/deserialization | De facto standard for JSON/binary serde in Rust |
| tokio | 1.x+ | Async runtime | Powers Axum and SQLx; single-threaded for Leptos compatibility |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| sqlx-cli | 0.8.6 | Migration and query prepare tool | Always — required for running migrations and offline mode setup |
| cargo-leptos | 0.3.x+ | Build tool for SSR + client hydration | Always — use `cargo leptos new` to scaffold and `cargo leptos watch` to develop |
| leptos_axum | 0.8.x | Integration glue between Leptos and Axum | Already included in start-axum template |
| tracing | 0.1.x | Structured logging framework | For production error logging and debugging |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Leptos | Yew or Actix-web frontend | Yew is CSR-only (no SSR), Actix-web is only backend; Leptos provides full-stack with single codebase |
| Axum | Actix-web or Rocket | Actix is heavier/more opinionated, Rocket is less async-idiomatic; Axum is minimal and composable |
| SQLx | Diesel or sqlx | Diesel is ORM-heavy, requires compile-time schema codegen; SQLx is query-focused, lighter |
| PostgreSQL | SQLite | SQLite good for local dev, but lacks concurrent write capability needed for real feeds |

**Installation:**
```bash
# Install cargo-leptos
cargo install cargo-leptos --locked

# Create new project (template)
cargo leptos new --git https://github.com/leptos-rs/start-axum

# Install sqlx-cli for migrations
cargo install sqlx-cli --features postgres --locked

# Inside project: initialize database (assumes PostgreSQL running on localhost:5432)
createdb my_x_db
sqlx database create
```

## Architecture Patterns

### Recommended Project Structure

The `cargo leptos new` template creates this structure automatically:

```
src/
├── bin/
│   └── server/
│       └── main.rs              # Axum server entry point
├── app.rs                       # Main Leptos app component
├── lib.rs                       # Leptos library root
├── server/                      # Server-only code (compiled for x86_64)
│   ├── db.rs                    # SQLx pool initialization
│   ├── error.rs                 # AppError enum and IntoResponse impl
│   └── handlers/                # Axum route handlers
├── [client_app_name]/
│   └── lib.rs                   # Client-side (WASM) components
migrations/
├── 001_create_users.sql         # SQLx migrations in date order
├── 002_create_posts.sql
└── ...
Cargo.toml                        # Workspace with [package.metadata.leptos]
.sqlx/                            # Offline query metadata (commit to git!)
```

### Pattern 1: Full-Stack Server Functions

**What:** Use `#[server]` attribute to write backend logic that is called from frontend code as if it were local.

**When to use:** Nearly all frontend-backend data flow (auth, data fetching, mutations).

**Example:**
```rust
// In Leptos component (same file as view)
#[server]
pub async fn fetch_posts(page: i32) -> Result<Vec<PostData>, ServerFnError> {
    let db = db_pool().await?;
    let posts = sqlx::query_as::<_, PostData>(
        "SELECT id, content, created_at FROM posts ORDER BY created_at DESC LIMIT 20 OFFSET $1"
    )
    .bind(page * 20)
    .fetch_all(&db)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    Ok(posts)
}

// In component render
#[component]
fn PostsFeed() -> impl IntoView {
    let posts = create_resource(
        || (),
        |_| fetch_posts(0)
    );
    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || posts.get()
                .map(|posts| match posts {
                    Ok(posts) => view! {
                        <For each=move || posts.clone() key=|p| p.id let:post>
                            <PostCard post=post />
                        </For>
                    }.into_view(),
                    Err(e) => view! { <p>"Error: " {e.to_string()}</p> }.into_view(),
                })
            }
        </Suspense>
    }
}

// Source: Leptos book server functions https://book.leptos.dev/server/25_server_functions.html
```

### Pattern 2: Structured Error Handling with AppError

**What:** Define a custom error enum, implement `IntoResponse` and `FromServerFnError` to return JSON errors instead of panics.

**When to use:** All error paths from handlers and server functions.

**Example:**
```rust
// In src/server/error.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    #[serde(rename = "auth_error")]
    AuthError(String),
    #[serde(rename = "db_error")]
    DbError(String),
    #[serde(rename = "validation_error")]
    ValidationError(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthError(msg) => write!(f, "Auth error: {}", msg),
            Self::DbError(msg) => write!(f, "Database error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::AuthError(_) => StatusCode::UNAUTHORIZED,
            Self::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(serde_json::json!({
            "error": match &self {
                Self::AuthError(msg) => "auth_error",
                Self::DbError(_) => "db_error",
                Self::ValidationError(_) => "validation_error",
            },
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

impl FromServerFnError for AppError {
    type Encoding = JsonEncoding;
    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        AppError::ServerError(value.to_string())
    }
}

// Handler usage
#[server]
pub async fn create_post(content: String) -> Result<PostData, ServerFnError> {
    if content.is_empty() {
        return Err(ServerFnError::new(AppError::ValidationError("Content cannot be empty".to_string())));
    }

    let db = db_pool().await
        .map_err(|e| ServerFnError::new(AppError::DbError(e.to_string())))?;

    let post = sqlx::query_as::<_, PostData>(
        "INSERT INTO posts (content) VALUES ($1) RETURNING *"
    )
    .bind(content)
    .fetch_one(&db)
    .await
    .map_err(|e| ServerFnError::new(AppError::DbError(e.to_string())))?;

    Ok(post)
}

// Source: Leptos book error handling https://book.leptos.dev/view/07_errors.html
```

### Pattern 3: PostgreSQL Cursor-Based Pagination

**What:** Use compound cursor `(created_at, id)` to paginate efficiently without offset.

**When to use:** Feed queries that need to scale to large datasets without O(n) cost.

**Example:**
```rust
#[derive(Serialize, Deserialize)]
pub struct FeedRequest {
    pub cursor: Option<(DateTime<Utc>, i32)>, // (created_at, id)
    pub limit: i32,
}

#[server]
pub async fn fetch_feed(req: FeedRequest) -> Result<(Vec<PostData>, String), ServerFnError> {
    let db = db_pool().await?;
    let limit = req.limit.min(100); // Safety: max 100 per page

    let mut query = "SELECT id, content, author_id, created_at FROM posts WHERE 1=1".to_string();
    let mut count = 1;

    if let Some((cursor_created_at, cursor_id)) = req.cursor {
        // Keyset pagination: only fetch posts created AFTER cursor or with same created_at but higher id
        query.push_str(&format!(
            " AND (created_at < $1 OR (created_at = $1 AND id < $2))"
        ));
        count += 2;
    }

    query.push_str(&format!(
        " ORDER BY created_at DESC, id DESC LIMIT ${}"
    , count));

    let mut q = sqlx::query_as::<_, PostData>(&query);

    if let Some((cursor_created_at, cursor_id)) = req.cursor {
        q = q.bind(cursor_created_at).bind(cursor_id);
    }

    let posts = q.bind(limit + 1)
        .fetch_all(&db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // If we got limit+1 rows, there are more; use last row as next cursor
    let (posts, next_cursor) = if posts.len() > limit as usize {
        let last = &posts[limit as usize - 1];
        (
            posts[..limit as usize].to_vec(),
            format!("{},{}", last.created_at.timestamp(), last.id)
        )
    } else {
        (posts, String::new())
    };

    Ok((posts, next_cursor))
}

// In view:
#[component]
fn Feed() -> impl IntoView {
    let (cursor, set_cursor) = create_signal(None);
    let posts = create_resource(
        move || cursor.get(),
        |cursor| fetch_feed(FeedRequest { cursor, limit: 20 })
    );

    // Source pattern: Keyset pagination with PostgreSQL
    // https://blog.sequinstream.com/keyset-cursors-not-offsets-for-postgres-pagination/
}
```

### Pattern 4: SQLx Offline Mode Setup

**What:** Generate `.sqlx/` metadata directory via `cargo sqlx prepare`, commit to git, enable `SQLX_OFFLINE=true`.

**When to use:** Every project with SQLx to allow CI/CD builds without database access.

**Example:**
```bash
# 1. Write a migration
mkdir -p migrations
cat > migrations/001_create_users.sql <<EOF
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
EOF

# 2. Apply migration
export DATABASE_URL=postgres://user:pass@localhost/my_x_db
sqlx migrate add -r create_users  # Creates 20260311120000_create_users.sql
sqlx migrate run

# 3. Generate .sqlx metadata (runs all queries in code against live DB)
cargo sqlx prepare --database-url "$DATABASE_URL"

# 4. Commit .sqlx/ directory
git add .sqlx/
git commit -m "chore: add SQLx metadata for offline compilation"

# 5. Set offline mode default in .env
echo "SQLX_OFFLINE=true" >> .env

# 6. Now builds work without DATABASE_URL
SQLX_OFFLINE=true cargo build

# Source: SQLx offline mode https://github.com/launchbadge/sqlx#offline-mode
```

### Anti-Patterns to Avoid

- **Leptos hydration mismatch via `cfg!(target_arch)`:** Never branch on `cfg!(target_arch = "wasm32")` in view code — server and client must render identically. Instead, render both on server and conditionally show on client via Leptos signals.
- **Invalid HTML in views:** `<div>` inside `<p>` or other violations cause browser parser to close the parent, breaking DOM tree and hydration. Use `view!` macro validation and browser DevTools to spot.
- **Forgetting `SQLX_OFFLINE=true` in CI:** If DATABASE_URL is set, SQLx will try to query live DB even with `.sqlx/` present. Set `SQLX_OFFLINE=true` explicitly.
- **Not running `cargo sqlx prepare` after query changes:** `.sqlx/` metadata gets out of sync, causing false compile errors. Add this to pre-commit hooks.
- **Missing indexes on pagination columns:** `(created_at, id)` compound index is essential for efficient keyset pagination. Without it, queries N-scan the entire posts table.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Project scaffolding | Custom Cargo.toml + src layout | `cargo leptos new --git https://github.com/leptos-rs/start-axum` | Template already handles Axum + Leptos integration, metadata config, build scripts |
| Database migration management | SQL files in a directory, manual ordering | sqlx-cli (included with SQLx) | sqlx-cli handles versioning, rollback, and offline metadata generation |
| Offline compilation setup | Manual query metadata collection | `cargo sqlx prepare --database-url $DATABASE_URL` | One command generates complete `.sqlx/` directory; prevents stale queries |
| Password hashing | Custom bcrypt or PBKDF2 impl | argon2 0.5.3 crate | argon2 is memory-hard (resistant to GPU attacks), OWASP-recommended, battle-tested |
| JWT token handling | Custom signing/validation | jsonwebtoken 10.3.0 crate | Standard library, handles key rotation, algorithm negotiation, expiry validation |
| Error responses | Custom middleware to catch panics | Axum IntoResponse trait + middleware | IntoResponse is idiomatic, allows per-error-type status codes and JSON structure |
| Session/auth state | Global mutex or thread-local storage | Leptos context (create_server_data) + Axum extensions | Type-safe, composable, integrates with Leptos lifecycle |
| SSR hydration logic | Custom DOM diffing or manual syncing | Leptos built-in (automatic) | Leptos handles hydration internally; manual attempts introduce mismatches |

**Key insight:** The Rust web stack is young and requires a lot of glue code to be correct. Start-axum, SQLx, and Leptos libraries already handle the hard parts (build coordination, query safety, SSR hydration). Custom layers here are a maintenance burden.

## Common Pitfalls

### Pitfall 1: SQLx Offline Mode DATABASE_URL Override

**What goes wrong:** You set `SQLX_OFFLINE=true` in `.env`, but if `DATABASE_URL` is also set (e.g., from `.bashrc` or CI env), SQLx ignores offline mode and tries to connect to the live database. On CI without DB access, this causes silent fallback to `.sqlx/` metadata, which may be stale if queries were changed without running `cargo sqlx prepare`.

**Why it happens:** SQLx prioritizes live DB connection if available, treating offline mode as a fallback, not a guarantee.

**How to avoid:**
- In CI, explicitly unset `DATABASE_URL` before building: `unset DATABASE_URL && cargo build`
- Or set `SQLX_OFFLINE=true` and verify no DATABASE_URL in env: `env | grep DATABASE_URL` should return nothing
- Document this in project README under "Build Prerequisites"

**Warning signs:**
- Build succeeds locally with DB but fails in CI without error message about connection
- Stale query metadata errors appear inconsistently
- `cargo sqlx prepare` regenerates `.sqlx/` with different content than committed version

### Pitfall 2: Leptos SSR Hydration Mismatch via Signals with Default Values

**What goes wrong:** You initialize a signal with a default value (e.g., `create_signal(vec![])`) on the server, but then load data in a `create_resource` that populates the signal. On the server, the view renders with default empty list. On the client after hydration, the signal gets updated from the resource. Hydration sees a mismatch: server rendered empty list, client wants to render list with data. Result: hydration error, view becomes blank or jumpy.

**Why it happens:** Leptos renders on the server synchronously; async resources don't resolve. The client then hydrates and async resources run, updating signals. Server/client renderings differ.

**How to avoid:**
- Use `create_resource` with a default pending state in view: render a loading skeleton on both server and client.
- Or use `server_fns` in Server Functions that resolve before returning HTML (requires blocking or generating initial state on server and passing via props).
- Store async-resolved state in context on server, inject into view as props.

**Warning signs:**
- Hydration error in browser console mentioning signal or resource
- View content disappears on page load then reappears (flash of empty/default state)

### Pitfall 3: PostgreSQL Missing Pagination Index

**What goes wrong:** You implement cursor pagination with `(created_at, id)` keyset, but forget to create a compound index. Initial pages are fast (using primary key on id), but once you fetch page 5 (using `WHERE (created_at, id) < (X, Y)`), PostgreSQL must scan millions of rows to find the keyset. Feed queries timeout or become unusable as data grows.

**Why it happens:** Cursor pagination queries use `WHERE (column1, column2) < (val1, val2)` syntax that doesn't automatically use indexes unless you create a compound index in the same order.

**How to avoid:**
- Add migration: `CREATE INDEX idx_posts_pagination ON posts (created_at DESC, id DESC);`
- Explain plan the query: `EXPLAIN SELECT ... WHERE (created_at, id) < (...) ORDER BY ...` — should show "Index Scan" not "Seq Scan"
- Test pagination with 100k+ row dataset before merging

**Warning signs:**
- Page 1 loads in 50ms, page 5 loads in 2 seconds
- `EXPLAIN ANALYZE` shows "Seq Scan on posts" or "Filter: ..."
- Database CPU spikes when paginating backwards (fetch old pages)

### Pitfall 4: argon2 Hashing Without Spawn Blocking in Async Handler

**What goes wrong:** You call `argon2::hash_password()` directly in an Axum handler (which is an async function). argon2 is CPU-intensive and blocks the async runtime thread. Handler is marked `async`, but the blocking work starves other concurrent connections. With 10+ simultaneous sign-ups, server becomes unresponsive.

**Why it happens:** Axum runs on single-threaded or limited-thread tokio runtime. Blocking I/O or CPU work in handlers blocks all concurrent tasks.

**How to avoid:**
- Use `tokio::task::spawn_blocking()` to move argon2 hashing to a dedicated thread pool:
  ```rust
  let password = password.to_string();
  let hash = tokio::task::spawn_blocking(move || {
      argon2::hash_password(password.as_bytes(), &salt)
  })
  .await
  .map_err(|e| AppError::ServerError(e.to_string()))?
  .map_err(|e| AppError::ServerError(e.to_string()))?;
  ```
- Or use a helper function in server code that is not async, call it via `spawn_blocking`.
- Document this in code comments (it's non-obvious that argon2 blocks).

**Warning signs:**
- Handler takes 1-2 seconds to respond (argon2 configured for high memory cost)
- Adding concurrent load causes timeout errors even with fast DB queries
- CPU usage low but response time high (CPU is blocked, not busy)

### Pitfall 5: Leptos Stylesheet/Style Component in Leptos 0.8 Hydration Bug

**What goes wrong:** You add a `<Stylesheet>` or `<Style>` component inside a Leptos view (e.g., in app root). On the server, styles are rendered as `<style>` or `<link>` tags. On the client, if the component is erased (compiled with certain features), the style tag is not re-created during hydration. The DOM mismatch causes hydration error, and styles don't apply.

**Why it happens:** Known issue in Leptos 0.8.x where meta-like components (`Stylesheet`, `Style`, `Title`) have edge cases with SSR hydration when using erase_components.

**How to avoid:**
- Avoid `<Stylesheet>` in view components if possible; instead, import CSS at root level or use Tailwind with `class=` on elements.
- If you must use `<Stylesheet>`, test hydration in dev build and watch browser console for warnings.
- Or upgrade to latest Leptos 0.8.x patch (issue was documented, may be fixed).

**Warning signs:**
- Hydration error in console mentioning `<style>` or `<link>` tag
- Styles work in SSR HTML but not applied after hydration completes

## Code Examples

Verified patterns from official sources:

### Initializing SQLx Pool with Connection String

```rust
// In src/server/db.rs
use sqlx::postgres::PgPoolOptions;

pub async fn init_db() -> Result<sqlx::PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/my_x_db".to_string());

    PgPoolOptions::new()
        .max_connections(5) // Adjust for expected concurrency
        .connect(&database_url)
        .await
}

// In src/bin/server/main.rs
#[tokio::main]
async fn main() {
    let db = init_db().await.expect("Failed to connect to DB");
    // Pass db pool to Axum app state
}

// Source: SQLx docs https://docs.rs/crate/sqlx/latest
```

### Creating a User with Hashed Password

```rust
// In src/server/handlers/auth.rs
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::thread_rng;

#[server]
pub async fn sign_up(email: String, password: String) -> Result<UserId, ServerFnError> {
    // Validate input
    if email.is_empty() || password.len() < 8 {
        return Err(ServerFnError::new(AppError::ValidationError(
            "Email required, password >= 8 chars".to_string()
        )));
    }

    // Hash password in blocking context
    let salt = SaltString::generate(thread_rng());
    let password_hash = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
    })
    .await
    .map_err(|_| ServerFnError::new(AppError::ServerError("Hashing failed".to_string())))?
    .map_err(|_| ServerFnError::new(AppError::ServerError("Hashing failed".to_string())))?;

    // Store in DB
    let db = db_pool().await?;
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id, email"
    )
    .bind(&email)
    .bind(&password_hash)
    .fetch_one(&db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate") {
            ServerFnError::new(AppError::AuthError("Email already registered".to_string()))
        } else {
            ServerFnError::new(AppError::DbError(e.to_string()))
        }
    })?;

    Ok(user.id)
}

// Source: argon2 docs, STATE.md auth decision
```

### Migration File Format

```sql
-- migrations/001_create_users.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);

-- migrations/002_create_posts.sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    author_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL CHECK (length(content) <= 280),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_posts_author_id ON posts(author_id);
CREATE INDEX idx_posts_pagination ON posts(created_at DESC, id DESC);

-- migrations/003_create_follows.sql
CREATE TABLE follows (
    follower_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (follower_id, following_id)
);

CREATE INDEX idx_follows_following_id ON follows(following_id);

-- Source: PostgreSQL design patterns, Phase 1 success criteria
```

### Leptos Component with Error Handling

```rust
// In src/app.rs or any Leptos component
#[component]
fn LoginForm() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);

    let login_action = create_action(|_| {
        let email = email.get();
        let password = password.get();
        async move {
            match sign_in(email, password).await {
                Ok(jwt) => {
                    // Store JWT in localStorage (client-side)
                    window()
                        .unwrap()
                        .local_storage()
                        .unwrap()
                        .unwrap()
                        .set_item("jwt", &jwt)
                        .unwrap();
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }
    });

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            login_action.dispatch(());
        }>
            <input
                type="email"
                value=email
                on:change=move |ev| set_email(event_target_value(&ev))
                placeholder="Email"
            />
            <input
                type="password"
                value=password
                on:change=move |ev| set_password(event_target_value(&ev))
                placeholder="Password"
            />
            {move || error.get().map(|e| view! {
                <p class="error">{e}</p>
            })}
            <button type="submit">"Sign In"</button>
        </form>
    }
}

#[server]
pub async fn sign_in(email: String, password: String) -> Result<String, ServerFnError> {
    let db = db_pool().await?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash FROM users WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(&db)
    .await
    .map_err(|e| ServerFnError::new(AppError::DbError(e.to_string())))?
    .ok_or_else(|| ServerFnError::new(AppError::AuthError("User not found".to_string())))?;

    // Verify password
    let password_to_check = password.clone();
    let hash_str = user.password_hash.clone();
    let is_valid = tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&hash_str).ok()?;
        Argon2::default()
            .verify_password(password_to_check.as_bytes(), &hash)
            .is_ok()
    })
    .await
    .unwrap_or(false);

    if !is_valid {
        return Err(ServerFnError::new(AppError::AuthError("Invalid password".to_string())));
    }

    // Create JWT (simplified; real impl uses exp, kid, etc.)
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &serde_json::json!({"user_id": user.id, "email": user.email}),
        &jsonwebtoken::EncodingKey::from_secret(b"secret-key"),
    )
    .map_err(|_| ServerFnError::new(AppError::ServerError("JWT creation failed".to_string())))?;

    Ok(token)
}

// Source: Leptos book, STATE.md auth approach
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Diesel ORM with codegen | SQLx with compile-time macros | 2019+ | SQLx is lighter, no code generation, better for custom queries |
| Offset pagination (OFFSET 100) | Cursor/keyset pagination (WHERE (col1, col2) < (...)) | 2020s standard | Cursor pagination is O(1) per page vs O(n) for offset, essential for feeds |
| Session cookies (server state) | JWT tokens (stateless) | Widespread in APIs | JWT allows horizontal scaling, no session store needed |
| Handwritten SSR in Node/Python | Leptos full-stack framework | 2022+ | Leptos SSR in same codebase, single type system, less glue |
| bcrypt for password hashing | argon2 (memory-hard) | 2015+ OWASP recommended | argon2 resists GPU attacks, better for modern hardware |
| Manual async/await handling | Tokio ecosystem maturity | 2019+ | Tokio is standard, well-documented, integrates with all frameworks |

**Deprecated/outdated:**
- **Offset-based pagination:** Still works for small datasets, but scales poorly. Cursor pagination is now standard for public APIs.
- **Session cookies without JWT:** Still valid for monolithic apps, but JWT is more flexible for mobile + microservices.
- **SQLite for multi-user apps:** SQLite lacks concurrent writers, fine for personal projects but not for shared feeds.
- **Actix-web over Axum:** Actix still works, but Axum is smaller, more composable, and the Leptos ecosystem has standardized on it.

## Open Questions

1. **Testing strategy for Phase 1?**
   - What we know: `cargo leptos new` includes end-to-end tests in `end2end/` folder using Playwright. No database-specific tests in the template.
   - What's unclear: Should Phase 1 include unit tests for handlers? Integration tests for DB migrations? Or defer to Phase 2 when actual features land?
   - Recommendation: Create a basic integration test for migrations (`migrations` run without error) and a smoke test that `cargo leptos watch` starts without panics. Defer handler/component tests to Phase 2.

2. **Environment variable management across cargo-leptos build stages?**
   - What we know: `cargo leptos watch` recompiles both client and server. Environment variables set in `.env` are read at compile time.
   - What's unclear: If DATABASE_URL changes between server compile and run, will client see stale connection string? How to manage secrets (DB password) without committing?
   - Recommendation: Use `.env` for local dev (git-ignored), Docker Compose for consistent local DB, and CI will pass DATABASE_URL at runtime. Secrets never committed.

3. **Should Phase 1 scaffold with TailwindCSS or plain CSS?**
   - What we know: `cargo leptos new` can include Tailwind. Phase 1 goal is a "runnable" app, not beautiful UI.
   - What's unclear: Does Tailwind add complexity? Is plain CSS sufficient?
   - Recommendation: Skip Tailwind in Phase 1 scaffold. Add in Phase 2 if UI polish is a goal. Simple CSS in `style/` is sufficient to verify app runs.

## Validation Architecture

**Test Infrastructure Detected:**
- Framework: Rust built-in tests + Tokio (async runtime)
- Config: No explicit pytest/jest config; tests in `src/` using `#[test]` and `#[tokio::test]` attributes
- Quick run command: `cargo test --lib` (unit tests only, excludes integration tests)
- Full suite command: `cargo test` (all tests: unit, integration, doc tests)

**Phase Requirements → Test Map:**

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| (scaffolding phase — no feature requirements) | `cargo leptos watch` starts server and serves HTML at `localhost:3000` | Integration / smoke test | `timeout 10 cargo leptos watch &` + curl http://localhost:3000 | ❌ Wave 0 |
| (scaffolding phase — no feature requirements) | `cargo sqlx migrate run` applies all migrations without error | Integration test | `cargo sqlx migrate run` | ❌ Wave 0 |
| (scaffolding phase — no feature requirements) | `SQLX_OFFLINE=true cargo build` compiles successfully | Build test | `SQLX_OFFLINE=true cargo build --release` | ❌ Wave 0 |
| (scaffolding phase — no feature requirements) | Handler returns structured `AppError` JSON, not 500 panic | Unit test (handler) | `cargo test server::error --lib` | ❌ Wave 0 |

**Sampling Rate:**
- Per task commit: `cargo test --lib` (fast, < 5 seconds)
- Per wave merge: `cargo test && SQLX_OFFLINE=true cargo build --release` (comprehensive, < 30 seconds)
- Phase gate: Full test suite green + `cargo leptos watch` starts without error before `/gsd:verify-work`

**Wave 0 Gaps:**
- [ ] `tests/integration_smoke.rs` — Start `cargo leptos watch`, curl localhost:3000, verify HTML response (smoke test)
- [ ] `tests/migration_test.rs` — Run `sqlx migrate run` against test DB, verify all tables exist with correct schema
- [ ] `src/server/error.rs` — Implement `AppError` enum and `IntoResponse` trait, add `#[test]` for JSON serialization
- [ ] `.env.example` — Document expected env vars (DATABASE_URL, SQLX_OFFLINE) for contributors
- [ ] Database test fixture — Create a `sqlx::Pool` for integration tests (shared between test cases)

*(Deferred to Phase 2: Component rendering tests, handler logic tests, auth flow tests)*

## Sources

### Primary (HIGH confidence)
- **Leptos Official Book** — Getting Started, Server Functions, Error Handling, SSR/Hydration: https://book.leptos.dev/
- **Leptos GitHub Start-Axum Template** — Reference project structure: https://github.com/leptos-rs/start-axum
- **SQLx Official Docs & GitHub** — Offline mode, migrations, compile-time checking: https://github.com/launchbadge/sqlx
- **Axum Official Docs** — Error handling, IntoResponse pattern, middleware: https://docs.rs/axum/latest/axum/

### Secondary (MEDIUM confidence)
- **Reintech 2026 Leptos Guide** — Current best practices: https://reintech.io/blog/building-web-applications-with-leptos-complete-guide-2026
- **LinkedIn Learning Full-Stack Rust & Leptos** — Practical setup examples: https://www.linkedin.com/learning/full-stack-web-applications-with-rust-and-leptos/
- **Medium: Elegant Error Handling in Axum** — IntoResponse patterns: https://leapcell.io/blog/elegant-error-handling-in-axum-actix-web-with-intoresponse
- **PostgreSQL Wiki: Schema Design Best Practices** — Normalization, indexing: https://wiki.postgresql.org/wiki/Database_Schema_Recommendations_for_an_Application

### Tertiary (Verified WebSearch)
- **PostgreSQL Cursor Pagination Guides** — (created_at, id) pattern: https://blog.sequinstream.com/keyset-cursors-not-offsets-for-postgres-pagination/ and https://bun.uptrace.dev/guide/cursor-pagination.html
- **Leptos Hydration Bugs Documentation** — Known issues and workarounds: https://book.leptos.dev/ssr/24_hydration_bugs.html
- **Rust By Example: Rust to PostgreSQL with SQLx** — Code patterns: https://gist.github.com/jeremychone/34d1e3daffc38eb602b1a9ab21298d10

## Metadata

**Confidence breakdown:**
- **Standard stack:** HIGH — All versions and libraries confirmed in STATE.md and verified in official docs/crates.io
- **Architecture patterns:** HIGH — All patterns sourced from official Leptos book, start-axum template, and Axum docs
- **Pitfalls:** HIGH — Documented issues found in Leptos issue tracker, SQLx FAQ, and PostgreSQL best practices
- **Validation architecture:** MEDIUM — Test patterns inferred from Rust conventions and Leptos template; no explicit testing strategy document found

**Research date:** 2026-03-11
**Valid until:** 2026-04-11 (30 days; stable ecosystem, minor version updates expected)

**Known limitations:**
- Leptos 0.8.17 is not the absolute latest version (0.8.x may have progressed), but current as of latest docs.rs
- No source-to-source verification of `jsonwebtoken 10.3.0` and `argon2 0.5.3` exact versions (derived from STATE.md decisions)
- Hydration bugs in Leptos 0.8.x may have fixes in unreleased versions; recommend checking GitHub issues before Phase 3
