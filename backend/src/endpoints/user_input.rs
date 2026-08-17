pub(crate) use std::{fmt::Display, fs::File};

use actix_multipart::form::MultipartForm;
use actix_web::{
    HttpRequest, HttpResponse, delete, post,
    web::{self, Data, Form},
};
use askama::Template;
use futures::StreamExt;
use mongodb::{
    bson::{DateTime as BsonDateTime, doc},
    options,
};
use redis::{AsyncCommands, aio};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    endpoints::templates::{IndexTemplate, IndivPost, JournalPostEditor},
    images::ImageUpload,
    models::{
        mongo::{self, JournalDraft},
        redis_conf::authenticated_user_id,
    },
};

type Oid = mongodb::bson::oid::ObjectId;

/// # TODO
///
/// Editing of posts
/// Automatic draft saving
/// Autosave interval
/// Categories of posts
/// Show word count
/// Journaling landing page
/// Sort order of posts
/// Entries per page and pagination
/// Confirmation before deletion && deletion
/// Trash retention period
/// Time zone metadata per post
/// Export journal entries to Markdown or JSON
/// Download all images and journal entries
/// Restore recently deleted posts
/// View the data the application stores
#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    title: String,
    body: String,
    author: String,
    #[serde(default)]
    user_id: String,
    #[serde(rename = "_id")]
    post_id: Option<Oid>,
    #[serde(default = "default_date")]
    date: BsonDateTime,
    #[serde(default)]
    logged_in: bool,
}

fn default_date() -> BsonDateTime {
    BsonDateTime::from_system_time(chrono::Utc::now().into())
}

impl BlogPost {
    #[must_use]
    pub fn to_name() -> String {
        String::from("BlogPosts")
    }
    #[must_use]
    pub fn new(
        title: String,
        body: String,
        author: String,
        user_id: String,
        post_id: Option<Oid>,
        logged_in: bool,
    ) -> Self {
        Self {
            title,
            body,
            author,
            user_id,
            post_id,
            date: BsonDateTime::from_system_time(chrono::Utc::now().into()),
            logged_in,
        }
    }

    #[must_use]
    pub fn get_title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn get_body(&self) -> &str {
        &self.body
    }

    #[must_use]
    pub fn get_author(&self) -> &str {
        &self.author
    }

    #[must_use]
    /// Get the date in a user readable format
    /// # Error
    ///
    /// - If the date is not in a valid format, it will return an empty string
    /// # Panics
    ///
    /// - If the time is not in a valid format, it will panic
    /// - if the format passed in is not a valid ``BsonDatetime`` instance
    pub fn get_date(&self) -> String {
        // Make the date user readable in the following format: YYYY-MM-DD HH:MM:SS where the HH is in 24 hour format and the time zone is EST and the MM are the 3 letter day in uppercase
        // let unreadable_date = &self.date.to_string();

        let unreadable_date = &self.date.to_string();
        let (month_date_year, time) = unreadable_date.split_once(' ').unwrap_or(("", ""));

        let month_and_day = month_date_year.split_once('-').map_or("", |(_, m)| m);

        let mut month = month_and_day.split('-').next().unwrap_or("");

        let month_map = [
            ("01", "JAN"),
            ("02", "FEB"),
            ("03", "MAR"),
            ("04", "APR"),
            ("05", "MAY"),
            ("06", "JUN"),
            ("07", "JUL"),
            ("08", "AUG"),
            ("09", "SEP"),
            ("10", "OCT"),
            ("11", "NOV"),
            ("12", "DEC"),
        ];

        for (m, m_str) in &month_map {
            if month == *m {
                // tracing::info!("Month: {}", m_str);
                month = m_str;
                break;
            }
        }

        // tracing::info!("Month: {}", month);

        let year = month_date_year.split_once('-').map_or("", |(y, _)| y);
        // tracing::info!("Year: {}", year);
        let day = month_date_year.split_once('-').map_or("", |(_, d)| {
            d.split('-')
                .next_back()
                .expect("day timestamp parsing issue")
        });
        // tracing::info!("Day: {}", day);

        let mut time = time
            .split_once('.')
            .map_or("", |(t, _)| t)
            .rsplit_once(':')
            .map_or_else(|| time.to_string(), |(h, _m)| format!("{h}HRS"));

        // tracing::info!("Zulu Time: {}", time);

        // reduce the time by four hours to account for EST time zone
        match time.split_once('H').unwrap_or_default().0.split_once(':') {
            Some((h, m)) if let Ok(h) = h.parse::<i32>() => {
                let h = h - 4; // (h - 4).rem_euclid(24);
                time = format!("{h}:{m}HRS");
            }
            Some(_) => todo!(),
            None => todo!(),
        }

        // tracing::info!("EST Time: {}", time);

        format!("{year}-{month}-{day}   {time} EST")
    }

