# Phase 3: Posts + Profiles - Research

**Researched:** 2026-03-12
**Domain:** Leptos 0.8 routing, server functions, reactive data loading, X-style layout
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**App Navigation**
- Left sidebar layout, X-style: logo top-left, Home link, Profile link (to own profile), a Post button, and Logout at bottom
- Sidebar is persistent on all authenticated pages
- Auth/login and register pages have no sidebar — standalone centered layout (consistent with Phase 2)
- All pages (/ and /{username}) require authentication — unauthenticated visitors redirect to /login
- A shared layout component wraps all authenticated pages (sidebar + main content area)

**Profile URLs**
- X-style: `/{username}` (e.g. `/robinsladky`) — no `/users/` prefix
- Routing must not conflict with `/login` and `/register` (static segments take priority)
- Navigation to another user's profile: clicking their @username on a post

**Profile Page**
- Full X-style header skeleton: banner area (grey box), avatar circle (grey placeholder), display name, @handle, bio area (placeholder text), post count
- Post count displayed under the handle (e.g. "42 posts")
- Same page component for own profile and others' profiles
- Delete button only appears on posts where `post.author_id == auth_user.id` — no UI difference otherwise

**Home Page (/)**
- Phase 3 home shows a compose box + the authenticated user's own posts in reverse-chronological order
- Phase 4 will replace this with the full social feed (following-based)
- Compose box at top of feed, always visible (X-style)
- Compose: text area + remaining character count (e.g. "234") + Post button
- Post button disabled when textarea is empty or character count exceeds 280
- After posting: compose clears, new post appears at top of feed immediately (reactive update — no page reload)

**Post Card Display**
- Each post card: grey circle avatar placeholder | @username (clickable link to /{username}) | relative timestamp (e.g. "2h", "3d") | post text
- Posts separated by thin border-bottom divider (no card shadow) — X-style high-density list
- Action buttons (reply, retweet, like): out of scope for Phase 3
- Delete: `...` overflow menu top-right of card, only rendered on own posts — clicking reveals "Delete" option

