use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::auth_user::AuthUser;

#[component]
pub fn Sidebar() -> impl IntoView {
    let auth_user = use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");
    let navigate = use_navigate();

    let username = Signal::derive(move || {
        auth_user.get().map(|u| u.username)
    });

    view! {
        <aside style="width: 250px; background: white; border-right: 1px solid #e1e8ed; padding: 1rem; display: flex; flex-direction: column; min-height: 100vh;">
            <h1 style="font-size: 1.5rem; margin: 0 0 2rem 0; color: #1d9bf0;">"My X"</h1>

            <nav style="flex: 1;">
                <ul style="list-style: none; padding: 0; margin: 0;">
                    <li style="margin-bottom: 1rem;">
                        <a
                            href="/"
                            style="color: #1a1a1a; text-decoration: none; display: block; padding: 0.75rem; border-radius: 20px; hover:background #f5f5f5; font-size: 1.1rem; font-weight: 500;"
                        >
                            "🏠 Home"
                        </a>
                    </li>
                    {username.get().map(|username| {
                        view! {
                            <li style="margin-bottom: 1rem;">
                                <a
                                    href=move || format!("/{}", username)
                                    style="color: #1a1a1a; text-decoration: none; display: block; padding: 0.75rem; border-radius: 20px; hover:background #f5f5f5; font-size: 1.1rem; font-weight: 500;"
                                >
                                    "👤 Profile"
                                </a>
                            </li>
                        }
                    })}
                </ul>
            </nav>

            <button
                on:click=move |_| {
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
                }
                style="width: 100%; padding: 0.75rem; background: #e0245e; color: white; border: none; border-radius: 20px; font-size: 1rem; cursor: pointer; font-weight: 600;"
            >
                "Log out"
            </button>
        </aside>
    }
}
