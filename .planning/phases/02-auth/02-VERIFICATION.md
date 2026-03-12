---
phase: 02-auth
verified: 2026-03-12T00:00:00Z
status: human_needed
score: 4/4 must-haves verified
human_verification:
  - test: "Submit register form with valid email/password, observe redirect and welcome page"
    expected: "User lands on home page showing 'Welcome, <username>' with a logout button"
    why_human: "Browser rendering and navigation cannot be verified programmatically"
  - test: "After successful login, refresh the page"
    expected: "Session is restored — home page still shows the logged-in welcome view, not the 'Not logged in' view"
    why_human: "localStorage read + validate_token round-trip across page reload is a live runtime behaviour"
  - test: "Submit login with wrong password, then with an email that does not exist"
    expected: "Both cases show exactly 'Invalid email or password' — no difference in the error message"
    why_human: "Visual error message equality across two distinct failure paths requires browser interaction"
  - test: "While logged in, click the Log out button"
    expected: "User is redirected to /login and the home page no longer shows the welcome view"
    why_human: "Redirect and cleared auth state requires a live browser session"
---

# Phase 2: Auth Verification Report

**Phase Goal:** Users can securely create accounts, log in, stay logged in across sessions, and log out
**Verified:** 2026-03-12
**Status:** human_needed (all automated checks passed; runtime browser flow needs human confirmation)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can submit register form and land on a logged-in page | VERIFIED | `register()` in `handlers.rs` inserts user + returns JWT; `RegisterPage` writes JWT to localStorage, calls `validate_token`, sets auth signal, navigates to `/`; `HomePage` renders welcome + logout when `auth_user` is `Some` |
| 2 | JWT is persisted in localStorage so page refresh restores session | VERIFIED | `LoginPage` and `RegisterPage` both call `storage.set_item("jwt", &token)` on success; `App` Effect reads `localStorage.jwt` on client load and calls `validate_token` to restore `auth_user` signal |
| 3 | Wrong credentials show the same generic error regardless of whether the account exists | VERIFIED | `login()` defines `generic_err = \|\| ServerFnError::new("Invalid email or password")` and applies it identically for missing user (`user.ok_or_else(generic_err)`) and wrong password (`verified.map_err(\|_\| generic_err())`) |
| 4 | User can click logout and be redirected to /login with session cleared | VERIFIED | `on_logout` in `HomePage` calls `storage.remove_item("jwt")`, `auth_user.set(None)`, and `navigate("/login", Default::default())` |

