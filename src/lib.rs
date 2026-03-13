#![recursion_limit = "1024"]

pub mod app;
pub mod auth_user;
pub mod components;
pub mod post_with_author;
pub mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