    #[must_use]
    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn set_user_id(&mut self, user_id: String) {
        self.user_id = user_id;
    }

    pub const fn toggle_logged_in(&mut self) {
        self.logged_in = !self.logged_in;
    }

    #[must_use]
    pub const fn is_logged_in(&self) -> bool {
        self.logged_in
    }

    pub const fn set_post_id(&mut self, post_id: Option<Oid>) {
        self.post_id = post_id;
    }

    #[must_use]
    pub fn get_post_id(&self) -> Oid {
        self.post_id.unwrap_or_default()
    }
}

impl Display for BlogPost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Title: {}, Author: {}, Date: {}, Logged In: {}",
            self.title, self.author, self.date, self.logged_in
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct JournalDraftInput {
    draft_id: Option<String>,
    title: String,
    body: String,
    author: String,
}

impl JournalDraftInput {
    #[must_use]
    pub const fn new(
        draft_id: Option<String>,
        title: String,
        body: String,
        author: String,
    ) -> Self {
        Self {
            draft_id,
            title,
            body,
            author,
        }
    }

    #[must_use]
    pub const fn get_draft_id(&self) -> Option<&String> {
        self.draft_id.as_ref()
    }

    #[must_use]
    pub fn get_title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn get_body(&self) -> &str {
        &self.body
    }

    #[must_use]
    pub fn get_author(&self) -> &str {
        &self.author
    }

    pub fn set_draft_id(&mut self, draft_id: Option<String>) {
        self.draft_id = draft_id;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    pub fn set_author(&mut self, author: String) {
        self.author = author;
    }

    pub fn clear(&mut self) {
        self.draft_id = None;
        self.title.clear();
        self.body.clear();
        self.author.clear();
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.title.is_empty() && self.body.is_empty() && self.author.is_empty()
    }
}

impl Display for JournalDraftInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Draft ID: {:?}, Title: {}, Body: {}, Author: {}",
            self.draft_id, self.title, self.body, self.author
        )
    }
}

