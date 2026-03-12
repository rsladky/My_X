# Phase 2: Auth - Research

**Researched:** 2026-03-12
**Domain:** JWT authentication, Argon2 password hashing, Leptos server functions, auth context signal, localStorage persistence
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **JWT library**: `jsonwebtoken` 10.3.0 (already in Cargo.toml under `[ssr]`)
- **Password hashing**: `argon2` 0.5.3, called via `tokio::task::spawn_blocking` to avoid blocking the async runtime
- **Frontend ↔ backend**: Leptos server functions only — no separate REST client, no manual fetch calls
- **JWT storage**: `localStorage` — persists across browser refresh
- **Credential errors**: Same generic error message for wrong password AND non-existent account (prevents account enumeration)
- Separate routes: `/login` and `/register`
- Registration has 3 fields: email, password, confirm password — confirm password checked client-side only; server receives email + password
- Server-side errors: single message displayed above the submit button
- Client-side validation: confirm password match only (all other validation is server-side)
- Post-auth navigation: Register → auto-login → redirect `/`; Login → redirect `/`; Logout → redirect `/login`
- Auth state signal: `RwSignal<Option<AuthUser>>` provided at App root via context; `AuthUser` carries user id (UUID) + username
- On page load: read JWT from `localStorage` → call server function to validate → populate signal
- Reuse `AppError::AuthError(String)` for 401 and `AppError::ValidationError(String)` for bad input

### Claude's Discretion
- Styling/layout — keep it clean and functional
- Username field: derive from email local-part (keep form simple at 3 fields) or collect explicitly — default: derive
- JWT TTL: suggest 7 days, planner to confirm
- Email format validation: `@` presence check server-side is sufficient
- Password minimum length: 8 characters, checked server-side

### Deferred Ideas (OUT OF SCOPE)
- Email verification / confirmation flow
- Password reset / forgot-password
- OAuth / social login
- Rate limiting on auth endpoints
- Session revocation / JWT blacklist
- Remember-me / "stay logged in" toggle
- Admin roles or permission scopes
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AUTH-01 | User can create an account with email and password | Server function `register` with argon2 hash → INSERT users; migration adds `username` column |
| AUTH-02 | User can log in with email and password and receive a JWT | Server function `login` with argon2 verify → jsonwebtoken encode → return token string |
| AUTH-03 | User session persists across browser refresh (JWT stored client-side) | localStorage write via `Effect::new` after login success; on-load `Resource` reads localStorage + calls `validate_token` server fn → populates `RwSignal<Option<AuthUser>>` |
| AUTH-04 | User can log out and have their session invalidated | Client-side logout: delete JWT from localStorage, write `None` to auth signal, `use_navigate` to `/login` |
</phase_requirements>

---

## Summary

Phase 2 implements a full auth flow on a Leptos 0.8 + Axum + SQLx stack where all auth operations run as Leptos server functions. The server handles password hashing with argon2, JWT signing with jsonwebtoken, and database queries via SQLx. The client handles localStorage persistence and auth state via a root-level `RwSignal<Option<AuthUser>>` provided through Leptos context.

**Critical schema gap discovered:** The `users` table created in Phase 1 has `id SERIAL PRIMARY KEY` (i32, not UUID) and no `username` column. The CONTEXT.md requires `AuthUser { id: Uuid, username: String }` and JWT claims with `sub` as UUID string. Phase 2 must add a migration that adds a `username` column to the existing `users` table. The planner must decide whether to keep `SERIAL` ids (store as string in JWT) or add a `uuid` column as an alternative identifier. Research recommendation: add `username TEXT NOT NULL` column via migration and keep `SERIAL` id (stored as string in JWT `sub`), since posts/follows already reference `user_id INTEGER` — changing to UUID would require rewriting all FK columns.

The auth flow has three interacting layers: (1) server functions behind `#[server]` that hold all business logic and DB access; (2) Leptos components with `ServerAction` and `ActionForm` that submit forms without manual fetch; (3) client-side effects (`Effect::new`) that write/read localStorage and trigger navigation.

