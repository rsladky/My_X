use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::auth_user::AuthUser;
use crate::post_with_author::PostWithAuthor;
use super::post_card::PostCard;

#[component]
fn ProfileHeader(profile: crate::post_with_author::UserProfile) -> impl IntoView {
    let username = profile.username.clone();
    let char_code = username.chars().next().unwrap_or('a') as u32;
    let colors = vec![
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8",
        "#F7DC6F", "#BB8FCE", "#85C1E2", "#F8B88B", "#85C1E2",
    ];
    let color = colors[char_code as usize % colors.len()];

    view! {
        <div style="border-bottom: 1px solid #e1e8ed; padding: 1rem; text-align: center;">
            <div
                style=format!(
                    "width: 80px; height: 80px; border-radius: 50%; background: {}; display: inline-flex; align-items: center; justify-content: center; color: white; font-weight: bold; font-size: 1.8rem; margin-bottom: 1rem;",
                    color
                )
            >
                {username.chars().next().unwrap_or('?').to_uppercase().to_string()}
            </div>
            <h2 style="margin: 0 0 0.5rem 0; color: #1a1a1a; font-size: 1.3rem;">
                "@"{username}
            </h2>
            <p style="color: #999; margin: 0 0 1rem 0;">
                {profile.post_count} " post"{if profile.post_count != 1 { "s" } else { "" }}
            </p>
        </div>
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();
    let auth_user = use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");

    let username = Signal::derive(move || {
        params.with(|p| p.get("username").map(|s| s.clone()).unwrap_or_default())
    });

    let profile_resource = Resource::new(
        move || username.get(),
        |u| async move {
            crate::server::posts::get_user_profile(u).await
        }
    );

    let posts_resource = Resource::new(
        move || username.get(),
        |u| async move {
            crate::server::posts::get_user_posts(u).await
        }
    );

    let posts: RwSignal<Vec<PostWithAuthor>> = RwSignal::new(vec![]);

    Effect::new(move |_| {
        if let Some(Ok(fetched_posts)) = posts_resource.get() {
            posts.set(fetched_posts);
        }
    });

    let is_own = Signal::derive(move || {
        let u_name = username.get();
        auth_user.get().map(|u| u.username == u_name).unwrap_or(false)
    });

    let on_delete_cb = Callback::new(move |post_id: i32| {
        posts.update(|p| p.retain(|x| x.id != post_id));
    });

    view! {
        <div>
            {profile_resource.get().map(|res: Result<crate::post_with_author::UserProfile, _>| {
                match res {
                    Ok(profile) => view! { <ProfileHeader profile=profile /> }.into_any(),
                    Err(e) => view! {
                        <div style="color: #d32f2f; text-align: center; padding: 1rem;">
                            {e.to_string()}
                        </div>
                    }.into_any(),
                }
            })}
            <For
                each=move || posts.get()
                key=|p| p.id
                children=move |post| {
                    let is_own_post = is_own.get() && auth_user.get().map(|u| u.id == post.author_id).unwrap_or(false);
                    let delete_cb = is_own_post.then_some(on_delete_cb);

                    view! {
                        <PostCard post=post.clone() on_delete=delete_cb />
                    }
                }
            />
        </div>
    }
}
