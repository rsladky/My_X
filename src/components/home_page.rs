use leptos::prelude::*;

use crate::auth_user::AuthUser;
use crate::post_with_author::PostWithAuthor;
use super::compose_box::ComposeBox;
use super::post_card::PostCard;

#[component]
pub fn HomePage() -> impl IntoView {
    let auth_user = use_context::<RwSignal<Option<AuthUser>>>().expect("auth context");
    let posts: RwSignal<Vec<PostWithAuthor>> = RwSignal::new(vec![]);

    let list_posts_resource = Resource::new(
        move || auth_user.get().is_some(),
        |_| async {
            crate::server::posts::list_own_posts().await
        }
    );

    Effect::new(move |_| {
        if let Some(Ok(fetched_posts)) = list_posts_resource.get() {
            posts.set(fetched_posts);
        }
    });

    let on_post_cb = Callback::new(move |new_post: PostWithAuthor| {
        posts.update(|p| p.insert(0, new_post));
    });

    let on_delete_cb = Callback::new(move |post_id: i32| {
        posts.update(|p| p.retain(|x| x.id != post_id));
    });

    view! {
        <ComposeBox on_post=on_post_cb />
        <div>
            <For
                each=move || posts.get()
                key=|p| p.id
                children=move |post| {
                    let is_own = auth_user.get()
                        .map(|u| u.id == post.author_id)
                        .unwrap_or(false);
                    let delete_cb = is_own.then_some(on_delete_cb);

                    view! {
                        <PostCard post=post.clone() on_delete=delete_cb />
                    }
                }
            />
        </div>
    }
}