**Primary recommendation:** Use `ServerAction::new()` + `ActionForm` for form submission. Wire PgPool via `leptos_routes_with_context` providing the pool as context, then use `use_context::<PgPool>()` inside server functions. localStorage access goes inside `Effect::new` only (never in component body to avoid SSR hydration panics). Navigate after auth via `use_navigate` inside an Effect watching action value.

---

## Standard Stack

### Core (all already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `jsonwebtoken` | 10.3.0 | JWT sign/verify with HS256 | Locked decision; de facto standard JWT crate in Rust ecosystem |
| `argon2` | 0.5.3 | Password hashing with Argon2id | Locked decision; OWASP recommended, memory-hard |
| `sqlx` | 0.8.6 | Async SQL queries against PostgreSQL | Locked from Phase 1; type-safe compile-time queries |
| `tokio` | 1.x | `spawn_blocking` for CPU-heavy hash operations | Already in project; required to keep argon2 off async executor |
| `uuid` | 1 (optional/ssr) | UUIDs if needed | Already feature-gated under ssr |
| `serde` | 1 | JWT Claims struct serialization | Already in project |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `web_sys` | via wasm-bindgen | `localStorage.set_item` / `get_item` / `remove_item` in WASM | Client-side JWT storage; feature-gate `Storage,Window` in Cargo.toml |
| `leptos_router::hooks::use_navigate` | 0.8.x | Programmatic navigation after auth | After login/logout success in client-side Effect |

### Dependency Addition Required
The `web_sys` crate is already pulled in transitively via `wasm-bindgen`. However, specific localStorage features must be explicitly enabled:

```toml
# Add to [dependencies] under hydrate feature or unconditionally (web_sys is already wasm-only)
web-sys = { version = "0.3", features = ["Storage", "Window"] }
```

Check if `web-sys` is already present in Cargo.toml before adding — if wasm-bindgen already pulls it, only add the features line.

**Installation:** No new crates needed for server-side auth. For localStorage:
```bash
# Verify web-sys is available and add features to Cargo.toml if needed
# All server-side auth crates (jsonwebtoken, argon2, uuid, sqlx) are already present
```

---

## Architecture Patterns

### Recommended File Structure for Phase 2
```
src/
├── app.rs                         # Add /login and /register routes; provide auth context
├── server/
│   ├── mod.rs                     # Add: pub mod auth;
│   ├── error.rs                   # Unchanged (AppError already has AuthError, ValidationError)
│   └── auth/
│       ├── mod.rs                 # pub mod handlers; pub use handlers::*;
│       └── handlers.rs            # register(), login(), validate_token() server functions
├── components/
│   ├── mod.rs                     # New: pub mod login_page; pub mod register_page;
│   ├── login_page.rs              # LoginPage component
│   └── register_page.rs          # RegisterPage component
└── lib.rs                         # Unchanged
```

Also: `migrations/YYYYMMDDHHMMSS_add_username_to_users.sql` — adds `username` column.

### Pattern 1: PgPool Wired via leptos_routes_with_context

**What:** Provide PgPool as Leptos context at route registration time, not via Axum `State`. Server functions access pool with `use_context::<PgPool>()`.

**When to use:** Whenever a server function needs database access in Leptos 0.8.

**Example:**
```rust
// src/main.rs (ssr feature gate)
// Source: https://book.leptos.dev/server/26_extractors.html

let pool = PgPool::connect(&database_url).await.expect("db connect");

let app = Router::new()
    .leptos_routes_with_context(
        &leptos_options,
        routes,
        {
            let pool = pool.clone();
            move || provide_context(pool.clone())
        },
        move || shell(leptos_options.clone()),
    )
    .fallback(leptos_axum::file_and_error_handler(shell))
    .with_state(leptos_options);
```

```rust
// Inside any #[server] function:
#[cfg(feature = "ssr")]
pub async fn register(email: String, password: String) -> Result<String, ServerFnError> {
    let pool = use_context::<PgPool>()
        .ok_or_else(|| ServerFnError::new("No pool in context"))?;
    // ...
}
```

### Pattern 2: Auth Server Functions

**What:** Three server functions handle all auth logic: `register`, `login`, `validate_token`.

**When to use:** Called from ActionForm or client-side effects.

