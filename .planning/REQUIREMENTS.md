# Requirements: My_X

**Defined:** 2026-03-11
**Core Value:** A working Rust full-stack app that teaches ownership, async, and real-world patterns by implementing the essential social graph of Twitter.

## v1 Requirements

### Authentication

- [x] **AUTH-01**: User can create an account with email and password
- [x] **AUTH-02**: User can log in with email and password and receive a JWT
- [x] **AUTH-03**: User session persists across browser refresh (JWT stored client-side)
- [x] **AUTH-04**: User can log out and have their session invalidated

### Posts

- [ ] **POST-01**: Authenticated user can create a text post (max 280 characters)
- [ ] **POST-02**: Authenticated user can delete their own post
- [ ] **POST-03**: Post displays author username and creation timestamp

### Profiles

- [ ] **PROF-01**: User can view their own profile with a list of their posts
- [ ] **PROF-02**: User can view another user's profile with their post list

### Social Graph

- [ ] **SOCL-01**: Authenticated user can follow another user
- [ ] **SOCL-02**: Authenticated user can unfollow a user they follow
- [ ] **SOCL-03**: Profile page shows whether the viewer follows that user

### Feed

- [ ] **FEED-01**: Authenticated user can view a home feed of posts from users they follow
- [ ] **FEED-02**: Feed is ordered chronologically (newest first)
- [ ] **FEED-03**: Feed is paginated using cursor-based pagination

## v2 Requirements

### Authentication

- **AUTH-V2-01**: User can reset password via email link

### Posts

- **POST-V2-01**: User can edit their own post

### Profiles

- **PROF-V2-01**: User can edit their display name and bio
- **PROF-V2-02**: Profile shows follower and following counts

### Social

- **SOCL-V2-01**: User can view their list of followers and who they follow

## Out of Scope

| Feature | Reason |
|---------|--------|
| Likes / reactions | Separate engagement mechanic, no new Rust patterns |
| Replies / threads | Recursive data model — its own learning topic |
| Retweets / reposts | Adds feed complexity without new Rust patterns |
| Media uploads | File storage / object storage — separate domain |
| Real-time feed updates | WebSockets lifecycle — separate async topic |
| Search | Full-text indexing — separate subsystem |
| Notifications | Not core to the social graph learning goal |
| DMs | Separate messaging domain |
| Mobile app | Web-first, local-only |
| OAuth (Google, GitHub) | Email/password sufficient for v1 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| AUTH-01 | Phase 2 | Complete |
| AUTH-02 | Phase 2 | Complete |
| AUTH-03 | Phase 2 | Complete |
| AUTH-04 | Phase 2 | Complete |
| POST-01 | Phase 3 | Pending |
| POST-02 | Phase 3 | Pending |
| POST-03 | Phase 3 | Pending |
| PROF-01 | Phase 3 | Pending |
| PROF-02 | Phase 3 | Pending |
| SOCL-01 | Phase 4 | Pending |
| SOCL-02 | Phase 4 | Pending |
| SOCL-03 | Phase 4 | Pending |
| FEED-01 | Phase 4 | Pending |
| FEED-02 | Phase 4 | Pending |
| FEED-03 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-11*
*Last updated: 2026-03-11 after roadmap creation*