#[allow(clippy::future_not_send)]
#[instrument(
    name = "User submits text",
    level = "info",
    target = "Personal journal",
    skip(req, mongo_client, redis_client, input)
)]
#[post("/submit_text")]
pub async fn submit_text(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    Form(mut input): web::Form<BlogPost>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Submit text endpoint");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    tracing::info!("The data passed from the form: {input:#?}");

    input.toggle_logged_in();
    input.set_user_id(user_oid.to_string());
    input.set_post_id(Some(Oid::new()));

    let journal_entries = match mongo::establish_connection(&mongo_client).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(?err, "Unable to procure a db connection");
            return HttpResponse::InternalServerError().finish();
        }
    }
    .collection::<BlogPost>("BlogPosts");

    tracing::warn!("Reusing the passed in  BlogPost instance: {input:#?}");

    match journal_entries.insert_one(&input).await {
        Ok(oid) => {
            tracing::warn!("The insert result: {oid:#?}");
            // tracing::info!("Inserting a new journal entry");
            // if let Err(err) = drafts
            //     .delete_one(doc! {
            //     "_id": user_oid
            //     })
            //     .await
            // {
            //     tracing::error!(
            //         ?err,
            //         %user_oid,
            //         "Entry published but draft cleanup failed"
            //     );
            // }

            // All business for saving the post is done, now return a result
            // let oid: Oid = oid.inserted_id.as_object_id().map_or_else(
            //     || {
            //         tracing::error!("Unable to convert the post oid to a string");
            //         Oid::from_str("no joy")
            //             .expect("failed to create oid upon failure to procure oid")
            //     },
            //     |str_oid| {
            //         tracing::info!("Oid string found: {str_oid:#?}");
            //         str_oid
            //     },
            // );

            // input.post_id = oid;
            input.set_post_id(oid.inserted_id.as_object_id());

            tracing::warn!("The post id has been set: {input:#?}");
            let blog_template = IndivPost::new(input);

            let render = match blog_template.render() {
                Ok(html) => html,
                Err(err) => {
                    tracing::error!(
                    ?err,
                    %user_oid,
                    "Entry published but rendering failed"
                    );
                    return HttpResponse::InternalServerError().body(
                        "Entry published but rendering failed. Your saved draft remains available.",
                    );
                }
            };

            HttpResponse::Ok().body(render)
        }
        Err(err) => {
            tracing::error!(
            ?err,
            %user_oid,
            "Unable to publish journal entry"
            );

            HttpResponse::InternalServerError()
                .body("Publishing failed. Yor saved draft remains available.")
        }
    }
}

/// Endpoint to edit a submission. This endpoint retrieves the latest journal entry for the authenticated user and renders it in an editable form.
/// # Errors
///
/// - Returns `HttpResponse::Unauthorized` if the user is not authenticated.
/// - Returns `HttpResponse::InternalServerError` if there is an issue connecting to the database or retrieving the journal entry.
/// - Returns `HttpResponse::NotFound` if no journal entry is found for the authenticated user.
#[allow(clippy::future_not_send)]
#[instrument(
    name = "User edits text",
    level = "info",
    target = "Edit the Blog Post",
    skip(req, mongo_client, redis_client, input)
)]
#[post("/edit_submission")]
pub async fn edit_submission(
    redis_client: Data<aio::ConnectionManager>,
    mongo_client: Data<mongodb::Client>,
    web::Form(mut input): web::Form<BlogPost>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Edit submission endpoint");

    tracing::warn!("The blogpost to show: {input:#?}");

    // let trim_newlines = input.get_body().trim()

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    input.toggle_logged_in();
    input.set_user_id(user_oid.to_string());

    let edit_template = IndivPost::new(input);

    let render = match edit_template.render() {
        Ok(html) => html,
        Err(err) => {
            tracing::error!(
                ?err,
                %user_oid,
                "Entry retrieved but rendering failed"
            );
            return HttpResponse::InternalServerError()
                .body("Entry retrieved but rendering failed.");
        }
    };

    HttpResponse::Ok().body(render)
}

/// Endpoint editor for  a submission. This endpoint retrieves the latest journal entry for the authenticated user and renders it to be editable.
/// # Errors
///
/// - Returns `HttpResponse::Unauthorized` if the user is not authenticated.
/// - Returns `HttpResponse::InternalServerError` if there is an issue connecting to the database or retrieving the journal entry.
/// - Returns `HttpResponse::NotFound` if no journal entry is found for the authenticated user.
#[allow(clippy::future_not_send)]
#[instrument(
    name = "User text editor",
    level = "info",
    target = "Editor for the Blog Post",
    skip(req, mongo_client, redis_client, input)
)]
#[post("/editor_submission")]
pub async fn editor_submission(
    redis_client: Data<aio::ConnectionManager>,
    mongo_client: Data<mongodb::Client>,
    Form(mut input): web::Form<BlogPost>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Edit submission endpoint");

    tracing::warn!("The blogpost to pass to the editor: {input:#?}");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    input.toggle_logged_in();
    input.set_user_id(user_oid.to_string());

    let edit_template = JournalPostEditor::new(input);

    let render = match edit_template.render() {
        Ok(html) => html,
        Err(err) => {
            tracing::error!(
                ?err,
                %user_oid,
                "Entry retrieved but rendering failed"
            );
            return HttpResponse::InternalServerError()
                .body("Entry retrieved but rendering failed.");
        }
    };

    HttpResponse::Ok().body(render)
}

