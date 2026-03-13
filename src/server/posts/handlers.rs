use leptos::prelude::*;

use crate::post_with_author::{PostWithAuthor, UserProfile};

#[allow(dead_code)]
fn validate_post_content(content: &str) -> Result<String, &'static str> {
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        return Err("Post cannot be empty");
    }
    if trimmed.len() > 280 {
        return Err("Post must be 280 characters or fewer");
    }
    Ok(trimmed)
}

#[leptos::server]
pub async fn create_post(content: String) -> Result<PostWithAuthor, ServerFnError> {
    use sqlx::PgPool;
    use crate::auth_user::AuthUser;

    let auth_signal = use_context::<RwSignal<Option<AuthUser>>>()
        .ok_or_else(|| ServerFnError::new("No auth context"))?;
    let user = auth_signal
        .get_untracked()
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let trimmed = validate_post_content(&content)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let pool = use_context::<PgPool>().ok_or_else(|| ServerFnError::new("Database not available"))?;

    let post = sqlx::query!(
        "INSERT INTO posts (author_id, content) VALUES ($1, $2) RETURNING id, author_id, content, created_at",
        user.id,
        trimmed
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to create post: {}", e)))?;

    Ok(PostWithAuthor {
        id: post.id,
        content: post.content,
        author_id: post.author_id,
        author_username: user.username,
        created_at: post.created_at,
    })
}

#[leptos::server]
pub async fn delete_post(post_id: i32) -> Result<(), ServerFnError> {
    use sqlx::PgPool;
    use crate::auth_user::AuthUser;

    let auth_signal = use_context::<RwSignal<Option<AuthUser>>>()
        .ok_or_else(|| ServerFnError::new("No auth context"))?;
    let user = auth_signal
        .get_untracked()
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = use_context::<PgPool>().ok_or_else(|| ServerFnError::new("Database not available"))?;

    let result = sqlx::query!(
        "DELETE FROM posts WHERE id = $1 AND author_id = $2",
        post_id,
        user.id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to delete post: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Post not found or unauthorized"));
    }

    Ok(())
}

#[leptos::server]
pub async fn list_own_posts() -> Result<Vec<PostWithAuthor>, ServerFnError> {
    use sqlx::PgPool;
    use crate::auth_user::AuthUser;

    let auth_signal = use_context::<RwSignal<Option<AuthUser>>>()
        .ok_or_else(|| ServerFnError::new("No auth context"))?;
    let user = auth_signal
        .get_untracked()
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = use_context::<PgPool>().ok_or_else(|| ServerFnError::new("Database not available"))?;

    let posts = sqlx::query!(
        r#"
        SELECT p.id, p.author_id, p.content, p.created_at, u.username AS author_username
        FROM posts p
        JOIN users u ON u.id = p.author_id
        WHERE p.author_id = $1
        ORDER BY p.created_at DESC, p.id DESC
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch posts: {}", e)))?;

    Ok(posts
        .into_iter()
        .map(|p| PostWithAuthor {
            id: p.id,
            content: p.content,
            author_id: p.author_id,
            author_username: p.author_username,
            created_at: p.created_at,
        })
        .collect())
}

#[leptos::server]
pub async fn get_user_posts(username: String) -> Result<Vec<PostWithAuthor>, ServerFnError> {
    use sqlx::PgPool;

    let pool = use_context::<PgPool>().ok_or_else(|| ServerFnError::new("Database not available"))?;

    let posts = sqlx::query!(
        r#"
        SELECT p.id, p.author_id, p.content, p.created_at, u.username AS author_username
        FROM posts p
        JOIN users u ON u.id = p.author_id
        WHERE u.username = $1
        ORDER BY p.created_at DESC, p.id DESC
        "#,
        username
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch posts: {}", e)))?;

    Ok(posts
        .into_iter()
        .map(|p| PostWithAuthor {
            id: p.id,
            content: p.content,
            author_id: p.author_id,
            author_username: p.author_username,
            created_at: p.created_at,
        })
        .collect())
}

#[leptos::server]
pub async fn get_user_profile(username: String) -> Result<UserProfile, ServerFnError> {
    use sqlx::PgPool;

    let pool = use_context::<PgPool>().ok_or_else(|| ServerFnError::new("Database not available"))?;

    let profile = sqlx::query!(
        r#"
        SELECT u.id, u.username, COUNT(p.id) AS post_count
        FROM users u
        LEFT JOIN posts p ON p.author_id = u.id
        WHERE u.username = $1
        GROUP BY u.id, u.username
        "#,
        username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch profile: {}", e)))?;

    let profile = profile.ok_or_else(|| ServerFnError::new("User not found"))?;

    Ok(UserProfile {
        id: profile.id,
        username: profile.username,
        post_count: profile.post_count.unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_post_content_empty() {
        assert_eq!(
            validate_post_content(""),
            Err("Post cannot be empty")
        );
    }

    #[test]
    fn test_validate_post_content_whitespace_only() {
        assert_eq!(
            validate_post_content("   \n\t  "),
            Err("Post cannot be empty")
        );
    }

    #[test]
    fn test_validate_post_content_too_long() {
        let long_content = "a".repeat(281);
        assert_eq!(
            validate_post_content(&long_content),
            Err("Post must be 280 characters or fewer")
        );
    }

    #[test]
    fn test_validate_post_content_exactly_280() {
        let content = "a".repeat(280);
        assert!(validate_post_content(&content).is_ok());
    }

    #[test]
    fn test_validate_post_content_trimmed() {
        let result = validate_post_content("  hello world  ");
        assert_eq!(result, Ok("hello world".to_string()));
    }
}
