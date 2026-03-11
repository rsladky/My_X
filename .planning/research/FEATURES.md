# Feature Research

**Domain:** Full-stack Twitter/X clone (Rust learning project)
**Researched:** 2026-03-11
**Confidence:** HIGH (core social features are well-established; scoping decisions grounded in PROJECT.md)

## Feature Landscape

### Table Stakes (Users Expect These)

Features that any Twitter clone must have. Missing these makes the product feel broken or incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Email/password registration | Entry point to the whole product — no auth, no app | LOW | Standard email + hashed password; bcrypt or argon2 in Rust |
| Login / logout | Session lifecycle is a baseline assumption | LOW | JWT or server-side session; both are valid in Axum/Actix |
| Create a text post ("tweet") | Core product action — the whole point of the app | LOW | 280-char cap (or configurable); stored with author + timestamp |
| View own profile | Users expect to see their posts and identity | LOW | Public page: avatar, display name, bio, post history |
| View another user's profile | Discovery and social graph navigation | LOW | Same structure as own profile, minus edit controls |
| Follow a user | Core social graph primitive — without it, there's no feed | LOW | Asymmetric follow (A follows B ≠ B follows A) like Twitter |
| Unfollow a user | Follows without unfollowing are unusable | LOW | Single DB delete; frontend toggle |
| Home feed (chronological) | The payoff for following — must show posts from followed users | MEDIUM | Query posts from all followed users, order by time descending |
| Delete own post | Basic content control; absence feels punishing | LOW | Soft or hard delete; restrict to post author |
| Persistent login | Logging in on every page load is unusable | LOW | Token in localStorage or httpOnly cookie; validate on each request |

### Differentiators (Competitive Advantage)

For a learning project, "differentiating" means features that deepen Rust knowledge rather than product novelty.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Rust WASM frontend (Leptos/Yew) | The entire point of the project — full Rust end-to-end, rare in Twitter clones | HIGH | Leptos preferred; SSR support, active ecosystem, better DX than Yew as of 2025 |
| Server-side rendering (SSR) | Demonstrates Rust's async power; real production pattern | HIGH | Leptos natively supports SSR; teaches Axum + WASM integration |
| Type-safe DB queries (SQLx) | Rust's compile-time SQL checking is a genuine differentiator; teaches a real Rust pattern | MEDIUM | SQLx macros check queries at compile time against live DB — distinct learning value |
| Compile-time safety across the stack | Using Rust types from DB schema → API → frontend eliminates whole classes of bugs; showcase-worthy | HIGH | Shared types crate between backend + frontend; teach Cargo workspace structure |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem natural to add but would hurt this project's learning goal.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Likes / reactions | Standard in every Twitter clone tutorial | Adds social mechanics complexity without teaching new Rust patterns; distracts from core graph | Skip for v1; revisit after follow + feed are solid |
| Replies / threads | Feels "incomplete" without them | Recursive data structures (parent_id chains) add meaningful DB + UI complexity before basics are working | Address in v2 once post model is stable |
| Retweets / reposts | Iconic Twitter feature | Requires content attribution, quote-tweet variants, feed ranking logic; scope creep for a learning project | Skip entirely; core value is the follow graph, not content amplification |
| Media uploads (images/video) | Adds visual richness | Multipart forms, object storage (S3/local FS), MIME handling — significant Rust complexity unrelated to social mechanics | Text-only posts for v1; clean scope boundary |
| Real-time feed updates (WebSockets) | Modern UX expectation | WebSocket lifecycle in Axum + WASM frontend is a substantial standalone topic; adds concurrency complexity before basics are taught | Polling on page load is sufficient; real-time is a standalone future phase |
| Search (users, posts) | Users expect it | Full-text search requires indexing strategy (pg tsvector, Meilisearch, etc.); entirely separate subsystem | Not needed for learning social graph; add only if project expands |
| Notifications | Platform polish | Requires event bus or background workers; teaches infrastructure, not Rust web patterns | Out of scope per PROJECT.md; keep excluded |
| Direct messaging (DMs) | Common feature request | Real-time messaging is a separate domain (WebSockets, message ordering, read receipts); doubles project scope | Explicitly out of scope; separate future project |
| Trending / Explore / Recommendations | Engagement driver | Algorithmic ranking requires data volumes and ML tooling that don't exist in a learning project | Chronological feed is the right call; avoids fake complexity |
| OAuth / social login (Google, GitHub) | Reduces friction | Adds OAuth2 flow complexity; teaches OAuth, not Rust web patterns | Email/password teaches the same JWT/session patterns more directly |

## Feature Dependencies