/// Update the text of the most recently posted journal entry for the authenticated user.
#[allow(clippy::future_not_send)]
#[instrument(
    name = "User updates a previously submitted text",
    level = "info",
    target = "Upadte Blog Post",
    skip(req, mongo_client, redis_client, input)
)]
#[post("/update_text")]
pub async fn update_text(
    req: HttpRequest,
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    Form(input): web::Form<BlogPost>,
) -> HttpResponse {
    tracing::info!("Update text endpoint");

    tracing::warn!("The BlogPost to modify: {input:#?}");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    let journal_entries = match mongo::establish_connection(&mongo_client).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(?err, "Unable to procure a db connection");
            return HttpResponse::InternalServerError().finish();
        }
    }
    .collection::<BlogPost>("BlogPosts");

    // Workaround for HTMX error that doesn't parse newlines
    let map = input.get_body().chars().map(|mut letter| {
        if letter == '\n' {
            letter = ' ';
        }
        letter
    });
    let mut stripped_body: String = map.collect();

    if stripped_body.is_empty() {
        stripped_body = input.get_body().to_string();
    }

    tracing::warn!("The post id to search for: {}", input.get_post_id());
    tracing::warn!("The body content to update with: {}", stripped_body);
    // Find the latest journal entry for the authenticated user
    let filter = doc! {
        "_id": input.get_post_id(),
    };

    // The exact data to be updated
    let update_doc = doc! {
    "$set": doc! {
        "title": input.get_title(),
        "body": stripped_body,
        "author": input.get_author(),
        "date": BsonDateTime::now(),
    }
    };

    let options = options::FindOneAndUpdateOptions::builder()
        //     .sort(doc! { "date": -1 }) // Sort by date in descending order
        .return_document(options::ReturnDocument::After) // Return the updated document
        .build();

    let _entry: BlogPost = match journal_entries
        .find_one_and_update(filter, update_doc)
        .with_options(options)
        .await
    {
        Ok(Some(entry)) => {
            tracing::warn!("The results of the update: {entry:#?}");

            let edit_template = IndivPost::new(entry);

            let render = match edit_template.render() {
                Ok(html) => html,
                Err(err) => {
                    tracing::error!(
                        ?err,
                        %user_oid,
                        "Entry retrieved but rendering failed"
                    );
                    return HttpResponse::InternalServerError()
                        .body("Entry retrieved but rendering failed.");
                }
            };

            return HttpResponse::Ok().body(render);
            // entry
        }
        Ok(None) => {
            tracing::error!(%user_oid, "No journal entry found for the user to update the posts");
            return HttpResponse::NotFound()
                .body("No journal entry found for the user to update the post");
        }
        Err(err) => {
            tracing::error!(?err, %user_oid, "Unable to retrieve the journal entry");
            return HttpResponse::InternalServerError()
                .body("Unable to retrieve the journal entry");
        }
    };

    // if entry.matched_count.ne(&0) {
    //     tracing::warn!(%user_oid, "User update returned more than one entry: {entry:#?}");
    //     return HttpResponse::InternalServerError()
    //         .body("User update returned more than one entry. This should not happen.");
    // }

    // get the updated entry from the database to render it
    // let updated_entry: BlogPost = match journal_entries
    //     .find_one(doc! { "_id": &user_oid.to_string() })
    //     .await
    // {
    //     Ok(Some(entry)) => entry,
    //     Ok(None) => {
    //         tracing::warn!(
    // 		%user_oid, "No journal entry found for the user after update when querying the updated entry");
    //         return HttpResponse::NotFound()
    //             .body("No journal entry found for the user after update");
    //     }

    //     Err(err) => {
    //         tracing::error!(?err, %user_oid, "Unable to retrieve the updated journal entry");
    //         return HttpResponse::InternalServerError()
    //             .body("Unable to retrieve the updated journal entry");
    //     }
    // };
}

