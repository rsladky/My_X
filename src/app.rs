use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
};

use crate::auth_user::AuthUser;
#[allow(unused_imports)]
use crate::server::auth::validate_token;
use crate::components::authenticated_layout::AuthenticatedLayout;
use crate::components::home_page::HomePage;
use crate::components::login_page::LoginPage;
use crate::components::profile_page::ProfilePage;
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
                    <Route path=StaticSegment("") view=|| view! { <AuthenticatedLayout><HomePage /></AuthenticatedLayout> }/>
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("register") view=RegisterPage/>
                    <Route path=ParamSegment("username") view=|| view! { <AuthenticatedLayout><ProfilePage /></AuthenticatedLayout> }/>
                </Routes>
            </main>
        </Router>
    }
}
