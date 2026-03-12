# TODO

## Phase 3 — Posts + Profiles

**Goal:** Authenticated users can create/delete posts, and any user's profile with their post history is viewable.

Requirements: POST-01, POST-02, POST-03, PROF-01, PROF-02

- [ ] **3.1 — Server functions** (`src/server/posts/`)
  - `PostWithAuthor` struct (outside ssr gate — must compile in WASM)
  - `relative_timestamp()` utility (s/m/h/d)
  - `create_post` — trim, reject empty / >280 chars, INSERT
  - `delete_post` — DELETE WHERE id=$1 AND author_id=$2, check rows_affected
  - `list_own_posts` — user's own posts, DESC
  - `get_user_posts` — posts by username, DESC
  - Unit tests for all validation logic (TDD)
  - Wire `pub mod posts` into `src/server/mod.rs`

- [ ] **3.2 — Home page UI** (`src/components/`)
  - `ComposeBox` — textarea + char count + Post button (disabled when empty or >280)
  - `PostCard` — avatar circle, clickable @username, relative timestamp, post text, `...` delete menu (own posts only)
  - `HomePage` — compose box + reactive post list (own posts in Phase 3, feed in Phase 4)
  - After post: clear compose, new post appears at top immediately (no reload)

- [ ] **3.3 — Layout + Profile + Routing** (`src/app.rs`, `src/components/`)
  - `Sidebar` — logo, Home, Profile, Post button, Logout (persistent on all auth pages)
  - `AuthenticatedLayout` — sidebar + main content area wrapper
  - `ProfilePage` — banner, avatar circle, display name, @handle, bio placeholder, post count, post list
  - Same component for own profile and others' profiles; delete button only on own posts
  - `/{username}` route (dynamic), must not conflict with `/login` and `/register`
  - All auth pages redirect to `/login` if no valid session

## Phase 4 — Social Graph + Feed

Requirements: SOCL-01, SOCL-02, SOCL-03, FEED-01, FEED-02, FEED-03

- [ ] Follow / unfollow another user (button on profile page, reactive state update)
- [ ] Profile page shows follow state (following / not following)
- [ ] Home feed: posts from all followed users, newest first
- [ ] Cursor-based pagination on home feed (created_at, id)
- [ ] Replace Phase 3 home feed (own posts only) with following-based feed

## Phase 5 — Polish

- [ ] Loading states on feed and profile post lists (no blank flash)
- [ ] After creating a post: redirect to feed, new post visible at top
- [ ] All auth pages redirect to `/login` without valid session, return to intended destination after login