```
[Auth: Registration]
    └──required by──> [Auth: Login]
                          └──required by──> [Create Post]
                          └──required by──> [Follow User]
                          └──required by──> [View Own Profile]
                          └──required by──> [Delete Own Post]

[Follow User]
    └──required by──> [Home Feed]
                          (feed has no content without follows)

[User exists in DB]
    └──required by──> [View Profile]
    └──required by──> [Follow User]

[Create Post]
    └──enhances──> [View Profile] (post history on profile)
    └──required by──> [Home Feed] (feed needs posts to show)

[Persistent Login (token/session)]
    └──enhances──> [All authenticated routes]
```

### Dependency Notes

- **Auth is the root dependency:** Every other feature requires a logged-in user. Auth must be Phase 1.
- **Follow requires users to exist:** The social graph depends on having at least two registered users navigable by profile.
- **Home feed requires both Follow and Post:** An empty follow list or no posts produces an empty feed — both must work before the feed is meaningful to test.
- **Profile view depends on Post:** A profile without post history is an incomplete loop; Create Post and View Profile should land in the same phase.

## MVP Definition

### Launch With (v1)

Minimum to demonstrate the core social graph working end-to-end.

- [ ] User registration with email + password — foundation for everything
- [ ] User login / logout with JWT or session cookie — enables all authenticated routes
- [ ] Create a text post (up to 280 chars) — the core product action
- [ ] View own profile with post history — closes the loop for the author
- [ ] View another user's profile — enables social discovery
- [ ] Follow and unfollow a user — the social graph primitive
- [ ] Home feed (posts from followed users, chronological) — the payoff for following

### Add After Validation (v1.x)

Improvements once the core loop works and Rust patterns are understood.

- [ ] Delete own post — add once CRUD patterns are established
- [ ] Edit profile (display name, bio, avatar URL) — after profile rendering is stable
- [ ] Follower / following counts on profiles — once follow data is in place
- [ ] Pagination on feed and profile post lists — once list rendering works

### Future Consideration (v2+)

Features to defer until v1 is complete and a new learning goal is defined.

- [ ] Likes — only if engagement mechanics become a learning objective
- [ ] Replies / threads — complex recursive data model; standalone learning topic
- [ ] Real-time feed updates via WebSockets — teach async + concurrency separately
- [ ] Media uploads — teach file handling + object storage separately
- [ ] Search — teach full-text indexing as a standalone topic

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Registration + Login | HIGH | LOW | P1 |
| Create post | HIGH | LOW | P1 |
| View profile (own + other) | HIGH | LOW | P1 |
| Follow / unfollow | HIGH | LOW | P1 |
| Home feed | HIGH | MEDIUM | P1 |
| Delete post | MEDIUM | LOW | P1 |
| Edit profile | MEDIUM | LOW | P2 |
| Follower/following counts | MEDIUM | LOW | P2 |
| Pagination | MEDIUM | MEDIUM | P2 |
| Likes | LOW | LOW | P3 |
| Replies | MEDIUM | HIGH | P3 |
| Real-time updates | MEDIUM | HIGH | P3 |
| Media uploads | MEDIUM | HIGH | P3 |
| Search | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for v1 launch
- P2: Should have, add once core is working
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Mastodon | Bluesky | Twitter/X | Our Approach |
|---------|----------|---------|-----------|--------------|
| Auth | Email + OAuth | DID-based | Email + OAuth | Email/password only — simpler, teaches JWT patterns directly |
| Post model | 500-char limit, media | 300-char, media | 280-char, media rich | Text only, 280-char cap — no media for v1 |
| Feed | Chronological, no algorithm | Algorithmic + custom feeds | Algorithmic (opaque) | Chronological only — correct for learning; avoids fake complexity |
| Follow model | Asymmetric | Asymmetric | Asymmetric | Asymmetric — standard social graph; A follows B is independent of B follows A |
| Interactions | Boost, favorite, reply | Like, repost, reply, quote | Like, retweet, reply, bookmark | None in v1 — deliberately excluded per PROJECT.md |
| Profiles | Avatar, bio, header | Avatar, bio, banner | Rich profile with links, pinned posts | Minimal: display name, bio, post history |

## Sources

- [GitHub: twitter-clone topic (424 repositories)](https://github.com/topics/twitter-clone) — surveyed common feature sets across popular implementations
- [Zero To Mastery: Build a Twitter Clone with Rust](https://zerotomastery.io/courses/rust-project-twitter-clone/) — Rust-specific scope reference
- [GetStream: Build a Twitter Clone](https://getstream.io/blog/build-twitter-clone/) — feature set reference
- [Adalo: Building a Twitter/X Clone](https://www.adalo.com/posts/step-by-step-guide-building-twitter-x-clone-with-adalo) — feature scope reference
- [Digital Software Labs: Twitter Clone features](https://digitalsoftwarelabs.com/whitelabelapp/twitter-clone-app/) — market feature expectations
- [Software Mind: How to Make a Social Media App 2025](https://softwaremind.com/blog/how-to-make-a-social-media-app-a-step-by-step-guide/) — table stakes for social apps

---
*Feature research for: Full-stack Twitter/X clone in Rust*
*Researched: 2026-03-11*
