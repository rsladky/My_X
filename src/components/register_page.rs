use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::auth_user::AuthUser;
use crate::server::auth::handlers::{validate_token, Register};

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_action = ServerAction::<Register>::new();
    let value = register_action.value();
    let auth_user =
        use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");
    let navigate = use_navigate();

    let confirm_password = RwSignal::new(String::new());
    let client_error = RwSignal::new(Option::<String>::None);

    let password_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if let Some(Ok(token)) = value.get() {
            // Store JWT in localStorage (client-side only)
            #[cfg(not(feature = "ssr"))]
            {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = storage.set_item("jwt", &token);
                }
            }

            let navigate = navigate.clone();
            leptos::task::spawn_local(async move {
                if let Ok(user) = validate_token(token).await {
                    auth_user.set(Some(user));
                }
                navigate("/", Default::default());
            });
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        let password_val = password_ref
            .get()
            .map(|el| el.value())
            .unwrap_or_default();
        let confirm_val = confirm_password.get();
        if password_val != confirm_val {
            ev.prevent_default();
            client_error.set(Some("Passwords do not match".to_string()));
        } else {
            client_error.set(None);
        }
    };

    view! {
        <div style="display: flex; justify-content: center; align-items: center; min-height: 100vh; background: #f5f5f5;">
            <div style="background: white; padding: 2rem; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); width: 100%; max-width: 400px;">
                <h1 style="margin: 0 0 1.5rem; font-size: 1.5rem; text-align: center; color: #1a1a1a;">"Create your account"</h1>

                {move || value.get().and_then(|v| v.err()).map(|e| view! {
                    <p style="color: #d32f2f; background: #ffebee; padding: 0.75rem; border-radius: 4px; margin-bottom: 1rem; font-size: 0.9rem;">{e.to_string()}</p>
                })}

                {move || client_error.get().map(|e| view! {
                    <p style="color: #d32f2f; background: #ffebee; padding: 0.75rem; border-radius: 4px; margin-bottom: 1rem; font-size: 0.9rem;">{e}</p>
                })}

                <ActionForm action=register_action on:submit=on_submit>
                    <div style="margin-bottom: 1rem;">
                        <label style="display: block; margin-bottom: 0.25rem; font-size: 0.9rem; color: #555;">"Email"</label>
                        <input
                            type="email"
                            name="email"
                            placeholder="you@example.com"
                            required
                            style="width: 100%; padding: 0.6rem 0.75rem; border: 1px solid #ddd; border-radius: 4px; font-size: 1rem; box-sizing: border-box;"
                        />
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <label style="display: block; margin-bottom: 0.25rem; font-size: 0.9rem; color: #555;">"Password"</label>
                        <input
                            type="password"
                            name="password"
                            placeholder="At least 8 characters"
                            required
                            node_ref=password_ref
                            style="width: 100%; padding: 0.6rem 0.75rem; border: 1px solid #ddd; border-radius: 4px; font-size: 1rem; box-sizing: border-box;"
                        />
                    </div>
                    <div style="margin-bottom: 1.5rem;">
                        <label style="display: block; margin-bottom: 0.25rem; font-size: 0.9rem; color: #555;">"Confirm password"</label>
                        <input
                            type="password"
                            placeholder="Repeat your password"
                            on:input=move |ev| confirm_password.set(event_target_value(&ev))
                            style="width: 100%; padding: 0.6rem 0.75rem; border: 1px solid #ddd; border-radius: 4px; font-size: 1rem; box-sizing: border-box;"
                        />
                    </div>
                    <button
                        type="submit"
                        style="width: 100%; padding: 0.75rem; background: #1d9bf0; color: white; border: none; border-radius: 4px; font-size: 1rem; cursor: pointer; font-weight: 600;"
                    >
                        "Create account"
                    </button>
                </ActionForm>

                <p style="text-align: center; margin-top: 1rem; font-size: 0.9rem; color: #555;">
                    <a href="/login" style="color: #1d9bf0; text-decoration: none;">"Already have an account? Log in"</a>
                </p>
            </div>
        </div>
    }
}
