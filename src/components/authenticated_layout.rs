use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::auth_user::AuthUser;
use super::sidebar::Sidebar;

#[component]
pub fn AuthenticatedLayout(children: Children) -> impl IntoView {
    let auth_user = use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");

    // On client, check for JWT in localStorage on mount
    Effect::new(move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            let navigate = use_navigate();
            let token = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item("jwt").ok().flatten());

            if token.is_none() {
                navigate("/login", Default::default());
            }
        }
    });

    // Redirect if auth_user becomes None (logged out)
    let navigate_clone2 = use_navigate().clone();
    Effect::new(move |_| {
        if auth_user.get().is_none() {
            navigate_clone2("/login", Default::default());
        }
    });

    view! {
        <div style="display: flex; min-height: 100vh; background: white;">
            <Sidebar />
            <main style="flex: 1; max-width: 600px; border-right: 1px solid #e1e8ed;">
                {children()}
            </main>
        </div>
    }
}
