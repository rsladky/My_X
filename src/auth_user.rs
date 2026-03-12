use serde::{Deserialize, Serialize};

/// Represents an authenticated user. Available on both SSR and hydrate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
}