/// Delete the most immediately posted post from the user
// #[authenticate] // injects a variable `user_oid` into the request extensions
#[allow(clippy::future_not_send)]
#[instrument(
    name = "User Deletes a submitted blog",
    level = "info",
    target = "Delete Submission",
    skip(req, mongo_client, redis_client)
)]
#[delete("/delete_submission")]
pub async fn delete_submission(
    req: HttpRequest,
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    web::Query(input): web::Query<BlogPost>,
) -> HttpResponse {
    tracing::info!("Delete submission endpoint");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    let journal_entries = match mongo::establish_connection(&mongo_client).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(?err, "Unable to procure a db connection");
            return HttpResponse::InternalServerError().finish();
        }
    }
    .collection::<BlogPost>(&BlogPost::to_name());

    let filter = doc! {
    "_id": input.get_post_id()
    };

    match journal_entries.find_one_and_delete(filter).await {
        Ok(deleted_entry) => {
            tracing::warn!(%user_oid, "Deleted journal entry: {deleted_entry:#?}");

            let filter = mongodb::bson::doc! { "user_id": deleted_entry.as_ref().expect("unable to delete").get_user_id() };
            let mut blog_post: Vec<BlogPost> = Vec::new();

            match journal_entries.find(filter).await {
                //
                Ok(mut user_cursor) => {
                    tracing::info!("User found!"); //
                    //
                    while let Some(result) = user_cursor.next().await {
                        match result {
                            Ok(document) => {
                                blog_post.push(document); //
                            }
                            Err(err) => {
                                tracing::error!("Error retrieving document: {err:#?}");
                                return HttpResponse::InternalServerError()
                                    .json(format!("Error retrieving document: {err:#?}"));
                            }
                        }
                    }

                    tracing::warn!("Found {} number of posts", blog_post.len());
                }

                Err(err) => {
                    tracing::error!("Error accessing the database: {err:#?}"); //
                    return HttpResponse::InternalServerError().json(format!(
                        //
                        "Unable to acquire the database connection: {err:#?}" //
                    )); //
                } //
            } //

            let index_template = IndexTemplate {
                user_email: String::from(deleted_entry.expect("Unable to delete").get_author()),
                is_logged_in: true,
                content: blog_post,
                ..Default::default()
            };

            let render = match index_template.render() {
                Ok(render) => render,
                Err(err) => {
                    tracing::error!("Unable to render the index page after deleting a post");
                    // Make this an InternalServerError
                    return HttpResponse::Ok().json(format!(
                        "Unable to render the Index page after deleting a post: {err:#?}"
                    ));
                }
            };
            HttpResponse::Ok().body(render)
        }
        Err(err) => {
            tracing::error!(?err, %user_oid, "Unable to delete the journal entry");
            HttpResponse::InternalServerError().json("Unable to delete the journal entry")
        }
    }
}