```rust
// Source: jsonwebtoken docs.rs + argon2 docs.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // user id as string
    pub username: String,
    pub exp: usize,    // Unix timestamp (seconds)
}

#[server]
pub async fn login(email: String, password: String) -> Result<String, ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::time::{SystemTime, UNIX_EPOCH};

    let pool = use_context::<sqlx::PgPool>().ok_or_else(|| ServerFnError::new("no pool"))?;

    // Look up user — same error for wrong password and missing account (no enumeration)
    let user = sqlx::query!(
        "SELECT id, username, password_hash FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let generic_error = ServerFnError::new("Invalid email or password");

    let user = user.ok_or_else(|| generic_error.clone())?;

    // Argon2 verify must run on a blocking thread
    let hash = user.password_hash.clone();
    let pwd = password.clone();
    let matches = tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash).map_err(|_| ())?;
        Argon2::default().verify_password(pwd.as_bytes(), &parsed).map_err(|_| ())
    })
    .await
    .map_err(|_| generic_error.clone())?;

    matches.map_err(|_| generic_error.clone())?;

    // Issue JWT — 7 day expiry
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 60 * 60 * 24 * 7;

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username,
        exp,
    };

    let secret = std::env::var("JWT_SECRET").map_err(|_| ServerFnError::new("missing JWT_SECRET"))?;
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(token)
}
```

### Pattern 3: Argon2 Hashing in register

**What:** Hash password on blocking thread before INSERT.

```rust
// Source: https://docs.rs/argon2/latest/argon2/

#[server]
pub async fn register(email: String, password: String) -> Result<String, ServerFnError> {
    use argon2::{password_hash::{rand_core::OsRng, SaltString, PasswordHasher}, Argon2};

    // Derive username from email local-part
    let username = email.split('@').next().unwrap_or(&email).to_string();

    // Hash on blocking thread
    let pwd = password.clone();
    let hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(pwd.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .map_err(|e| ServerFnError::new(e))?;

    // INSERT — returns token immediately for auto-login
    // ... sqlx INSERT then call login logic or return token
    Ok(token)
}
```

### Pattern 4: Auth Context Signal at App Root

**What:** `RwSignal<Option<AuthUser>>` provided once at App root, read by all children.

```rust
// Source: https://book.leptos.dev/15_global_state.html

#[derive(Clone, Debug, PartialEq)]
pub struct AuthUser {
    pub id: i32,       // matches SERIAL in users table
    pub username: String,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let auth_user: RwSignal<Option<AuthUser>> = RwSignal::new(None);
    provide_context(auth_user);

    // On first client render, read JWT from localStorage and validate
    // (see Pattern 6 for the Effect that populates this)

    view! {
        <Stylesheet id="leptos" href="/pkg/my_x.css"/>
        <Title text="My X"/>
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("register") view=RegisterPage/>
                </Routes>
            </main>
        </Router>
    }
}
```

### Pattern 5: ActionForm for Login/Register Forms

**What:** Leptos `ActionForm` with `ServerAction` — no manual fetch, progressive enhancement built in.

```rust
// Source: https://book.leptos.dev/progressive_enhancement/action_form.html

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<Login>::new();
    let value = login_action.value();

    // Effect: on success, write JWT to localStorage and navigate
    let navigate = leptos_router::hooks::use_navigate();
    let auth_user = use_context::<RwSignal<Option<AuthUser>>>()
        .expect("AuthUser context");

    Effect::new(move |_| {
        if let Some(Ok(token)) = value.get() {
            // Write to localStorage (client-only, safe inside Effect)
            #[cfg(not(feature = "ssr"))]
            {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = storage.set_item("jwt", &token);
                }
            }
            // TODO: decode claims client-side or call validate_token to get AuthUser
            navigate("/", Default::default());
        }
    });

    view! {
        <ActionForm action=login_action>
            // Display error above submit button
            {move || value.get().and_then(|v| v.err()).map(|e| view! {
                <p class="error">{e.to_string()}</p>
            })}
            <input type="email" name="email" placeholder="Email" required/>
            <input type="password" name="password" placeholder="Password" required/>
            <button type="submit">"Log in"</button>
        </ActionForm>
        <a href="/register">"Don't have an account? Register"</a>
    }
}
```

