use mongodb::bson::oid::ObjectId;
use redis::{AsyncCommands, aio};
use serde::{Deserialize, Serialize};

use crate::mongo_conf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserSession {
    pub oid: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuthenticationError {
    MissingSession,
    InvalidSession,
    Redis,
}

fn session_key(session_cookie: &str) -> String {
    format!("session:{session_cookie}")
}

fn user_auth_key(email: &str) -> String {
    format!("user:auth:{email}")
}

fn parse_cached_user_session(value: &str) -> Result<ObjectId, AuthenticationError> {
    let session: UserSession =
        serde_json::from_str(value).map_err(|_| AuthenticationError::InvalidSession)?;

    ObjectId::parse_str(&session.oid).map_err(|_| AuthenticationError::InvalidSession)
}

/// # Errors
///
///    - Will Error if the session key is missing returning a `Missingsession`
///    - Will Error if the session is invalid returning a `Invalidsession`
///    - Will Error if the user is not `Authenticated`
pub async fn authenticated_user_id(
    session_cookie: &str,
    mongo_client: &mongodb::Client,
    redis_client: &mut aio::ConnectionManager,
) -> Result<ObjectId, AuthenticationError> {
    let user_session = session_key(session_cookie);

    let user_email: Option<String> = redis_client
        .get(&user_session)
        .await
        .map_err(|_| AuthenticationError::Redis)?;

    let user_email = user_email.ok_or(AuthenticationError::MissingSession)?;

    let cache_key = user_auth_key(&user_email);

    let cached_user: Option<String> = redis_client
        .get(&cache_key)
        .await
        .map_err(|_| AuthenticationError::Redis)?;

    if let Some(session_json) = cached_user {
        parse_cached_user_session(&session_json)
    } else {
        let mongo_database = mongo_conf::establish_connection(mongo_client)
            .await
            .map_err(|_| AuthenticationError::InvalidSession)?;

        let filter = mongodb::bson::doc! {
            "email": &user_email
        };

        let user_doc = mongo_database
            .collection::<mongodb::bson::Document>("development")
            .find_one(filter)
            .await
            .map_err(|_| AuthenticationError::InvalidSession)?;

        let user_doc = user_doc.ok_or(AuthenticationError::MissingSession)?;

        user_doc
            .get_object_id("_id")
            .map_err(|_| AuthenticationError::InvalidSession)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    use mongodb::bson::oid::ObjectId;
    use rstest::rstest;

    #[rstest]
    #[case("abc123", "session:abc123")]
    #[case("cookie_string", "session:cookie_string")]
    #[case("", "session:")]
    #[case("some-long-session-value", "session:some-long-session-value")]
    fn session_key_is_formatted_correctly(#[case] session_id: &str, #[case] expected: &str) {
        assert_eq!(session_key(session_id), expected);
    }

    #[rstest]
    #[case("user@example.com", "user:auth:user@example.com")]
    #[case("test@podcasters.com", "user:auth:test@podcasters.com")]
    #[case("", "user:auth:")]
    fn user_auth_key_is_formatted_correctly(#[case] email: &str, #[case] expected: &str) {
        assert_eq!(user_auth_key(email), expected);
    }

    #[rstest]
    #[case("507f1f77bcf86cd799439011")]
    #[case("64b64c66b07f9f641a5d1832")]
    fn cached_user_session_parses_valid_object_id(#[case] oid: &str) {
        let json = format!(r#"{{"oid":"{oid}"}}"#);

        let result = parse_cached_user_session(&json).unwrap();

        assert_eq!(result.to_hex(), oid);
    }

    #[rstest]
    #[case("")]
    #[case("{}")]
    #[case(r#"{"oid":""}"#)]
    #[case(r#"{"oid":"not-an-object-id"}"#)]
    #[case(r#"{"id":"507f1f77bcf86cd799439011"}"#)]
    #[case("this is not JSON")]
    fn cached_user_session_rejects_invalid_data(#[case] input: &str) {
        let result = parse_cached_user_session(input);

        assert_eq!(result, Err(AuthenticationError::InvalidSession));
    }

    #[test]
    fn user_session_serialization_round_trip() {
        let object_id = ObjectId::new();

        let original = UserSession {
            oid: object_id.to_hex(),
        };

        let serialized = serde_json::to_string(&original).unwrap();

        let deserialized: UserSession = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}