/// # Errors
///   If the user is not logged in, return an "`HttpResponse::Unauthorized`" error.
/// # Panics
///   If the "`redis`" connection pool is not available, the function will panic.
#[allow(clippy::future_not_send)]
#[instrument(
    name = "Validates a user",
    level = "info",
    target = "User input",
    skip(req, redis_client)
)]
pub async fn validate_user(
    req: &HttpRequest,
    redis_client: &Data<redis::aio::ConnectionManager>,
) -> Result<bool, HttpResponse> {
    let session_id = if let Some(cookie) = req.cookie("session_id") {
        tracing::info!("Cookie found");
        cookie.value().to_string()
    } else {
        tracing::error!("User cookie not found: {:#?}", req.cookies());
        return Err(HttpResponse::Unauthorized().body(format!(
            "No session found: {:#?}",
            req.cookies().expect("No cookies found")
        )));
    };

    // let mut red_conn = match redis_client {
    //     Ok(conn) => conn,
    //     Err(err) => {
    //         tracing::error!("Unable to acquire the cache layer connection: {err:#?}");
    //         return Err(
    //             HttpResponse::InternalServerError().body(format!("Cache layer error: {err:#?}"))
    //         );
    //     }
    // };

    tracing::info!("Creating the session key");
    let session_key = format!("session:{session_id}");

    // tracing::warn!("The cache-layer's session key: {session_key}");

    tracing::info!("The session key to use to search: {session_key}");
    let user: String = match redis_client.as_ref().clone().get(session_key).await {
        Ok(result) => result,
        Err(err) => {
            tracing::error!("Error accessing the cache layer: {err:#?}");
            return Err(HttpResponse::InternalServerError()
                .body(format!("Unable to acquire the cache layer: {err:#?}")));
        }
    };

    if user.is_empty() {
        tracing::warn!("User is not logged in: {user:#?}");
        Ok(false)
    } else {
        tracing::info!("User is logged in: {user:#?}");
        Ok(true)
    }
}

#[allow(clippy::future_not_send)]
#[instrument(
    name = "User Deletes a submitted blog",
    level = "info",
    target = "Delete Submission",
    skip(req, mongo_client, redis_client, image)
)]
#[post("/post_image")]
pub async fn post_image(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    req: HttpRequest,
    MultipartForm(image): MultipartForm<ImageUpload>,
) -> HttpResponse {
    tracing::info!("Post image endpoint");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    tracing::info!(%user_oid, "User is authenticated");

    // Handle the image upload logic here

    if let Some(img) = &image.image {
        tracing::warn!(
            size = img.size / (1024 * 1024),
            file_name = ?img.file_name,
            content_type = ?img.content_type,
        );

        // Give the file size in Mega Bytes not Mega bits
        let img_size: f64 = f64::from(u32::try_from(img.size).expect("")) / (1024.0 * 1024.0);
        tracing::warn!("Image size in Mb: {img_size:.2} MB");

        let img_location: &str = &img.file.path().to_string_lossy();

        tracing::warn!("The location on this system of the temp file: {img_location}");

        // Decode the file as an image
        let img_file: File = File::open(img_location).expect("");

        // tracing::warn!("The file is open: {:#?}", img_file.metadata().expect(""));

        let metadata = match img_file.try_clone().expect("").metadata() {
            Ok(data) => data,
            Err(err) => {
                tracing::error!("Unable to get parse file metadata: {err:#?}");
                return HttpResponse::Ok().body("Unable to parse the image");
            }
        };

        // Ensure the file is not a directory or a symlink
        if metadata.is_dir() || metadata.is_symlink() && metadata.is_file() {
            tracing::error!("Image is not a file");
            return HttpResponse::Ok().body("Image is not a file");
        }

        // img_file.lock().unwrap()

        // let image_id = img
        //     .file_name
        //     .as_ref()
        //     .expect("Image has no name")
        //     .to_string();

        return HttpResponse::Ok().body(format!("Image file size is: {img_size:.3} Mb"));
    }
    tracing::info!("No image uploaded");

    HttpResponse::Ok().body("Image Error")
}

