use leptos::prelude::*;

use crate::post_with_author::{relative_timestamp, PostWithAuthor};
use crate::server::posts::DeletePost;

#[component]
fn DeleteButton(
    post_id: i32,
    #[prop(into)] on_delete: Callback<i32>,
) -> impl IntoView {
    let delete_action = ServerAction::<DeletePost>::new();

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            on_delete.run(post_id);
        }
    });

    view! {
        <button
            on:click=move |_| {
                delete_action.dispatch(DeletePost { post_id });
            }
            style="font-size: 0.85rem; color: #e0245e; background: none; border: none; cursor: pointer; padding: 0; hover:underline;"
        >
            "Delete"
        </button>
    }
}

#[component]
pub fn PostCard(
    post: PostWithAuthor,
    #[prop(default = None)] on_delete: Option<Callback<i32>>,
) -> impl IntoView {
    let avatar_color = {
        let char_code = post.author_username.chars().next().unwrap_or('a') as u32;
        let colors = vec![
            "#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8",
            "#F7DC6F", "#BB8FCE", "#85C1E2", "#F8B88B", "#85C1E2",
        ];
        colors[(char_code % colors.len() as u32) as usize]
    };

    view! {
        <div style="border-bottom: 1px solid #e1e8ed; padding: 1rem; display: flex; gap: 1rem;">
            <div
                style=format!(
                    "width: 48px; height: 48px; border-radius: 50%; background: {}; display: flex; align-items: center; justify-content: center; color: white; font-weight: bold; font-size: 1.2rem; flex-shrink: 0;",
                    avatar_color
                )
            >
                {post.author_username.chars().next().unwrap_or('?').to_uppercase().to_string()}
            </div>
            <div style="flex: 1; min-width: 0;">
                <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                    <div>
                        <a
                            href=format!("/{}", post.author_username)
                            style="color: #1a1a1a; text-decoration: none; font-weight: 600; hover:underline;"
                        >
                            "@"{post.author_username.clone()}
                        </a>
                        <span style="color: #999; margin-left: 0.5rem; font-size: 0.9rem;">
                            {relative_timestamp(post.created_at)}
                        </span>
                    </div>
                    {on_delete.map(|cb| view! { <DeleteButton post_id=post.id on_delete=cb /> })}
                </div>
                <p style="color: #1a1a1a; margin: 0.5rem 0 0 0; word-break: break-word;">
                    {post.content.clone()}
                </p>
            </div>
        </div>
    }
}
