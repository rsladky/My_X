# Phase 3: Posts + Profiles - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Authenticated users can create and delete posts, and any user's profile with their post history is viewable. Following, feed from followed users, and social graph are Phase 4. Likes, replies, and media are out of scope for v1.

</domain>

<decisions>
## Implementation Decisions

### App Navigation
- Left sidebar layout, X-style: logo top-left, Home link, Profile link (to own profile), a Post button, and Logout at bottom
- Sidebar is persistent on all authenticated pages
- Auth/login and register pages have no sidebar — standalone centered layout (consistent with Phase 2)
- All pages (/ and /{username}) require authentication — unauthenticated visitors redirect to /login
- A shared layout component wraps all authenticated pages (sidebar + main content area)

### Profile URLs
- X-style: `/{username}` (e.g. `/robinsladky`) — no `/users/` prefix
- Routing must not conflict with `/login` and `/register` (static segments take priority)
- Navigation to another user's profile: clicking their @username on a post

### Profile Page
- Full X-style header skeleton: banner area (grey box), avatar circle (grey placeholder), display name, @handle, bio area (placeholder text), post count
- Post count displayed under the handle (e.g. "42 posts")
- Same page component for own profile and others' profiles
- Delete button only appears on posts where `post.author_id == auth_user.id` — no UI difference otherwise

### Home Page (/)
- Phase 3 home shows a compose box + the authenticated user's own posts in reverse-chronological order
- Phase 4 will replace this with the full social feed (following-based)
- Compose box at top of feed, always visible (X-style)
- Compose: text area + remaining character count (e.g. "234") + Post button
- Post button disabled when textarea is empty or character count exceeds 280
- After posting: compose clears, new post appears at top of feed immediately (reactive update — no page reload)

### Post Card Display
- Each post card: grey circle avatar placeholder | @username (clickable link to /{username}) | relative timestamp (e.g. "2h", "3d") | post text
- Posts separated by thin border-bottom divider (no card shadow) — X-style high-density list
- Action buttons (reply, retweet, like): out of scope for Phase 3
- Delete: `...` overflow menu top-right of card, only rendered on own posts — clicking reveals "Delete" option

### Timestamp Format
- Relative timestamps for display: seconds → "Xs", minutes → "Xm", hours → "Xh", days → "Xd"
- No hover/absolute fallback in Phase 3 (Claude's discretion to add title attribute for accessibility)

### Claude's Discretion
- Exact sidebar width and spacing
- Color palette details (should follow X's dark-on-light or dark theme — either is fine)
- Avatar placeholder exact shade
- Overflow menu open/close interaction mechanics (toggle on click)
- Error handling UX for failed post creation (single message pattern from Phase 2)

</decisions>

<specifics>
## Specific Ideas

- "Make it as close as possible to real X" — the north star for all visual/interaction decisions
- Header skeleton shape should be there even if data is empty (banner box, avatar circle, name/handle/bio placeholders)
- Home feed in Phase 3 is a stepping stone — Phase 4 replaces it with following-based feed; design home page to make that swap clean

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AuthUser` (src/auth_user.rs): carries `user_id` (UUID) + `username` — available via `use_context::<RwSignal<Option<AuthUser>>>()` in all child components
- `AppError::AuthError` + `AppError::ValidationError` (src/server/error.rs): reuse for post permission errors and validation (empty post, over 280 chars)
- `src/components/login_page.rs` + `register_page.rs`: reference for server function call pattern, error display pattern (single message above submit), and spawn_local usage

### Established Patterns
- Server functions only for frontend↔backend (no manual fetch, no REST client) — established Phase 2
- Inline styles for all UI (no CSS framework, no Tailwind) — existing codebase pattern
- `RwSignal` + `use_context` for shared state — auth signal pattern to replicate for post list
- Feature-gate all server-only crates under `[ssr]` in Cargo.toml — required for WASM compilation

### Integration Points
- `src/app.rs`: Add `/{username}` route (dynamic segment), update `/` to new HomePage, add authenticated layout wrapper with sidebar
- `src/server/mod.rs`: Add `posts` submodule for create/delete/list server functions
- DB: `posts` table already exists from Phase 1 migration — verify columns match (id, author_id, content, created_at)
- `src/components/mod.rs`: Add new components (sidebar, profile page, post card, compose box)

### Known Risk (from STATE.md)
- Leptos SSR + Axum hydration is the least-documented part of the stack
- Research recommends a focused spike (trivial SSR page compiling and hydrating) as the first deliverable of Phase 3 before building full components

</code_context>

<deferred>
## Deferred Ideas

- Follow/unfollow button on profiles — Phase 4
- Follower/following counts on profile — Phase 4 (SOCL-03)
- Action buttons on post cards (reply, retweet, like) — out of scope v1
- Real avatar/photo upload — out of scope v1 (media uploads excluded)
- Edit profile (display name, bio) — v2 (PROF-V2-01)
- Hover tooltip showing absolute timestamp — nice-to-have, Claude can add as title attribute

</deferred>

---

*Phase: 03-posts-+-profiles*
*Context gathered: 2026-03-12*