// #[allow(clippy::future_not_send)]
// #[post("/draft/autosave")]
// pub async fn autosave_journal_draft(
//     req: HttpRequest,
//     mongo_client: Data<mongodb::Client>,
//     redis_client: Data<aio::ConnectionManager>,
//     Form(input): Form<JournalDraftInput>,
// ) -> HttpResponse {
//     let user_id = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
//         Ok(user_id) => user_id,
//         Err(AuthenticationError::MissingSession | AuthenticationError::InvalidSession) => {
//             return HttpResponse::Unauthorized()
//                 .body("Your session expired. The draft was not saved");
//         }
//         Err(AuthenticationError::Redis) => {
//             tracing::error!("Unable to access Redis during draft autosave");

//             return HttpResponse::ServiceUnavailable()
//                 .body("Draft autosave is temporarily unavailable");
//         }
//     };

//     let title = input.title.trim();
//     let author = input.author.trim();

//     if title.chars().count() > 300 {
//         return HttpResponse::BadRequest().body("Draft was not saved: title is too long");
//     }
//     if input.body.chars().count() > 1_000_000 {
//         return HttpResponse::BadRequest().body("Draft was not saved: journal entry is too long");
//     }

//     if author.chars().count() > 200 {
//         return HttpResponse::BadRequest().body("Draft was not saved: author name is too long");
//     }

//     // Check if the meaningful fields have been filled in
//     if title.is_empty() && input.body.trim().is_empty() {
//         return HttpResponse::Ok().body("Begin typing to create a draft");
//     }

//     let drafts = mongo::establish_connection(&mongo_client)
//         .await
//         .expect("mongo error")
//         .collection::<JournalDraft>("journal_drafts");

//     let now = bson::DateTime::now();

//     let draft_id =
//         match bson::oid::ObjectId::parse_str(input.draft_id.as_ref().unwrap_or(&"".to_string())) {
//             Ok(draft_id) => draft_id,
//             Err(err) => {
//                 tracing::error!(
//                     ?err,
//                     "Unable to parse the draft_id: {}",
//                     input.draft_id.as_ref().unwrap_or(&"".to_string())
//                 );
//                 return HttpResponse::BadRequest().body("Draft was not saved: invalid draft_id");
//             }
//         };

//     let filter = doc! {
//     "_id": draft_id,
//     "user_id": user_id,
//     "state": "active"
//     };

//     let update = doc! {
//     "$set": {
//         "title": title,
//         "body": &input.body,
//         "author": author,
//         "updated_at": now,
//     },
//     "$setOnInsert": {
//         "_id": mongodb::bson::oid::ObjectId::new(),
//         "user_id": user_id,
//         "created_at": now,
//     },
//     "$inc": {
//         "revision": 1_i64
//     },
//     };

//     let update_result = drafts.update_one(filter, update).upsert(true).await;

//     match update_result {
//         Ok(_) => {
//             tracing::warn!("DB update successful for user_id: {user_id}");
//             let displayed_time = chrono::Utc::now().format("%Y-%m-%d %H:%S UTC");

//             let message = format!("Draft saved at {displayed_time}.");

//             let response = DraftStatusTemplate {
//                 status_class: "saved",
//                 message: &message,
//             };

//             match response.render() {
//                 Ok(rend) => HttpResponse::Ok()
//                     .content_type("text/html; charset=utf-8")
//                     .body(rend),

//                 Err(err) => {
//                     tracing::error!("Unable to render draft status: {err}");
//                     HttpResponse::InternalServerError().finish()
//                 }
//             }
//         }
//         Err(err) => {
//             tracing::error!(
//             ?err,
//             %user_id,
//             "Unable to autosave journal draft"
//             );

//             HttpResponse::InternalServerError()
//                 .body("The draft could not be saved. Continue typing and try again")
//         }
//     }
// }