**Score:** 4/4 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/server/auth/handlers.rs` | register, login, validate_token server functions | VERIFIED | 307 lines; all three functions fully implemented with argon2 hashing, JWT signing, and unit tests |
| `src/server/auth/mod.rs` | Auth module declaration and re-exports | VERIFIED | Declares `pub mod handlers` and re-exports with `pub use handlers::*` |
| `src/components/login_page.rs` | LoginPage component with ActionForm, localStorage write, navigate | VERIFIED | 82 lines; full implementation — ServerAction, Effect for JWT write + navigate, error display |
| `src/components/register_page.rs` | RegisterPage with confirm-password validation, same post-login flow | VERIFIED | 114 lines; client-side confirm-password guard via NodeRef + RwSignal, ActionForm, identical post-login flow |
| `src/app.rs` | App with auth context, page-load Effect, /login /register routes, logout | VERIFIED | 136 lines; provide_context(auth_user), page-load Effect with SSR guard, three routes, logout handler |
| `src/auth_user.rs` | Shared AuthUser type (id: i32, username: String) | VERIFIED | 8 lines; derives Clone, Debug, PartialEq, Serialize, Deserialize |
| `src/components/mod.rs` | Component module declarations | VERIFIED | Declares login_page and register_page modules |
| `src/lib.rs` | pub mod components declaration | VERIFIED | Declares app, auth_user, components, server modules |
| `src/main.rs` | PgPool wiring via leptos_routes_with_context | VERIFIED | Uses `leptos_routes_with_context` with `provide_context(pool.clone())` closure |
| `migrations/20240101000004_add_username_to_users.sql` | username column + unique index | VERIFIED | ALTER TABLE adds username TEXT NOT NULL; creates unique index idx_users_username |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `LoginPage` | `login()` server fn | `ServerAction::<Login>::new()` + `ActionForm` | WIRED | `ServerAction::<Login>` instantiated; `ActionForm action=login_action` submits to server |
| `LoginPage` | localStorage | `web_sys::window().local_storage()` | WIRED | `storage.set_item("jwt", &token)` in Effect on `Ok(token)` |
| `LoginPage` | auth_user signal | `use_context::<RwSignal<Option<AuthUser>>>()` | WIRED | Signal retrieved and set via `auth_user.set(Some(user))` after validate_token |
| `RegisterPage` | `register()` server fn | `ServerAction::<Register>::new()` + `ActionForm` | WIRED | Same pattern as LoginPage |
| `RegisterPage` | localStorage + auth signal | same as LoginPage | WIRED | Identical Effect block — JWT write, validate_token, auth_user set, navigate |
| `App` page-load Effect | localStorage → `validate_token` | `web_sys`, `spawn_local`, `validate_token(token).await` | WIRED | Reads `"jwt"` key, awaits `validate_token`, sets `auth_user` on `Ok` |
| `HomePage` logout | localStorage + auth signal + navigate | `remove_item("jwt")`, `auth_user.set(None)`, `navigate("/login")` | WIRED | All three steps present in `on_logout` closure |
| `main.rs` | PgPool → server functions | `leptos_routes_with_context` with `provide_context(pool.clone())` | WIRED | Server functions call `use_context::<PgPool>()` which resolves via this context |
| `App` Router | `/login`, `/register` routes | `Route path=StaticSegment("login")`, `Route path=StaticSegment("register")` | WIRED | Both routes declared in Router with correct components |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AUTH-01 | 02-01-PLAN | User can create an account with email and password | SATISFIED | `register()` server fn + `RegisterPage` component; email validation, argon2 hashing, DB insert |
| AUTH-02 | 02-01-PLAN | User can log in with email and password and receive a JWT | SATISFIED | `login()` server fn verifies argon2 hash, returns signed JWT; `LoginPage` calls it via `ServerAction` |
| AUTH-03 | 02-02-PLAN | User session persists across browser refresh | SATISFIED | JWT stored in localStorage on login/register; page-load Effect in `App` reads JWT and calls `validate_token` |
| AUTH-04 | 02-02-PLAN | User can log out and have their session invalidated | SATISFIED | `on_logout` in `HomePage` removes JWT from localStorage, clears auth signal, navigates to /login |

---

### Anti-Patterns Found

None. The grep for TODO/FIXME/HACK/placeholder found only HTML `placeholder=""` attributes on form inputs — not code stubs.

---

### Human Verification Required

All four automated-pass truths have a corresponding runtime behaviour that cannot be confirmed without a browser session. The SUMMARY documents that a human approved 9 browser test steps, but that was self-reported by the implementation agent. The following tests should be run once to confirm:

#### 1. Register → auto-login flow

**Test:** Open `/register`, submit a new email and password (8+ characters), observe the redirect.
**Expected:** Browser navigates to `/`, home page shows "Welcome, `<username>`" and a red "Log out" button.
**Why human:** React/Leptos `navigate()` after `spawn_local` + `validate_token` round-trip requires a running server and hydrated client.

#### 2. Page refresh restores session

**Test:** While on the home page logged in, press F5 or Cmd+R to hard-refresh.
**Expected:** Home page still shows the logged-in welcome view — the page-load Effect restored the session from `localStorage.jwt`.
**Why human:** localStorage read + async server call across a full page reload is a live runtime behaviour.

#### 3. Account enumeration check

**Test:** On `/login`, submit (a) a valid email with wrong password, and (b) an email address that has never been registered.
**Expected:** Both cases show the identical error message: "Invalid email or password". No timing difference or alternate wording.
**Why human:** Visual message equality across two distinct server failure paths requires browser interaction; timing side-channels cannot be measured statically.

#### 4. Logout clears session

**Test:** While logged in on the home page, click "Log out".
**Expected:** Browser navigates to `/login`; going back to `/` shows the "Not logged in" view with Login/Register links, confirming localStorage was cleared.
**Why human:** Post-logout state and navigation requires a live browser session.

---

### Gaps Summary

No gaps. All four observable truths are fully implemented and all key links are wired. The phase goal is met by the code. Status is `human_needed` only because the final browser-side confirmation of the complete auth flow is a runtime check that cannot be done statically.

---

_Verified: 2026-03-12_
_Verifier: Claude (gsd-verifier)_
