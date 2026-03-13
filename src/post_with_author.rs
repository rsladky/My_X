use serde::{Deserialize, Serialize};

/// Represents a post with its author's username. Available on both SSR and hydrate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PostWithAuthor {
    pub id: i32,
    pub content: String,
    pub author_id: i32,
    pub author_username: String,
    pub created_at: chrono::NaiveDateTime,
}

/// Represents a user's profile information.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub post_count: i64,
}

/// Returns a relative timestamp string (e.g., "5s", "3m", "2h", "1d").
pub fn relative_timestamp(created_at: chrono::NaiveDateTime) -> String {
    let now = chrono::Utc::now().naive_utc();
    let diff = now.signed_duration_since(created_at);
    let secs = diff.num_seconds();

    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_timestamp_seconds() {
        let now = chrono::Utc::now().naive_utc();
        let past = now - chrono::Duration::seconds(30);
        assert_eq!(relative_timestamp(past), "30s");
    }

    #[test]
    fn test_relative_timestamp_minutes() {
        let now = chrono::Utc::now().naive_utc();
        let past = now - chrono::Duration::minutes(5);
        assert_eq!(relative_timestamp(past), "5m");
    }

    #[test]
    fn test_relative_timestamp_hours() {
        let now = chrono::Utc::now().naive_utc();
        let past = now - chrono::Duration::hours(3);
        assert_eq!(relative_timestamp(past), "3h");
    }

    #[test]
    fn test_relative_timestamp_days() {
        let now = chrono::Utc::now().naive_utc();
        let past = now - chrono::Duration::days(2);
        assert_eq!(relative_timestamp(past), "2d");
    }
}
