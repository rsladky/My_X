use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_navigate,
    StaticSegment,
};

use crate::auth_user::AuthUser;
#[allow(unused_imports)]
use crate::server::auth::validate_token;
use crate::components::login_page::LoginPage;
use crate::components::register_page::RegisterPage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Auth state signal — provided to all child components via context
    let auth_user: RwSignal<Option<AuthUser>> = RwSignal::new(None);
    provide_context(auth_user);

    // Page-load JWT validation: read localStorage on client, validate with server
    Effect::new(move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            let token = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item("jwt").ok().flatten());
            if let Some(token) = token {
                leptos::task::spawn_local(async move {
                    if let Ok(user) = validate_token(token).await {
                        auth_user.set(Some(user));
                    }
                });
            }
        }
    });

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

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let auth_user =
        use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");
    let navigate = use_navigate();

    view! {
        <div style="display: flex; justify-content: center; align-items: center; min-height: 100vh; background: #f5f5f5;">
            <div style="background: white; padding: 2rem; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); width: 100%; max-width: 480px; text-align: center;">
                {move || {
                    let navigate = navigate.clone();
                    let on_logout = move |_: leptos::ev::MouseEvent| {
                        // Clear JWT from localStorage (client-side only)
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
                    match auth_user.get() {
                        Some(user) => view! {
                            <div>
                                <h1 style="margin: 0 0 0.5rem; color: #1a1a1a; font-size: 1.5rem;">
                                    "Welcome, " {user.username}
                                </h1>
                                <p style="color: #555; margin-bottom: 1.5rem;">"You are logged in."</p>
                                <button
                                    on:click=on_logout
                                    style="padding: 0.6rem 1.5rem; background: #e0245e; color: white; border: none; border-radius: 4px; font-size: 1rem; cursor: pointer; font-weight: 600;"
                                >
                                    "Log out"
                                </button>
                            </div>
                        }.into_any(),
                        None => view! {
                            <div>
                                <h1 style="margin: 0 0 0.5rem; color: #1a1a1a; font-size: 1.5rem;">"My X"</h1>
                                <p style="color: #555; margin-bottom: 1.5rem;">"Not logged in."</p>
                                <div style="display: flex; gap: 0.75rem; justify-content: center;">
                                    <a
                                        href="/login"
                                        style="padding: 0.6rem 1.5rem; background: #1d9bf0; color: white; border-radius: 4px; text-decoration: none; font-size: 1rem; font-weight: 600;"
                                    >"Log in"</a>
                                    <a
                                        href="/register"
                                        style="padding: 0.6rem 1.5rem; background: white; color: #1d9bf0; border: 2px solid #1d9bf0; border-radius: 4px; text-decoration: none; font-size: 1rem; font-weight: 600;"
                                    >"Register"</a>
                                </div>
                            </div>
                        }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}
