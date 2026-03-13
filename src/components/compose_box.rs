use leptos::prelude::*;

use crate::post_with_author::PostWithAuthor;
use crate::server::posts::CreatePost;

#[component]
pub fn ComposeBox(
    #[prop(into)] on_post: Callback<PostWithAuthor>,
) -> impl IntoView {
    let content = RwSignal::new(String::new());
    let create_action = ServerAction::<CreatePost>::new();
    let value = create_action.value();

    let char_count = Signal::derive(move || content.get().len());
    let is_over = Signal::derive(move || char_count.get() > 280);
    let is_empty = Signal::derive(move || content.get().trim().is_empty());

    Effect::new(move |_| {
        if let Some(Ok(post)) = value.get() {
            on_post.run(post);
            content.set(String::new());
        }
    });

    view! {
        <div style="border-bottom: 1px solid #e1e8ed; padding: 1rem;">
            <textarea
                prop:value=move || content.get()
                on:input=move |ev| content.set(event_target_value(&ev))
                placeholder="What's happening?!"
                style="width: 100%; min-height: 100px; padding: 1rem; font-size: 1.2rem; border: none; resize: none; font-family: inherit;"
            />
            <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 1rem;">
                <span
                    style=move || {
                        if is_over.get() {
                            "color: #e0245e; font-size: 0.9rem; font-weight: 500;"
                        } else {
                            "color: #999; font-size: 0.9rem;"
                        }
                    }
                >
                    {move || format!("{}/{}", char_count.get(), 280)}
                </span>
                <form
                    on:submit=move |ev| {
                        ev.prevent_default();
                        create_action.dispatch(CreatePost {
                            content: content.get(),
                        });
                    }
                >
                    <button
                        type="submit"
                        disabled=move || is_empty.get() || is_over.get()
                        style="padding: 0.6rem 1.5rem; background: #1d9bf0; color: white; border: none; border-radius: 20px; font-size: 1rem; cursor: pointer; font-weight: 600; disabled:opacity 0.5; disabled:cursor not-allowed;"
                    >
                        "Post"
                    </button>
                </form>
            </div>
        </div>
    }
}