### Pattern 6: Page-Load JWT Validation

**What:** On first render, read JWT from localStorage, call `validate_token` server function, populate auth signal.

**Key constraint:** localStorage reads MUST be in `Effect::new` or guarded by `#[cfg(not(feature = "ssr"))]` to prevent SSR panics.

```rust
// In App() component, after providing auth_user context:
Effect::new(move |_| {
    #[cfg(not(feature = "ssr"))]
    {
        let token = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("jwt").ok().flatten());

        if let Some(token) = token {
            spawn_local(async move {
                if let Ok(user) = validate_token(token).await {
                    auth_user.set(Some(user));
                }
            });
        }
    }
});
```

### Pattern 7: Logout (Client-Side Only)

**What:** Logout is purely client-side — remove localStorage item, clear signal, navigate.

```rust
// In any component that has a logout button:
let auth_user = use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");
let navigate = leptos_router::hooks::use_navigate();

let on_logout = move |_| {
    #[cfg(not(feature = "ssr"))]
    {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let _ = storage.remove_item("jwt");
        }
    }
    auth_user.set(None);
    navigate("/login", Default::default());
};
```

### Anti-Patterns to Avoid
- **Accessing `window()` or `localStorage` in component body:** Panics on SSR. Always wrap in `Effect::new` or `#[cfg(not(feature = "ssr"))]`.
- **Blocking argon2 on async executor:** `Argon2::hash_password` is CPU-bound and will starve other tasks. Always use `tokio::task::spawn_blocking`.
- **Different error messages for wrong password vs. non-existent user:** This enables account enumeration. Return identical `ServerFnError` for both cases.
- **Providing PgPool via Axum `State` and trying to extract it in Leptos server functions without `extract_with_state`:** Use `leptos_routes_with_context` + `use_context::<PgPool>()` instead — cleaner integration.
- **Using `create_effect` (Leptos < 0.7 API):** In Leptos 0.8, use `Effect::new(move |_| { ... })`.
- **Storing JWT decode result in component body signal initialized from localStorage:** This creates an SSR/client mismatch. The signal must start as `None` server-side; only Effects populate it client-side.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Password hashing | Custom bcrypt or plain SHA-256 | `argon2` crate | Argon2id is memory-hard; resists GPU brute force; PHC string format handles salt encoding |
| JWT encode/decode | Manual base64 + HMAC | `jsonwebtoken` crate | Handles header, claims, signature, expiry validation correctly |
| UUID generation | Custom random string | `uuid` crate with `v4` feature | UUID v4 uses OS entropy correctly, already gated under `ssr` for WASM safety |
| Form state management | Manual `on:input` + `create_signal` per field | `ActionForm` + `ServerAction` | ActionForm handles serialization, pending state, error propagation automatically |
| Email validation | Complex regex | `str.contains('@')` check | CONTEXT.md explicitly specifies `@` presence is sufficient for v1 |

**Key insight:** The argon2 and jsonwebtoken crates handle all the subtle edge cases (salt uniqueness, timing-safe comparison, claim expiry) that a custom implementation would get wrong.

---

## Common Pitfalls

### Pitfall 1: localStorage Access in SSR Context
**What goes wrong:** Calling `web_sys::window()` or any browser API during server-side rendering causes a panic with "cannot call wasm-bindgen imported functions on non-wasm targets."
**Why it happens:** Leptos runs component functions on the server to generate HTML. Browser globals don't exist there.
**How to avoid:** Gate ALL localStorage access inside `Effect::new(move |_| { ... })` or `#[cfg(not(feature = "ssr"))]`. Effects are guaranteed to run only on the client.
**Warning signs:** Panic at startup mentioning `wasm_bindgen` or `window`.

### Pitfall 2: schema mismatch — users table has SERIAL id, no username column
**What goes wrong:** CONTEXT.md specifies `AuthUser { id: Uuid, username: String }` but the actual `users` table has `id SERIAL PRIMARY KEY` and no `username` column.
**Why it happens:** Phase 1 migration created a minimal schema; username was not included.
**How to avoid:** Phase 2 MUST add a migration: `ALTER TABLE users ADD COLUMN username TEXT NOT NULL DEFAULT ''`. After migration, set a NOT NULL constraint. The `id` should stay as `INTEGER` (not UUID) to avoid rewriting FK columns in `posts` and `follows`. The `AuthUser.id` field should be `i32`, not `Uuid`, to match the actual schema. JWT `sub` stores the id as a string (`user.id.to_string()`).
**Warning signs:** `sqlx::query!` compile error about missing column, or type mismatch on `id` field.