/// # Errors
///
/// If the database connection fails, or if the query fails, this function will return a `mongodb::error::Error`.
pub async fn find_active_draft(
    mongo: &mongodb::Client,
    user_id: mongodb::bson::oid::ObjectId,
) -> anyhow::Result<Option<JournalDraft>> {
    let drafts = mongo::establish_connection(mongo)
        .await?
        .collection::<JournalDraft>("journal_drafts");

    Ok(drafts
        .find_one(doc! {
            "user_id": user_id
        })
        .await?)
}

// #[allow(clippy::future_not_send)]
// #[delete("draft/current")]
// pub async fn discard_current_draft(
//     req: HttpRequest,
//     mongo_client: Data<mongodb::Client>,
//     redis_client: Data<aio::ConnectionManager>,
// ) -> HttpResponse {
//     let user_id = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
//         Ok(user_id) => user_id,
//         Err(AuthenticationError::MissingSession | AuthenticationError::InvalidSession) => {
//             return HttpResponse::Unauthorized().body("Your session has expired");
//         }
//         Err(AuthenticationError::Redis) => {
//             tracing::error!("The cache-layer could not be established");
//             return HttpResponse::ServiceUnavailable()
//                 .json("The draft could not be discarded".to_string());
//         }
//     };

//     let drafts = match mongo::establish_connection(&mongo_client).await {
//         Ok(db) => db,
//         Err(err) => {
//             tracing::error!("Unable to procure the database: {err}");
//             return HttpResponse::InternalServerError().body("Failed to procure the db: {err}");
//         }
//     }
//     .collection::<JournalDraft>("journal_drafts");

//     match drafts
//         .delete_one(doc! {
//             "user_id": user_id,
//         })
//         .await
//     {
//         Ok(_) => {
//             tracing::info!("Successfully deleted an entry");
//             let empty_form = JournalFormTemplate::new(vec![BlogPost::default()]);
//             match empty_form.render() {
//                 Ok(html) => HttpResponse::Ok()
//                     .content_type("text/html; charset=utf-8")
//                     .body(html),
//                 Err(err) => {
//                     tracing::error!(?err, %user_id, "Unable to save journal draft");
//                     HttpResponse::InternalServerError().finish()
//                 }
//             }
//         }
//         Err(err) => {
//             tracing::error!(?err, "Unable to render empty journal form");
//             HttpResponse::InternalServerError().body("The draft could not be discarded")
//         }
//     }
// }

pub async fn get_all_posts(
    mongo_client: &Data<mongodb::Client>,
    user_oid: Oid,
) -> anyhow::Result<Vec<BlogPost>> {
    let db: mongodb::Collection<BlogPost> =
        match mongo::establish_connection(&mongo_client).await {
            Ok(collection) => collection,
            Err(err) => {
                tracing::error!("Error accessing the database: {err:#?}");
                return Err(anyhow::Error::msg(format!(
                    "Error accessing the database: {err:#?}"
                )));
            }
        }
        .collection::<BlogPost>(&BlogPost::to_name());

    let filter = mongodb::bson::doc! { "user_id": user_oid.to_string() };
    tracing::warn!("The id to check against: {}", user_oid.to_string());
    let mut blog_post: Vec<BlogPost> = vec![];

    // Each user can have more than one blog post, so we need to find all of them
    match db.find(filter).await {
        Ok(mut user_cursor) => {
            tracing::info!("User found!");

            while let Some(result) = user_cursor.next().await {
                match result {
                    Ok(document) => {
                        blog_post.push(document);
                    }
                    Err(err) => {
                        tracing::error!("Error retrieving document: {err:#?}");
                        return Err(anyhow::Error::msg(format!(
                            "Error retrieving document: {err:#?}"
                        )));
                    }
                }
            }

            tracing::warn!("Found {} number of posts", blog_post.len());
            Ok(blog_post)
        }

        Err(err) => {
            tracing::error!("Error accessing the database: {err:#?}");
            return Err(anyhow::Error::msg(format!(
                "Error accessing the database: {err:#?}"
            )));
        }
    }
}
