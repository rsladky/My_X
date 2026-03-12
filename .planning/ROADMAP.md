# Roadmap: My_X

## Overview

My_X is a full-stack Rust Twitter/X clone built for learning. The journey runs from an empty Cargo workspace to a working social network: scaffold the project and database, add authentication, build posts and profiles, wire up the social graph and home feed, then polish the UX. Every phase delivers a coherent, testable capability — nothing works until auth works, nothing social works until posts exist, and the feed is the payoff that makes it feel like a real product.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation** - Cargo workspace, Axum server, PostgreSQL schema, SQLx offline mode
- [x] **Phase 2: Auth** - Registration, login, JWT middleware, persistent session, logout (completed 2026-03-12)
- [ ] **Phase 3: Posts + Profiles** - Create/delete posts, view own and other users' profiles with post history
- [ ] **Phase 4: Social Graph + Feed** - Follow/unfollow, follow state on profiles, home feed with cursor pagination
- [ ] **Phase 5: Polish** - Loading states, post-create redirect, UX completeness across all pages

## Phase Details

### Phase 1: Foundation
**Goal**: A runnable full-stack Rust project with a healthy database schema that all subsequent work can build on
**Depends on**: Nothing (first phase)
**Requirements**: None (scaffolding phase — gates all other requirements)
**Success Criteria** (what must be TRUE):
  1. `cargo leptos watch` starts the server without errors and serves an HTML response at `localhost`
  2. `cargo sqlx migrate run` applies all migrations (users, posts, follows tables with indexes) against a local PostgreSQL instance without errors
  3. `SQLX_OFFLINE=true cargo build` compiles successfully, confirming offline mode is configured and `.sqlx/` is committed
  4. A deliberate bad handler returns a structured `AppError` JSON response (not a panic or empty 500)
**Plans**: 3 plans
Plans:
- [ ] 01-01-PLAN.md — Scaffold Cargo workspace with start-axum template and all dependencies
- [ ] 01-02-PLAN.md — Database migrations (users, posts, follows) and SQLx offline mode
- [ ] 01-03-PLAN.md — AppError enum with IntoResponse and test route

### Phase 2: Auth
**Goal**: Users can securely create accounts, log in, stay logged in across sessions, and log out
**Depends on**: Phase 1
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04
**Success Criteria** (what must be TRUE):
  1. User can submit the register form with email and password and land on a logged-in page without error
  2. User can log in with correct credentials and have their JWT persisted in localStorage so a page refresh keeps them logged in
  3. User attempting to log in with wrong credentials sees the same generic error message as a non-existent account (no account enumeration)
  4. User can click logout from any authenticated page and be redirected to the login page with their session cleared
**Plans**: 2 plans
Plans:
- [ ] 02-01-PLAN.md — DB migration (username column), auth server functions (register/login/validate_token), PgPool wiring, unit tests
- [ ] 02-02-PLAN.md — Login and register UI components, auth context signal, localStorage persistence, logout

### Phase 3: Posts + Profiles
**Goal**: Authenticated users can create and delete posts, and any user's profile with their post history is viewable
**Depends on**: Phase 2
**Requirements**: POST-01, POST-02, POST-03, PROF-01, PROF-02
**Success Criteria** (what must be TRUE):
  1. Authenticated user can submit a post (up to 280 characters) and see it appear immediately with their username and a timestamp
  2. Authenticated user can delete one of their own posts and see it disappear from the page; attempting to delete another user's post is rejected
  3. User can navigate to their own profile page and see all their posts in reverse-chronological order
  4. User can navigate to another user's profile page and see that user's posts in reverse-chronological order
**Plans**: 3 plans
Plans:
- [ ] 03-01-PLAN.md — PostWithAuthor types, create/delete/list server functions, unit tests (TDD)
- [ ] 03-02-PLAN.md — ComposeBox, PostCard, HomePage UI with reactive post list
- [ ] 03-03-PLAN.md — Sidebar, AuthenticatedLayout, ProfilePage, ProtectedParentRoute routing

### Phase 4: Social Graph + Feed
**Goal**: Authenticated users can follow and unfollow others, see follow state on profiles, and read a home feed of posts from people they follow
**Depends on**: Phase 3
**Requirements**: SOCL-01, SOCL-02, SOCL-03, FEED-01, FEED-02, FEED-03
**Success Criteria** (what must be TRUE):
  1. Authenticated user can follow another user from their profile page; the follow button changes state immediately without a full page reload
  2. Authenticated user can unfollow a user they already follow; the follow button reflects the new state immediately
  3. Profile page shows whether the currently logged-in viewer follows that user (correct for both following and not-following states)
  4. Authenticated user can view their home feed showing posts from all users they follow, ordered newest first
  5. Home feed loads the next page of posts via cursor-based pagination without duplicating or skipping posts when new content is posted between page loads
**Plans**: TBD

### Phase 5: Polish
**Goal**: The full app feels complete — all pages have loading states, navigation is consistent, and the post-create flow lands the user in the right place
**Depends on**: Phase 4
**Requirements**: None (UX completeness — no new v1 requirements)
**Success Criteria** (what must be TRUE):
  1. Feed and profile post lists show a visible loading indicator while data is being fetched (no blank white flash)
  2. After creating a post, the user is automatically redirected to the feed and the new post is visible at the top
  3. All authenticated pages redirect to the login page when accessed without a valid session, and return the user to their intended destination after login
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 1/3 | In Progress|  |
| 2. Auth | 2/2 | Complete   | 2026-03-12 |
| 3. Posts + Profiles | 0/TBD | Not started | - |
| 4. Social Graph + Feed | 0/TBD | Not started | - |
| 5. Polish | 0/TBD | Not started | - |