### Pitfall 3: argon2 Blocking the Async Executor
**What goes wrong:** Argon2 hash/verify takes 50-300ms on purpose (memory-hard). Running it directly in `async fn` blocks the Tokio thread, preventing other requests from being served.
**Why it happens:** Argon2 is CPU/memory bound — not I/O awaitable.
**How to avoid:** Wrap in `tokio::task::spawn_blocking(move || { ... }).await`.
**Warning signs:** Server becomes unresponsive during login under any load.

### Pitfall 4: PgPool Not Available in Server Functions
**What goes wrong:** `use_context::<PgPool>()` returns `None` inside a server function, causing a runtime error.
**Why it happens:** PgPool was added to Axum's `.with_state()` but not provided as Leptos context. Leptos server functions run in a context separate from Axum's state unless explicitly bridged.
**How to avoid:** Use `leptos_routes_with_context` (not `leptos_routes`) and call `provide_context(pool.clone())` in the context closure. The current `main.rs` uses `.leptos_routes()` — this must be changed.
**Warning signs:** `None` from `use_context::<PgPool>()` at runtime, or server function panics on pool access.

### Pitfall 5: ServerFnError API in Leptos 0.8
**What goes wrong:** Using `ServerFnError::ServerError(String)` (Leptos 0.7 API) causes a compile error in Leptos 0.8.
**Why it happens:** In Leptos 0.8, `ServerFnError` was refactored. Custom constructors changed.
**How to avoid:** Use `ServerFnError::new("message")` in Leptos 0.8. Alternatively, since Leptos 0.8 supports returning custom error types from server functions, a `Result<T, String>` is the simplest option for auth errors.
**Warning signs:** Compile error mentioning `ServerFnError` variant not found.

### Pitfall 6: use_navigate Must Be Called Inside a Router
**What goes wrong:** Calling `use_navigate()` outside a `<Router>` component panics with "You cannot call `use_navigate` outside a `<Router>`".
**Why it happens:** `use_navigate` reads router context; if called at module level or outside the component tree it fails.
**How to avoid:** Call `use_navigate()` inside the component function body, not in server function body or outside any component.
**Warning signs:** Panic mentioning `use_navigate` outside Router.

### Pitfall 7: SQLx Offline Mode and New Queries
**What goes wrong:** Adding new `sqlx::query!()` macros without running `cargo sqlx prepare` causes compile failure with `SQLX_OFFLINE=true` because the `.sqlx/` cache doesn't contain the new query metadata.
**Why it happens:** SQLx offline mode verifies queries against a cached database schema. New queries must be registered.
**How to avoid:** After writing auth queries, run `cargo sqlx prepare` with a live database before committing. This updates `.sqlx/` files.
**Warning signs:** Compile error like "failed to find data for query" or "no query in offline store".

---

## Code Examples

### Verified: JWT Encode (jsonwebtoken 10.3)
```rust
// Source: https://docs.rs/jsonwebtoken/latest/jsonwebtoken/fn.encode.html
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    exp: usize,
}

let claims = Claims {
    sub: "42".to_string(),
    username: "alice".to_string(),
    exp: 9999999999, // Unix timestamp
};

let token = encode(
    &Header::default(), // HS256
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
)?;
```

### Verified: JWT Decode (jsonwebtoken 10.3)
```rust
// Source: https://docs.rs/jsonwebtoken/latest/jsonwebtoken/fn.decode.html
use jsonwebtoken::{decode, DecodingKey, Validation};

let token_data = decode::<Claims>(
    &token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(), // validates exp automatically
)?;
let claims = token_data.claims;
```

