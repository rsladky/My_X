use serde::{Deserialize, Serialize};

pub use crate::auth_user::AuthUser;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

#[cfg(feature = "ssr")]
fn sign_jwt(user_id: i32, username: &str) -> Result<String, leptos::prelude::ServerFnError> {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let exp = now + 7 * 24 * 60 * 60; // 7 days

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| leptos::prelude::ServerFnError::new(format!("JWT signing failed: {}", e)))
}

#[leptos::server]
pub async fn register(
    email: String,
    password: String,
) -> Result<String, leptos::prelude::ServerFnError> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    use sqlx::PgPool;

    // Validation
    if !email.contains('@') {
        return Err(leptos::prelude::ServerFnError::new(
            "Please enter a valid email address",
        ));
    }
    if password.len() < 8 {
        return Err(leptos::prelude::ServerFnError::new(
            "Password must be at least 8 characters",
        ));
    }

    let pool = leptos::prelude::use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Database not available"))?;

    // Derive username from email local-part
    let base_username = email
        .split('@')
        .next()
        .unwrap_or(&email)
        .to_string();

    // Hash password in blocking thread (argon2 is CPU-intensive)
    let password_hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| leptos::prelude::ServerFnError::new(format!("Hash failed: {}", e)))
    })
    .await
    .map_err(|e| leptos::prelude::ServerFnError::new(format!("Task failed: {}", e)))??;

    // Try inserting with base username, retry once with suffix on unique conflict
    let username = base_username.clone();
    let result = sqlx::query!(
        "INSERT INTO users (email, username, password_hash) VALUES ($1, $2, $3) RETURNING id",
        email,
        username,
        password_hash
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(row) => sign_jwt(row.id, &username),
        Err(sqlx::Error::Database(db_err)) if db_err.constraint() == Some("users_email_key") => {
            Err(leptos::prelude::ServerFnError::new(
                "An account with this email already exists",
            ))
        }
        Err(sqlx::Error::Database(db_err))
            if db_err.constraint() == Some("idx_users_username") =>
        {
            // Retry with random suffix
            let suffix: u8 = rand::random::<u8>() % 100;
            let username_with_suffix = format!("{}{:02}", base_username, suffix);
            let row = sqlx::query!(
                "INSERT INTO users (email, username, password_hash) VALUES ($1, $2, $3) RETURNING id",
                email,
                username_with_suffix,
                password_hash
            )
            .fetch_one(&pool)
            .await
            .map_err(|e| leptos::prelude::ServerFnError::new(format!("Registration failed: {}", e)))?;
            sign_jwt(row.id, &username_with_suffix)
        }
        Err(e) => Err(leptos::prelude::ServerFnError::new(format!(
            "Registration failed: {}",
            e
        ))),
    }
}

#[leptos::server]
pub async fn login(
    email: String,
    password: String,
) -> Result<String, leptos::prelude::ServerFnError> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    use sqlx::PgPool;

    let pool = leptos::prelude::use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Database not available"))?;

    let generic_err = || leptos::prelude::ServerFnError::new("Invalid email or password");

    // Look up user by email
    let user = sqlx::query!(
        "SELECT id, username, password_hash FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| leptos::prelude::ServerFnError::new(format!("DB error: {}", e)))?;

    let user = user.ok_or_else(generic_err)?;

    let stored_hash = user.password_hash.clone();

    // Verify password in blocking thread
    let verified = tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&stored_hash).map_err(|_| ())?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| ())
    })
    .await
    .map_err(|_| generic_err())?;

    verified.map_err(|_| generic_err())?;

    sign_jwt(user.id, &user.username)
}

#[leptos::server]
pub async fn validate_token(
    token: String,
) -> Result<AuthUser, leptos::prelude::ServerFnError> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string());

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| leptos::prelude::ServerFnError::new("Invalid or expired token"))?;

    let id = token_data
        .claims
        .sub
        .parse::<i32>()
        .map_err(|_| leptos::prelude::ServerFnError::new("Invalid or expired token"))?;

    Ok(AuthUser {
        id,
        username: token_data.claims.username,
    })
}

#[cfg(test)]
mod tests {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2,
    };
    use jsonwebtoken::{
        decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
    };

    use super::Claims;

    const TEST_SECRET: &str = "test-secret-key-for-unit-tests-32+";

    fn make_claims(exp_offset_secs: i64) -> Claims {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Claims {
            sub: "42".to_string(),
            username: "alice".to_string(),
            exp: (now + exp_offset_secs) as usize,
        }
    }

    #[test]
    fn test_argon2_hash_and_verify() {
        let password = "password123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("hash failed")
            .to_string();

        // Hash must not equal plaintext
        assert_ne!(hash, password);

        // Correct password verifies
        let parsed_hash = PasswordHash::new(&hash).expect("parse hash");
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .expect("correct password should verify");

        // Wrong password does not verify
        let wrong = argon2.verify_password(b"wrongpassword", &parsed_hash);
        assert!(wrong.is_err(), "wrong password should not verify");
    }

    #[test]
    fn test_jwt_encode_decode() {
        let claims = make_claims(3600);
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .expect("encode failed");

        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(TEST_SECRET.as_bytes()),
            &Validation::default(),
        )
        .expect("decode failed");

        assert_eq!(decoded.claims.sub, "42");
        assert_eq!(decoded.claims.username, "alice");
    }

    #[test]
    fn test_jwt_expired() {
        let claims = make_claims(-3600); // expired 1 hour ago
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .expect("encode failed");

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(TEST_SECRET.as_bytes()),
            &Validation::default(),
        );

        assert!(result.is_err(), "expired token should fail decode");
        let err = result.unwrap_err();
        assert_eq!(*err.kind(), ErrorKind::ExpiredSignature);
    }

    #[test]
    fn test_jwt_invalid_string() {
        let result = decode::<Claims>(
            "not.a.jwt",
            &DecodingKey::from_secret(TEST_SECRET.as_bytes()),
            &Validation::default(),
        );
        assert!(result.is_err(), "garbage string should fail decode");
    }

    #[test]
    fn test_username_from_email() {
        let email = "alice@example.com";
        let username = email.split('@').next().unwrap_or(email).to_string();
        assert_eq!(username, "alice");

        let email_no_at = "notanemail";
        let username_fallback = email_no_at.split('@').next().unwrap_or(email_no_at).to_string();
        assert_eq!(username_fallback, "notanemail");
    }
}
