use actix_web::{
    HttpResponse,
    cookie::{Cookie, time::Duration},
    web::Data,
};
use askama::Template;
use mongodb::bson::oid::ObjectId;
use redis::{AsyncCommands, aio};
use uuid::Uuid;

use crate::{
    endpoints::{
        templates::IndexTemplate,
        user_input::{BlogPost, get_all_posts},
    },
    personnel::users,
};

/// # Panics
///
/// If the cookie cannot be built, the function will panic.
pub async fn create_session(
    oid: ObjectId,
    user: &users::Users,
    mut redis_client: aio::ConnectionManager,
    mongo_client: &Data<mongodb::Client>,
) -> HttpResponse {
    tracing::info!("Generating the cookie");
    // Generate a cryptographically strong, random session ID
    let session_id = Uuid::new_v4().to_string();
    let session_key = format!("session:{session_id}");

    match redis_client
        .set_ex(&session_key, user.get_email(), 86400) // 24 hours
        .await
    {
        Ok(()) => (),
        Err(err) => {
            tracing::error!("Unable to set the session key into the cache layer: {err:#?}");
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    }

    // Build the HTTP-only, Secure cookie
    let session_cookie: Cookie = Cookie::build("session_id", session_id)
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(actix_web::cookie::SameSite::Strict)
        .max_age(Duration::seconds(86400))
        .expires(actix_web::cookie::time::OffsetDateTime::now_utc() + Duration::seconds(86400))
        .finish();

    let posts: Vec<BlogPost> = match get_all_posts(mongo_client, oid).await {
        Ok(posts) => posts,
        Err(err) => {
            tracing::error!("Unable to procure all of the posts: {err:#?}");
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    let template = IndexTemplate {
        user_email: user.get_email(),
        content: posts,
        is_logged_in: true,
        ..Default::default()
    };

    let render = template.render().expect("unable to render web page");

    tracing::info!("The session cookie to insert: {session_cookie}");

    HttpResponse::Ok().cookie(session_cookie).body(render)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use redis::{AsyncCommands, aio};
    use rstest::{fixture, rstest};

    use crate::{personnel::users, security::session::create_session, settings};

    #[fixture]
    fn get_local_redis_connection() -> redis::Client {
        redis::Client::open(settings::get().unwrap().redis.uri)
            .expect("Failed to create Redis client")
    }

    #[rstest]
    #[ignore = "Add the mongodb and a user OID"]
    #[actix_web::test]
    async fn test_create_session_sets_cookie(get_local_redis_connection: redis::Client) {
        let conn = aio::ConnectionManager::new(get_local_redis_connection)
            .await
            .expect("Failed to create Redis connection pool");

        let email = "test_email@example.com";
        let user: users::Users = users::Users::new(
            email.to_string(),
            "test_password".to_string(),
            String::new(),
        );

        let resp = create_session(&user, conn).await;

        // Check that the response has a Set-Cookie header
        let cookies = resp.cookies().collect::<Vec<_>>();
        assert_eq!(cookies.len(), 1);
        let cookie = &cookies[0];
        assert_eq!(cookie.name(), "session_id");
        assert!(cookie.http_only().unwrap());
        assert!(cookie.secure().unwrap());
        assert_eq!(cookie.path().unwrap(), "/");
    }

    #[rstest]
    #[ignore = "Add the mongodb and a user OID"]
    #[actix_web::test]
    async fn test_create_session_stores_in_redis(get_local_redis_connection: redis::Client) {
        let mut conn = aio::ConnectionManager::new(get_local_redis_connection)
            .await
            .expect("Failed to create Redis connection pool");
        let user: users::Users = users::Users::new(
            "some_email@example.com".to_string(),
            "some_password".to_string(),
            String::new(),
        );

        let session_id = create_session(&user, conn.clone())
            .await
            .cookies()
            .find(|cookie| cookie.name() == "session_id")
            .unwrap()
            .value()
            .to_string();

        let session_key = format!("session:{session_id}");
        let stored_email: String = conn.get(&session_key).await.unwrap();
        assert_eq!(stored_email, user.get_email());
    }

    #[rstest]
    #[ignore = "Add the mongodb and a user OID"]
    #[actix_web::test]
    async fn test_create_session_has_ttl(get_local_redis_connection: redis::Client) {
        // let mut conn = get_local_redis_connection.get_connection().unwrap();

        let mut conn = aio::ConnectionManager::new(get_local_redis_connection)
            .await
            .expect("Failed to create Redis connection pool");
        let user: users::Users = users::Users::new(
            "the_email@example.com".to_string(),
            "some_password".to_string(),
            String::new(),
        );

        let resp = create_session(&user, conn.clone()).await;

        let session_id = resp
            .cookies()
            .find(|cookie| cookie.name() == "session_id")
            .unwrap()
            .value()
            .to_string();

        let session_key = format!("session:{session_id}");
        let ttl: i64 = conn.ttl(&session_key).await.unwrap();

        // Check that the TTL is set (greater than 0 and less than or equal to (86400 seconds / 24 hours))
        assert!(ttl > 0 && ttl <= 86400);
    }
}