### Verified: Argon2 Hash (argon2 0.5)
```rust
// Source: https://docs.rs/argon2/latest/argon2/
use argon2::{
    password_hash::{rand_core::OsRng, SaltString, PasswordHasher},
    Argon2,
};

let salt = SaltString::generate(&mut OsRng);
let hash = Argon2::default()
    .hash_password(password.as_bytes(), &salt)?
    .to_string();
```

### Verified: Argon2 Verify (argon2 0.5)
```rust
use argon2::{password_hash::{PasswordHash, PasswordVerifier}, Argon2};

let parsed_hash = PasswordHash::new(&stored_hash)?;
Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
// returns Ok(()) if valid, Err if not
```

### Verified: localStorage in Effect (Leptos 0.8)
```rust
// Source: https://book.leptos.dev/web_sys.html
// Must be inside Effect::new — never in component body
Effect::new(move |_| {
    #[cfg(not(feature = "ssr"))]
    {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let _ = storage.set_item("jwt", &token_string);
        }
    }
});
```

### Verified: provide_context + use_context Pattern (Leptos 0.8)
```rust
// Source: https://book.leptos.dev/15_global_state.html
// In App():
let auth_user: RwSignal<Option<AuthUser>> = RwSignal::new(None);
provide_context(auth_user);

// In any child component:
let auth_user = use_context::<RwSignal<Option<AuthUser>>>()
    .expect("AuthUser context must be provided");
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `create_server_action::<Fn>()` | `ServerAction::<Fn>::new()` | Leptos 0.7→0.8 | API change; old form causes compile error |
| `create_effect(move |_| {...})` | `Effect::new(move |_| {...})` | Leptos 0.7→0.8 | API change |
| `create_rw_signal(val)` | `RwSignal::new(val)` | Leptos 0.7→0.8 | API change |
| `ServerFnError::ServerError(msg)` | `ServerFnError::new(msg)` | Leptos 0.8 | Old variant removed |
| `leptos_axum::extract()` with State | `leptos_routes_with_context` + `use_context` | Leptos 0.6→0.7+ | Context approach is cleaner for DB pools |

**Deprecated/outdated:**
- `create_signal`, `create_rw_signal`, `create_effect`: All renamed in Leptos 0.8; use `signal()`, `RwSignal::new()`, `Effect::new()`.

---

## Schema Gap — Migration Needed

The Phase 1 `users` table is missing a `username` column required by `AuthUser`. Phase 2 Wave 1 must include a migration:

```sql
-- migrations/YYYYMMDDHHMMSS_add_username_to_users.sql
ALTER TABLE users ADD COLUMN username TEXT NOT NULL DEFAULT '';

-- After backfilling (no existing rows in dev):
ALTER TABLE users ALTER COLUMN username DROP DEFAULT;

CREATE UNIQUE INDEX idx_users_username ON users(username);
```

**ID type decision:** Keep `users.id` as `INTEGER` (SERIAL). The `AuthUser` struct should use `i32` (not `Uuid`) to match. JWT `sub` stores the integer id as a string. The `uuid` crate (already in Cargo.toml under ssr) is not needed for user IDs in this phase — it may be needed for other purposes in future phases.

---

## Open Questions

1. **`AuthUser.id` type — i32 vs. UUID**
   - What we know: `users.id` is `SERIAL` (i32). CONTEXT.md mentions UUID for user id in `AuthUser`.
   - What's unclear: Whether planner wants to add a separate `uuid` column or just use `i32` and update CONTEXT.md.
   - Recommendation: Use `i32` matching the actual schema. Document the discrepancy. No UUID column needed — CONTEXT.md will need a minor correction in PLAN.md.

2. **Username uniqueness on derive-from-email**
   - What we know: If two users have `alice@gmail.com` and `alice@yahoo.com`, both derive username `alice`, violating a UNIQUE constraint.
   - What's unclear: How to handle collisions — suffix with number? Require unique email-derived username?
   - Recommendation: On INSERT conflict for username, append a short random suffix (2 digits). Or omit the UNIQUE constraint on username for v1 since profiles are email-keyed anyway.

3. **`validate_token` server function return type**
   - What we know: Called on page load to convert stored JWT into `AuthUser`.
   - What's unclear: Should it return `Result<AuthUser, ServerFnError>` or `Result<Option<AuthUser>, ServerFnError>`?
   - Recommendation: `Result<AuthUser, ServerFnError>`. Client ignores `Err` (treats as not logged in) and only populates signal on `Ok`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[cfg(test)]` modules |