**Timestamp Format**
- Relative timestamps for display: seconds → "Xs", minutes → "Xm", hours → "Xh", days → "Xd"
- No hover/absolute fallback in Phase 3 (Claude's discretion to add title attribute for accessibility)

### Claude's Discretion
- Exact sidebar width and spacing
- Color palette details (should follow X's dark-on-light or dark theme — either is fine)
- Avatar placeholder exact shade
- Overflow menu open/close interaction mechanics (toggle on click)
- Error handling UX for failed post creation (single message pattern from Phase 2)

### Deferred Ideas (OUT OF SCOPE)
- Follow/unfollow button on profiles — Phase 4
- Follower/following counts on profile — Phase 4 (SOCL-03)
- Action buttons on post cards (reply, retweet, like) — out of scope v1
- Real avatar/photo upload — out of scope v1 (media uploads excluded)
- Edit profile (display name, bio) — v2 (PROF-V2-01)
- Hover tooltip showing absolute timestamp — nice-to-have, Claude can add as title attribute
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| POST-01 | Authenticated user can create a text post (max 280 characters) | Server function `create_post` with content validation; `ServerAction` + `Resource` with `action.version()` for reactive list update |
| POST-02 | Authenticated user can delete their own post | Server function `delete_post` verifying `author_id == auth_user.id` from JWT; `AppError::AuthError` for rejection |
| POST-03 | Post displays author username and creation timestamp | JOIN query `posts JOIN users ON author_id = users.id`; chrono `NaiveDateTime` on Rust side; relative timestamp computed client-side from `created_at` |
| PROF-01 | User can view their own profile with a list of their posts | `/{username}` dynamic route; `get_user_posts` server function returning posts for that username; profile skeleton component |
| PROF-02 | User can view another user's profile with their post list | Same component and server function as PROF-01 — param driven, no separate impl needed |
</phase_requirements>

---

## Summary

Phase 3 builds on a fully working Leptos 0.8 + Axum + SQLx foundation from Phase 2. The core technical challenges are: (1) introducing a shared authenticated layout using `ProtectedParentRoute` + `Outlet`, (2) implementing a `/{username}` dynamic route that doesn't conflict with `/login` and `/register`, (3) wiring reactive post creation so the list updates immediately using the `action.version()` → `Resource` dependency pattern, and (4) server-side authorization in `delete_post` matching `author_id` against the JWT-derived user identity.

All UI uses inline styles (established codebase pattern), `PgPool` is accessed via `use_context::<PgPool>()` inside server functions (established Phase 2 pattern), and auth identity flows from the JWT via the `AuthUser` context signal. The `posts` table already exists with the correct schema (`id`, `author_id`, `content`, `created_at`, `updated_at`) and has the right indexes for the queries needed.

The known risk logged in STATE.md — Leptos SSR + hydration being least-documented — is mitigated by Phase 2 already shipping a working SSR+hydration app. Phase 3 adds routing complexity (nested/parent routes) but the pattern is well-documented. A focused SSR spike before full component buildout remains the right first step.

**Primary recommendation:** Use `ProtectedParentRoute` for the authenticated layout shell with `Outlet`, `path!("/:username")` for profile routing, and the `action.version()` → `Resource` dependency chain for reactive post list updates.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| leptos | 0.8.17 | Component framework, server functions, reactive signals | Already in use; all patterns established in Phase 2 |
| leptos_router | 0.8 | `ProtectedParentRoute`, `ParentRoute`, dynamic `path!` macro | Provides all routing constructs needed |
| leptos_axum | 0.8 | SSR integration, `LeptosRoutes`, `generate_route_list` | Already wired in main.rs |
| sqlx | 0.8.6 | Async PostgreSQL queries with compile-time checking | Already in use; `posts` table ready |
| chrono | 0.4 | `NaiveDateTime` for `created_at`, timestamp arithmetic | Already in Cargo.toml |
| axum | 0.8.8 | HTTP server, pool context injection | Already running |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| web-sys | 0.3 | `performance.now()` or `Date.now()` for client-side relative timestamp computation | Only needed if computing relative time in WASM; can compute server-side too |
| chrono | 0.4 | Server-side relative timestamp as `String` (alternative: compute in server fn, return formatted string) | Simplest approach: compute in server function, no web-sys needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `ProtectedParentRoute` + `Outlet` | Manual auth check in each component | `ProtectedParentRoute` is cleaner and DRY; manual checks would duplicate redirect logic everywhere |
| `action.version()` as Resource source | `resource.refetch()` in action effect | `action.version()` is the idiomatic Leptos pattern; `refetch()` exists but coupling via reactive dependency is cleaner |
| Server-side relative timestamp | Client-side `Date.now()` - `created_at` | Server-side is simpler (no web-sys, no hydration mismatch risk), slightly stale on long-held pages — acceptable for Phase 3 |

**Installation:** No new dependencies required. `chrono` is already in `Cargo.toml`. No additions needed.

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── app.rs                    # Add ProtectedParentRoute, /{username} route; replace HomePage
├── auth_user.rs              # Unchanged
├── lib.rs                    # Unchanged
├── main.rs                   # Unchanged
├── components/
│   ├── mod.rs                # Export all new components
│   ├── login_page.rs         # Unchanged
│   ├── register_page.rs      # Unchanged
│   ├── authenticated_layout.rs  # Sidebar + <Outlet/> shell
│   ├── sidebar.rs            # Left nav: logo, Home, Profile, Post button, Logout
│   ├── home_page.rs          # Compose box + own post feed
│   ├── profile_page.rs       # Profile header skeleton + post list (own or other)
│   ├── post_card.rs          # Single post card with overflow delete menu
│   └── compose_box.rs        # Textarea + char count + Post button
└── server/
    ├── mod.rs                # Add `pub mod posts;`
    ├── error.rs              # Unchanged
    ├── auth/                 # Unchanged
    └── posts/
        ├── mod.rs            # pub mod handlers; pub use handlers::*;
        └── handlers.rs       # create_post, delete_post, list_own_posts, get_user_posts server fns
```

### Pattern 1: Authenticated Layout with ProtectedParentRoute + Outlet

**What:** A parent route that checks `auth_user.get().is_some()`, redirects to `/login` if not, and renders sidebar + `<Outlet/>` for all authenticated child routes.

**When to use:** Any route that requires authentication. Wrap all of `/` and `/:username` inside it.

**Example:**
```rust
// Source: https://docs.rs/leptos_router/latest/leptos_router/components/fn.ProtectedParentRoute.html
// In app.rs Routes block:
<ProtectedParentRoute
    path=StaticSegment("")
    view=AuthenticatedLayout
    condition=move || {
        let user = auth_user.get();
        // None = still loading (avoid flash redirect), Some(true/false) = known state
        match user {
            None => None,        // JWT validation in-flight — hold off on redirect
            Some(None) => Some(false),   // not logged in → redirect
            Some(Some(_)) => Some(true), // logged in → show layout
        }
    }
    redirect_path=|| "/login"
>
    <Route path=StaticSegment("") view=HomePage/>
    <Route path=path!("/:username") view=ProfilePage/>
</ProtectedParentRoute>
```

**Note on auth signal type:** `auth_user` is `RwSignal<Option<AuthUser>>`. The outer `Option` is from the signal itself (always `Some` after initialization in this codebase — the signal starts as `None` meaning "not yet validated"). The condition closure must return `Option<bool>`. Returning `None` while JWT is validating prevents a flash redirect to `/login` on page load.

### Pattern 2: Dynamic Route Parameter Reading

**What:** Extract `:username` from the URL inside `ProfilePage`.

**When to use:** Whenever a component needs the current URL's dynamic segment.

**Example:**
```rust
// Source: https://book.leptos.dev/router/18_params_and_queries.html
use leptos_router::hooks::use_params_map;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();
    let username = move || {
        params.read().get("username").unwrap_or_default()
    };
    // username is a reactive closure — re-runs when URL changes
    // ...
}
```

### Pattern 3: Reactive Post List via action.version()

**What:** A `Resource` that lists posts takes `action.version()` as its source signal. When a create or delete action completes, the version increments, causing the resource to re-fetch automatically — no manual `refetch()` call needed.

**When to use:** Whenever a list must update after a mutating ServerAction.

**Example:**
```rust
// Source: https://github.com/leptos-rs/leptos (community-verified pattern)
#[component]
pub fn HomePage() -> impl IntoView {
    let auth_user = use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");

    let create_action = ServerAction::<CreatePost>::new();
    let delete_action = ServerAction::<DeletePost>::new();

    // Resource re-fetches whenever either action version changes
    let posts = Resource::new(
        move || (create_action.version().get(), delete_action.version().get()),
        move |_| async move {
            list_own_posts().await
        },
    );

    view! {
        <ComposeBox action=create_action/>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || posts.get().map(|result| match result {
                Ok(posts) => posts.into_iter().map(|p| view! { <PostCard post=p/> }).collect_view(),
                Err(_) => view! { <p>"Failed to load posts."</p> }.into_any(),
            })}
        </Suspense>
    }
}
```

### Pattern 4: Authorization in delete_post Server Function

**What:** Server function reads `auth_user` from JWT (via a helper that re-decodes the JWT from headers, or by passing `user_id` from the client with server-side verification against the post's `author_id`).

**Critical insight:** Server functions do NOT receive the `AuthUser` context automatically — context is only `PgPool`. Auth must be re-established server-side. The established pattern (Phase 2) encodes `user_id` in the JWT and validates with `validate_token`. For delete, the simplest correct approach: call `validate_token` inside the server function by reading the JWT from a cookie or from a client-passed token argument.

**Practical approach for Phase 3:** Accept `user_id: i32` from the client (from `auth_user.get().map(|u| u.id)`) and verify against `author_id` in the DB. This is safe because the final check is server-side SQL: `DELETE FROM posts WHERE id = $1 AND author_id = $2`.

**Example:**
```rust
// Source: established Phase 2 pattern + SQLx docs
#[leptos::server]
pub async fn delete_post(
    post_id: i32,
    user_id: i32,
) -> Result<(), leptos::prelude::ServerFnError> {
    use sqlx::PgPool;
    let pool = use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Database not available"))?;

    let result = sqlx::query!(
        "DELETE FROM posts WHERE id = $1 AND author_id = $2",
        post_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| leptos::prelude::ServerFnError::new(format!("DB error: {}", e)))?;

    if result.rows_affected() == 0 {
        Err(leptos::prelude::ServerFnError::new("Not authorized or post not found"))
    } else {
        Ok(())
    }
}
```

### Pattern 5: Post with Author Username (JOIN query)

**What:** `list_own_posts` and `get_user_posts` need the author's username to display on the post card. Use a JOIN query returning a struct with both.

**Example:**
```rust
// Source: SQLx docs + established codebase pattern
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostWithAuthor {
    pub id: i32,
    pub author_id: i32,
    pub author_username: String,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
}

