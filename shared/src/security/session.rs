// use actix_web::{
//     HttpResponse,
//     cookie::{Cookie, time::Duration},
//     web::Data,
// };
// use mongodb::bson::oid::ObjectId;
// use redis::{AsyncCommands, aio};
// use uuid::Uuid;

// use crate::{
//     endpoints::{
//         templates::IndexTemplate,
//         user_input::{BlogPost, get_all_posts},
//     },
//     personnel::users,
// };

// /// # Panics
// ///
// /// If the cookie cannot be built, the function will panic.
// pub async fn create_session(
//     oid: ObjectId,
//     user: &users::Users,
//     mut redis_client: aio::ConnectionManager,
//     mongo_client: &Data<mongodb::Client>,
// ) -> HttpResponse {
//     tracing::info!("Generating the cookie");
//     // Generate a cryptographically strong, random session ID
//     let session_id = Uuid::new_v4().to_string();
//     let session_key = format!("session:{session_id}");

//     match redis_client
//         .set_ex(&session_key, user.get_email(), 86400) // 24 hours
//         .await
//     {
//         Ok(()) => (),
//         Err(err) => {
//             tracing::error!("Unable to set the session key into the cache layer: {err:#?}");
//             return HttpResponse::InternalServerError().body(err.to_string());
//         }
//     }

//     // Build the HTTP-only, Secure cookie
//     let session_cookie: Cookie = Cookie::build("session_id", session_id)
//         .path("/")
//         .http_only(true)
//         .secure(true)
//         .same_site(actix_web::cookie::SameSite::Strict)
//         .max_age(Duration::seconds(86400))
//         .expires(actix_web::cookie::time::OffsetDateTime::now_utc() + Duration::seconds(86400))
//         .finish();

//     let posts: Vec<BlogPost> = match get_all_posts(mongo_client, oid).await {
//         Ok(posts) => posts,
//         Err(err) => {
//             tracing::error!("Unable to procure all of the posts: {err:#?}");
//             return HttpResponse::InternalServerError().body(err.to_string());
//         }
//     };

//     let template = IndexTemplate {
//         user_email: user.get_email(),
//         content: posts,
//         is_logged_in: true,
//         ..Default::default()
//     };

//     let render = template.render().expect("unable to render web page");

//     tracing::info!("The session cookie to insert: {session_cookie}");

//     HttpResponse::Ok().cookie(session_cookie).body(render)
// }

// #[cfg(test)]
// mod tests {}