| Config file | none (uses `cargo test`) |
| Quick run command | `cargo test --features ssr` |
| Full suite command | `cargo test --features ssr` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-01 | `register` server fn inserts user and returns JWT | unit | `cargo test --features ssr auth::tests::register_creates_user` | Wave 0 |
| AUTH-01 | Password is hashed (stored hash != plaintext) | unit | `cargo test --features ssr auth::tests::password_is_hashed` | Wave 0 |
| AUTH-02 | `login` returns JWT for valid credentials | unit | `cargo test --features ssr auth::tests::login_valid_creds` | Wave 0 |
| AUTH-02 | `login` returns same error for wrong password and missing account | unit | `cargo test --features ssr auth::tests::login_same_error_on_failure` | Wave 0 |
| AUTH-03 | `validate_token` returns AuthUser for valid JWT | unit | `cargo test --features ssr auth::tests::validate_token_ok` | Wave 0 |
| AUTH-03 | `validate_token` errors on expired JWT | unit | `cargo test --features ssr auth::tests::validate_token_expired` | Wave 0 |
| AUTH-04 | Logout is client-side: no server-side test needed | manual | n/a — verify localStorage cleared + redirect in browser | manual only |

> Note: server function tests run without a live database by testing the logic units in isolation (argon2 hash/verify, JWT encode/decode). DB integration tests would require a test database — out of scope for this phase's validation gate.

### Sampling Rate
- **Per task commit:** `cargo test --features ssr`
- **Per wave merge:** `cargo test --features ssr`
- **Phase gate:** All `--features ssr` tests green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/server/auth/handlers.rs` — test module `#[cfg(test)] mod tests { ... }` covering AUTH-01 through AUTH-03 logic units
- [ ] No framework install needed — `cargo test` is built-in

---

## Sources

### Primary (HIGH confidence)
- [jsonwebtoken docs.rs](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/fn.encode.html) — encode/decode API, EncodingKey, DecodingKey, Claims
- [argon2 docs.rs](https://docs.rs/argon2/latest/argon2/) — Argon2, PasswordHasher, PasswordVerifier, SaltString
- [Leptos book — Server Functions](https://book.leptos.dev/server/25_server_functions.html) — `#[server]` macro, ServerFnError, ActionForm
- [Leptos book — Extractors](https://book.leptos.dev/server/26_extractors.html) — leptos_routes_with_context, provide_context for PgPool
- [Leptos book — Global State](https://book.leptos.dev/15_global_state.html) — RwSignal, provide_context, use_context
- [Leptos book — Hydration Bugs](https://book.leptos.dev/ssr/24_hydration_bugs.html) — localStorage must be in Effect
- [Leptos book — ActionForm](https://book.leptos.dev/progressive_enhancement/action_form.html) — ServerAction, ActionForm, value signal

### Secondary (MEDIUM confidence)
- [Leptos GitHub discussions #1363](https://github.com/leptos-rs/leptos/discussions/1363) — JWT + token renewal patterns in Leptos (community verified)
- [axum/examples/sqlx-postgres](https://github.com/tokio-rs/axum/blob/main/examples/sqlx-postgres/src/main.rs) — SQLx + Axum state pattern

### Tertiary (LOW confidence)
- Various community posts on argon2 + spawn_blocking — consistent with official docs pattern, not independently verified with a dated authoritative source

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates already in Cargo.toml, versions confirmed
- Architecture (server functions, context): HIGH — verified against official Leptos book
- JWT encode/decode patterns: HIGH — verified against docs.rs
- Argon2 patterns: HIGH — verified against docs.rs
- localStorage/Effect pattern: HIGH — verified against Leptos book hydration chapter
- Schema gap (username column): HIGH — confirmed by reading actual migration files
- Leptos 0.8 API names (Effect::new, ServerAction::new): MEDIUM — confirmed from release notes and book, cross-referenced with community sources

**Research date:** 2026-03-12
**Valid until:** 2026-04-12 (stable stack; Leptos 0.8.x patch releases unlikely to break these patterns)
