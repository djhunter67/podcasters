use std::task::Poll;

use crate::{
    endpoints::{
        templates::IndivInput,
        user_input::{self, BlogPost},
    },
    models::redis_conf::authenticated_user_id,
    personnel::users,
};

use super::templates::IndexTemplate;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, get,
    http::{
        self, StatusCode,
        header::{ContentEncoding, ContentType},
    },
    web::{self, Data},
};
use askama::Template;
use futures::stream;
use redis::{AsyncCommands, aio};
use tracing::instrument;

#[allow(clippy::future_not_send)]
#[instrument(
    name = "Serving main page",
    level = "debug",
    target = "web_app_bloodhound",
    fields(samples = 25, title = "Home"),
    skip(redis_client, req, mongo_client)
)]
#[get("/")]
pub async fn index(
    req: HttpRequest,
    redis_client: Data<aio::ConnectionManager>,
    mongo_client: Data<mongodb::Client>,
) -> HttpResponse {
    tracing::info!("Serving main page");

    tracing::debug!("The req cookies found: {:#?}", req.cookies());

    let session_id = if let Some(cookie) = req.cookie("session_id") {
        cookie.value().to_string()
    } else {
        tracing::error!("User cookie not found: {:#?}", req.connection_info());

        let var_name = IndexTemplate::new(vec![], "Login to create a journal entry.", false);

        let rendered = var_name.render().expect("Failed to render template");

        return HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(rendered);
    };

    let oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("Unable to validate the user: {err:#?}");
            // return HttpResponse::InternalServerError().json(format!("{err:#?}"));
            let default_template = IndexTemplate {
                user_email: String::from("Please login to create a journal entry"),
                ..Default::default()
            };
            let rendered = match default_template.render() {
                Ok(template) => template,
                Err(err) => return HttpResponse::InternalServerError().json(format!("{err:#?}")),
            };
            return HttpResponse::Ok().body(rendered);
        }
    };

    tracing::info!("Creating the cache-layer session key");
    let session_key = format!("session:{session_id}");

    tracing::info!("Searching for the session key: {session_key}");
    let user: Option<String> = match redis_client.as_ref().clone().get(&session_key).await {
        Ok(result) => Some(result),
        Err(err) => {
            tracing::error!("Error accessing the cache layer: {err:#?}");
            return HttpResponse::InternalServerError()
                .json(format!("Unable to acquire the cache layer: {err:#?}"));
        }
    };

    tracing::warn!("The session id: {session_key}");
    let blog_post: Vec<BlogPost> = match user_input::get_all_posts(&mongo_client, oid).await {
        Ok(posts) => posts,
        Err(err) => {
            tracing::error!("Unable to procure the users posts: {err:#?}");
            let default_index = IndexTemplate {
                user_email: format!("Unable to procure your posts: {err:#?}"),
                is_logged_in: true,
                ..Default::default()
            };

            return HttpResponse::Ok().body(default_index.render().unwrap_or_default());
        }
    };

    let user: &str = if let Some(user_data) = &user {
        user_data
    } else {
        "No user data found"
    };

    let var_name = IndexTemplate::new(blog_post, user, true);

    let rendered = var_name.render().expect("Failed to render template");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered)
}

#[get("/create_post")]
#[instrument(
    name = "Serving main page",
    level = "debug",
    target = "web_app_bloodhound",
    fields(samples = 25, title = "Home"),
    skip(_redis_client, _mongo_client)
)]
pub async fn create_post(
    _mongo_client: Data<mongodb::Client>,
    _redis_client: Data<aio::ConnectionManager>,
) -> HttpResponse {
    let blog_post: users::Users = users::Users::default();
    let post_creator: IndivInput = IndivInput::new(blog_post);

    let rendered = match post_creator.render() {
        Ok(rend) => rend,
        Err(err) => {
            return HttpResponse::Ok().body(format!("Unable to render the index page: {err:#?}"));
        }
    };
    HttpResponse::Ok().body(rendered)
}

#[allow(clippy::future_not_send)]
pub async fn sse(_req: HttpRequest) -> impl Responder {
    let mut counter: usize = 5;

    // yeilds `data N` whrere N in [5; 1]
    let server_events = stream::poll_fn(move |_cx| -> Poll<Option<Result<web::Bytes, Error>>> {
        if counter == 0 {
            return Poll::Ready(None);
        }
        let payload = format!("data: {counter}\n\n");
        counter -= 1;
        Poll::Ready(Some(Ok(web::Bytes::from(payload))))
    });

    HttpResponse::build(StatusCode::OK)
        .insert_header((http::header::CONTENT_TYPE, "text/event-stream"))
        .insert_header(ContentEncoding::Identity)
        .streaming(server_events)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::pin::pin;

    use actix_web::{
        App,
        body::{self, MessageBody},
        test,
        web::{self, Bytes},
    };
    use futures::future;

    use super::{index, sse};

    #[actix_web::test]
    #[ignore = "known to fail in this stage of development"]
    async fn test_get_index() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    #[ignore = "known to fail during this stage of development"]
    async fn test_index_is_html() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_and_read_body(&app, req).await;
        assert!(!resp.is_empty());
        let first_letters: Bytes = resp.slice(0..15).iter().copied().collect();
        let conv_str = std::str::from_utf8(&first_letters).unwrap();
        assert_eq!(conv_str, "<!DOCTYPE html>");
    }

    #[actix_web::test]
    async fn test_stream_chunk() {
        let app = test::init_service(App::new().route("/sse", web::get().to(sse))).await;
        let req = test::TestRequest::get().uri("/sse").to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = resp.into_body();
        let mut body = pin!(body);

        // first chunk
        let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
        println!("byte 1: {bytes:#?}");

        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 5\n\n")
        );

        // Second chunk
        let bytes = future::poll_fn(|cx| body.as_pin_mut().poll_next(cx)).await;
        println!("byte 2: {bytes:#?}");
        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 4\n\n")
        );

        // Remaining part
        for i in 0..3 {
            let expected_data = format!("data: {}\n\n", 3 - i);
            let bytes = future::poll_fn(|cx| body.as_pin_mut().poll_next(cx)).await;
            println!("rem bytes: {bytes:#?}");
            assert_eq!(bytes.unwrap().unwrap(), web::Bytes::from(expected_data));
        }
    }

    #[actix_web::test]
    async fn test_stream_full_payload() {
        let app = test::init_service(App::new().route("/sse", web::get().to(sse))).await;
        let req = test::TestRequest::get().uri("/sse").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = resp.into_body();
        let bytes = body::to_bytes(body).await;
        assert_eq!(
            bytes.unwrap(),
            web::Bytes::from_static(b"data: 5\n\ndata: 4\n\ndata: 3\n\ndata: 2\n\ndata: 1\n\n")
        );
    }
}