// In server function (ssr-only):
let rows = sqlx::query_as!(
    PostWithAuthor,
    r#"SELECT p.id, p.author_id, u.username AS author_username, p.content, p.created_at
       FROM posts p
       JOIN users u ON u.id = p.author_id
       WHERE p.author_id = $1
       ORDER BY p.created_at DESC, p.id DESC"#,
    author_id
)
.fetch_all(&pool)
.await?;
```

**Note:** `PostWithAuthor` must be `Serialize + Deserialize` (not ssr-only) so it can cross the server-function boundary to the WASM client.

### Pattern 6: Relative Timestamp (Server-Side Formatting)

**What:** Compute relative timestamp string in the server function (or in a shared utility). This avoids web-sys dependency and hydration mismatch.

**Example:**
```rust
// Source: chrono docs, project pattern
pub fn relative_timestamp(created_at: chrono::NaiveDateTime) -> String {
    let now = chrono::Utc::now().naive_utc();
    let diff = now.signed_duration_since(created_at);
    if diff.num_seconds() < 60 {
        format!("{}s", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h", diff.num_hours())
    } else {
        format!("{}d", diff.num_days())
    }
}
```

Place in a shared (non-ssr-gated) utility module so it compiles on both server and client. Return the formatted string as part of `PostWithAuthor` or compute server-side and include in the returned struct.

### Anti-Patterns to Avoid

- **Gating `PostWithAuthor` under `#[cfg(feature = "ssr")]`:** The struct must be available in WASM to deserialize server function responses. Only the DB query code should be ssr-gated.
- **Using `use_context::<AuthUser>()` inside a server function:** Context is only `PgPool` in server functions. Auth identity must come from client input (user_id from signal) or be re-derived from JWT.
- **`create_resource` with no reactive dependency on mutations:** A resource that only runs once on mount won't reflect post creation/deletion. Always include the action version(s) in the source.
- **Returning `None` from ProtectedParentRoute condition permanently:** If JWT validation never completes, the app hangs. Ensure the auth effect in `App` always resolves the signal to `Some(Some(user))` or `Some(None)`.
- **`/{username}` conflicting with static routes:** `leptos_router` resolves static segments before dynamic ones when both are at the same level. `/login` and `/register` must be declared as siblings of `/:username`, not nested inside it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Route parameter extraction | Custom URL parsing | `use_params_map()` from `leptos_router::hooks` | Reactive, updates on navigation, handles encoding |
| Auth-gated routes | Conditional rendering in every component | `ProtectedParentRoute` from `leptos_router` | Single point of redirect logic; avoids flash |
| Reactive list after mutation | Manual `refetch()` in every effect | `action.version()` as `Resource` source | Idiomatic Leptos; handles concurrent actions correctly |
| DB cascade on user delete | Application-level post cleanup | `ON DELETE CASCADE` already in `posts` migration | Already implemented in migration 20240101000002 |
| Timestamp formatting | JS `Intl.RelativeTimeFormat` or similar | Server-side chrono arithmetic in server fn | Avoids web-sys, no hydration mismatch, simpler |
| Post ownership check | Client-side hide only | SQL `WHERE id = $1 AND author_id = $2` in delete | `rows_affected() == 0` is the authoritative auth check |

---

## Common Pitfalls

### Pitfall 1: ProtectedParentRoute condition returning None indefinitely
**What goes wrong:** The `auth_user` signal starts as `None` (not yet validated). If the condition returns `None` forever, the route never renders and never redirects. If it returns `Some(false)` before JWT validation completes, users get bounced to `/login` on every page load even when logged in.
**Why it happens:** The auth validation is async (spawn_local + server function call). There's a window of ~50-200ms where the signal is `None`.
**How to avoid:** Return `None` from the condition while the signal is `None` (loading state). Return `Some(false)` only when the signal is definitively `Some(None)` (validation completed, no user). Return `Some(true)` when `Some(Some(_))`.
**Warning signs:** Users get redirected to `/login` briefly on page reload even when they have a valid JWT.

### Pitfall 2: PostWithAuthor not available in WASM
**What goes wrong:** Compilation fails for the `hydrate` feature because `PostWithAuthor` is inside `#[cfg(feature = "ssr")]`.
**Why it happens:** Server functions return values that must be deserializable on the client. The return type struct must be available in both compilation contexts.
**How to avoid:** Define `PostWithAuthor` (and other transfer types) outside any `#[cfg(feature = "ssr")]` block. Only gate the DB query implementations under ssr.
**Warning signs:** WASM build fails with "cannot find type `PostWithAuthor`" or similar.

### Pitfall 3: Static routes (/login, /register) inside ProtectedParentRoute
**What goes wrong:** `/login` and `/register` inherit the auth protection and redirect logged-out users to `/login` — creating an infinite redirect loop.
**Why it happens:** Nesting static routes inside `ProtectedParentRoute` makes them subject to its condition.
**How to avoid:** Place `/login` and `/register` as siblings of the `ProtectedParentRoute`, not children of it.
**Warning signs:** Navigating to `/login` redirects back to `/login` (infinite loop).

### Pitfall 4: ResourceFetch not triggered after post creation
**What goes wrong:** User submits a new post; compose box clears (action succeeded) but the post list doesn't update.
**Why it happens:** The `Resource` was created without including `create_action.version()` in its source function.
**How to avoid:** Always pass `create_action.version().get()` (and `delete_action.version().get()`) into the Resource source tuple.
**Warning signs:** Post appears after manual page reload but not immediately.

### Pitfall 5: /{username} route conflicts
**What goes wrong:** `leptos_router` matches `/login` as the username `"login"` in `/:username` route, never reaching the `LoginPage`.
**Why it happens:** If `/:username` is declared as a `Route` sibling of `/login` inside the same route tree, order and specificity matter.
**How to avoid:** Verify that `leptos_router` 0.8 gives static segments priority over dynamic ones at the same level. Based on documentation this is the case — static wins over dynamic. Keep `/login` and `/register` as sibling routes at the same nesting level as `/:username`, not after a wildcard.
**Warning signs:** Login page shows a profile page for user "login".

### Pitfall 6: Hydration mismatch on timestamps
**What goes wrong:** SSR renders a relative timestamp at server request time; client hydrates with a different time-since value, causing a hydration warning or visible flicker.
**Why it happens:** Time passes between SSR render and client hydration. Timestamps computed client-side differ from server-side.
**How to avoid:** For Phase 3, accept a small staleness — the timestamp returned from the server function is already a computed string. Since post lists are loaded via server functions (not SSR-streamed initial data that would differ), hydration mismatch on timestamps is not expected. If it appears, wrap timestamps in a `<Show when=is_client>` pattern.

---

## Code Examples

Verified patterns from official sources and established codebase:

### Server Function: Create Post
```rust
// Source: established Phase 2 pattern (handlers.rs)
#[leptos::server]
pub async fn create_post(
    content: String,
    user_id: i32,
) -> Result<(), leptos::prelude::ServerFnError> {
    use sqlx::PgPool;
    let pool = leptos::prelude::use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Database not available"))?;

    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        return Err(leptos::prelude::ServerFnError::new("Post cannot be empty"));
    }
    if trimmed.chars().count() > 280 {
        return Err(leptos::prelude::ServerFnError::new("Post exceeds 280 characters"));
    }

    sqlx::query!(
        "INSERT INTO posts (author_id, content) VALUES ($1, $2)",
        user_id,
        trimmed
    )
    .execute(&pool)
    .await
    .map_err(|e| leptos::prelude::ServerFnError::new(format!("DB error: {}", e)))?;

    Ok(())
}
```

### Server Function: List Posts for a User
```rust
// Source: SQLx + established Phase 2 pattern
#[leptos::server]
pub async fn get_user_posts(
    username: String,
) -> Result<Vec<PostWithAuthor>, leptos::prelude::ServerFnError> {
    use sqlx::PgPool;
    let pool = leptos::prelude::use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Database not available"))?;

    let rows = sqlx::query!(
        r#"SELECT p.id, p.author_id, u.username AS author_username, p.content, p.created_at
           FROM posts p
           JOIN users u ON u.id = p.author_id
           WHERE u.username = $1
           ORDER BY p.created_at DESC, p.id DESC"#,
        username
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| leptos::prelude::ServerFnError::new(format!("DB error: {}", e)))?;

    Ok(rows.iter().map(|r| PostWithAuthor {
        id: r.id,
        author_id: r.author_id,
        author_username: r.author_username.clone(),
        content: r.content.clone(),
        created_at: r.created_at,
    }).collect())
}
```

### Compose Box Component (Character Count + Disable Logic)
```rust
// Source: Leptos 0.8 reactive signals pattern (book.leptos.dev)
#[component]
pub fn ComposeBox(action: ServerAction<CreatePost>) -> impl IntoView {
    let auth_user = use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");
    let content = RwSignal::new(String::new());
    let char_count = Signal::derive(move || content.get().chars().count());
    let is_over_limit = Signal::derive(move || char_count.get() > 280);
    let is_empty = Signal::derive(move || content.get().trim().is_empty());
    let chars_remaining = Signal::derive(move || 280i32 - char_count.get() as i32);

    let on_submit = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        if let Some(Some(user)) = auth_user.get() {
            action.dispatch(CreatePost {
                content: content.get(),
                user_id: user.id,
            });
            content.set(String::new());
        }
    };

    view! {
        <div style="border-bottom: 1px solid #e1e8ed; padding: 1rem;">
            <textarea
                prop:value=move || content.get()
                on:input=move |ev| content.set(event_target_value(&ev))
                placeholder="What's happening?"
                style="width: 100%; resize: none; border: none; outline: none; font-size: 1.25rem; min-height: 80px;"
            />
            <div style="display: flex; justify-content: flex-end; align-items: center; gap: 1rem;">
                <span style=move || format!(
                    "font-size: 0.9rem; color: {};",
                    if is_over_limit.get() { "#e0245e" } else { "#657786" }
                )>
                    {move || chars_remaining.get().to_string()}
                </span>
                <button
                    on:click=on_submit
                    disabled=move || is_empty.get() || is_over_limit.get()
                    style="padding: 0.5rem 1.25rem; background: #1d9bf0; color: white; border: none; border-radius: 9999px; font-weight: 700; cursor: pointer;"
                >
                    "Post"
                </button>
            </div>
        </div>
    }
}
```

### Route Structure in app.rs
```rust
// Source: https://docs.rs/leptos_router/latest/leptos_router/components/
// https://book.leptos.dev/router/17_nested_routing.html
<Router>
    <main>
        <Routes fallback=|| "Page not found.".into_view()>
            // Unauthenticated routes — no sidebar
            <Route path=StaticSegment("login") view=LoginPage/>
            <Route path=StaticSegment("register") view=RegisterPage/>

            // Authenticated routes — wrapped in layout with sidebar
            <ProtectedParentRoute
                path=StaticSegment("")
                view=AuthenticatedLayout
                condition=move || {
                    // None = still checking JWT; Some(false) = not logged in
                    auth_user.with(|u| match u {
                        None => None,
                        Some(None) => Some(false),
                        Some(Some(_)) => Some(true),
                    })
                    // Wait — auth_user is RwSignal<Option<AuthUser>>, not nested Option
                    // .get() returns Option<AuthUser>
                    // None = not logged in, Some(_) = logged in
                }
                redirect_path=|| "/login"
            >
                <Route path=StaticSegment("") view=HomePage/>
                <Route path=path!("/:username") view=ProfilePage/>
            </ProtectedParentRoute>
        </Routes>
    </main>
</Router>
```

**Note on auth_user type:** `auth_user` is `RwSignal<Option<AuthUser>>`. It is initialized to `None` (unauthenticated) on page load. The JWT validation Effect sets it to `Some(user)` after the server round-trip. For the condition: `None` = not logged in / not yet validated; `Some(AuthUser{...})` = logged in.

Corrected condition:
```rust
condition=move || {
    // Phase 2 auth validation is async — we can't distinguish "loading" from "not logged in"
    // with a plain RwSignal<Option<AuthUser>>.
    // Safest Phase 3 approach: treat None as Some(false) (redirect to login).
    // The JWT validation Effect runs immediately on mount — the window is tiny.
    // Navigate back after login restores auth state.
    Some(auth_user.get().is_some())
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `create_resource` (Leptos 0.6/0.7 API) | `Resource::new(source, fetcher)` | Leptos 0.7+ | Same concept, new API surface |
| `create_action` | `Action::new` / `ServerAction::new` | Leptos 0.7+ | `ServerAction::<MyFn>::new()` is current idiom |
| `ProtectedRoute` had issues with async conditions | `ProtectedRoute` + `ProtectedParentRoute` work with resources as of late 2025 | 2025 | Safe to use for auth guards |

**Deprecated/outdated:**
- `create_resource`: replaced by `Resource::new(...)` in 0.7+
- `create_action`: replaced by `Action::new(...)` / `ServerAction::new()` in 0.7+
- Both old APIs may still work as aliases but should not be used in new code

---

## Open Questions

1. **ProtectedParentRoute: None-state race condition on page load**
   - What we know: `auth_user` starts as `None`; JWT validation completes asynchronously; `ProtectedParentRoute` condition fires reactively
   - What's unclear: Whether `Some(auth_user.get().is_some())` (which immediately returns `Some(false)` before JWT validates) will cause a login redirect flash
   - Recommendation: Spike this first. If flash is observed, introduce a tri-state (`Loading | Authenticated | Unauthenticated`) signal instead of `Option<AuthUser>`. Phase 2 already had a similar concern (navigate after spawn_local) and solved it. The same pattern applies here.

2. **`path!("/:username")` vs `path!("{username}")` macro syntax**
   - What we know: Documentation shows `path!("/users/:id")` with colon syntax for dynamic segments
   - What's unclear: Whether the macro inside a `Route` nested inside `ProtectedParentRoute` uses a leading slash or not
   - Recommendation: Try `path!("/:username")` first (with leading slash, as sibling of root). If routing doesn't match, try `path!(":username")` (no leading slash — relative to parent path).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` with `#[cfg(test)]` modules (no external test runner) |
| Config file | none — inline test modules only (established Phase 2 pattern) |
| Quick run command | `cargo test --features ssr` |
| Full suite command | `cargo test --features ssr` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| POST-01 | `create_post` rejects empty content | unit | `cargo test --features ssr post::create_post_rejects_empty` | ❌ Wave 0 |
| POST-01 | `create_post` rejects content > 280 chars | unit | `cargo test --features ssr post::create_post_rejects_over_limit` | ❌ Wave 0 |
| POST-02 | `delete_post` with wrong user_id returns error | unit | `cargo test --features ssr post::delete_post_rejects_wrong_user` | ❌ Wave 0 |
| POST-03 | `PostWithAuthor` serializes and deserializes correctly | unit | `cargo test --features ssr post::post_with_author_serde_roundtrip` | ❌ Wave 0 |
| POST-03 | `relative_timestamp` formats all ranges correctly | unit | `cargo test relative_timestamp_formats` | ❌ Wave 0 |
| PROF-01 | `get_user_posts` returns posts in DESC order | unit (logic) | `cargo test --features ssr post::user_posts_order` | ❌ Wave 0 |
| PROF-02 | Same as PROF-01 (same server fn, param-driven) | n/a | covered by PROF-01 test | n/a |

### Sampling Rate
- **Per task commit:** `cargo test --features ssr`
- **Per wave merge:** `cargo test --features ssr`
- **Phase gate:** Full suite green + manual browser smoke test before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/server/posts/handlers.rs` — create server module for posts
- [ ] `src/server/posts/mod.rs` — module definition
- [ ] Test functions for POST-01, POST-02, POST-03, PROF-01 (inline `#[cfg(test)]` in `handlers.rs`)
- [ ] `PostWithAuthor` struct definition (non-ssr-gated, in a shared module)
- [ ] `relative_timestamp` utility function

---

## Sources

### Primary (HIGH confidence)
- `https://docs.rs/leptos_router/latest/leptos_router/components/` — ProtectedRoute, ProtectedParentRoute, ParentRoute component signatures
- `https://book.leptos.dev/router/17_nested_routing.html` — ParentRoute + Outlet pattern, confirmed syntax
- `https://book.leptos.dev/router/18_params_and_queries.html` — `use_params_map()` hook, confirmed syntax
- `https://book.leptos.dev/router/16_routes.html` — `path!` macro syntax for dynamic segments
- `https://book.leptos.dev/async/10_resources.html` — `Resource::new(source, fetcher)` API
- Project source code (`src/server/auth/handlers.rs`, `src/app.rs`, `src/auth_user.rs`, `Cargo.toml`) — established patterns

### Secondary (MEDIUM confidence)
- `https://github.com/leptos-rs/leptos/issues/2743` + related discussions — `action.version()` as Resource source for refetch-after-mutation pattern
- WebSearch + multiple community sources confirming `ProtectedRoute` works with resources as of 2025

### Tertiary (LOW confidence)
- Exact behavior of `ProtectedParentRoute` condition with `None` during JWT validation race — not directly confirmed by official docs, flagged as open question

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in use, versions confirmed from Cargo.toml
- Architecture: HIGH — routing patterns confirmed from official Leptos docs; server function patterns mirror Phase 2 established code
- Pitfalls: MEDIUM-HIGH — most confirmed from official docs or direct codebase analysis; JWT race condition is LOW (not officially documented)
- Validation: HIGH — follows existing Phase 2 test pattern exactly

**Research date:** 2026-03-12
**Valid until:** 2026-04-12 (stable framework; Leptos 0.8.x patch releases unlikely to break these APIs)
