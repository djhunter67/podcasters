use askama::Template;

use crate::{personnel::users, startup::VERSION};

use super::user_input::BlogPost;

/// # TODO
///
/// Dark Theme
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub content: Vec<BlogPost>,
    pub version: String,
    pub user_email: String,
    pub is_logged_in: bool,
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            title: String::from("Home"),
            content: Vec::default(),
            version: default_version(),
            user_email: String::from("Please login to create a journal entry"),
            is_logged_in: false,
        }
    }
}

impl IndexTemplate {
    /// Creates a new [`IndexTemplate`].
    #[must_use]
    pub fn new(content: Vec<BlogPost>, user_email: &str, is_logged_in: bool) -> Self {
        Self {
            title: String::from("Home"),
            content,
            version: env!("CARGO_PKG_VERSION").to_string(),
            user_email: String::from(user_email),
            is_logged_in,
        }
    }

    /*
        /// # Panics
        ///
        /// - Pannics if the `draft` is `Some` and the `updated_at` field cannot be converted to a `chrono::DateTime`.
        #[must_use]
        pub fn with_draft(
            title: String,
            content: Vec<BlogPost>,
            user_email: String,
            is_logged_in: bool,
            draft: Option<JournalDraft>,
        ) -> Self {
            let (draft_id, draft_title, draft_body, draft_author, draft_saved_at) = match draft {
                Some(draft) => {
                    let saved_at: Result<String, ()> = Ok(chrono::DateTime::<chrono::Utc>::from(
                        draft.updated_at.to_system_time(),
                    ))
                    .map(|date| date.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string());
                    (
                        draft.id.to_hex(),
                        draft.title,
                        draft.body,
                        draft.author,
                        saved_at.expect("Error with 'save_at' time"),
                    )
                }

                None => (
                    String::new(),
                    String::new(),
                    String::new(),
                    "Hunter, Christerper".to_owned(),
                    String::new(),
                ),
            };

            Self {
                title,
                version: "1".to_string(),
                content,
                user_email,
                is_logged_in,
            }
    }
        */
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub title: &'a str,
    pub is_logged_in: bool,
    pub user_email: &'a str,
    pub version: &'a str,
}

impl Default for LoginTemplate<'_> {
    fn default() -> Self {
        Self {
            title: "Login",
            is_logged_in: Default::default(),
            user_email: "Please login to create a post",
            version: VERSION,
        }
    }
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate<'a> {
    pub title: &'a str,
    pub content: Vec<&'a str>,
    pub is_logged_in: bool,
    pub user_email: &'a str,
    pub version: &'a str,
}

impl Default for RegisterTemplate<'_> {
    fn default() -> Self {
        Self {
            title: Default::default(),
            content: Vec::default(),
            is_logged_in: Default::default(),
            user_email: Default::default(),
            version: VERSION,
        }
    }
}

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate<'a> {
    pub title: &'a str,
    pub is_logged_in: bool,
    pub user_email: &'a str,
    pub version: &'a str,
}

impl Default for SettingsTemplate<'_> {
    fn default() -> Self {
        Self {
            title: Default::default(),
            is_logged_in: Default::default(),
            user_email: Default::default(),
            version: VERSION,
        }
    }
}

#[derive(Template)]
#[template(path = "parts/indiv_post.html")]
pub struct IndivPost {
    content: BlogPost,
}

impl IndivPost {
    #[must_use = "Create a new IndivPost"]
    pub const fn new(content: BlogPost) -> Self {
        Self { content }
    }
}

#[derive(Template)]
#[template(path = "parts/journal_form.part.html")]
pub struct IndivInput {
    content: users::Users,
}
impl IndivInput {
    #[must_use = "Create a new instance since the members are private"]
    pub const fn new(content: users::Users) -> Self {
        Self { content }
    }
}

#[derive(Template)]
#[template(path = "parts/journal_form_input.part.html")]
pub struct JournalPostEditor {
    content: BlogPost,
}

impl JournalPostEditor {
    #[must_use = "This function is used to allow a user to edit their post"]
    pub const fn new(content: BlogPost) -> Self {
        Self { content }
    }
}

#[derive(Template)]
#[template(path = "parts/confirmations.part.html")]
pub struct Confirmation {
    header_message: String,
    body_message: String,
}

impl Confirmation {
    #[must_use]
    pub const fn new(header_message: String, body_message: String) -> Self {
        Self {
            header_message,
            body_message,
        }
    }
}

#[derive(Template)]
#[template(path = "parts/draft_status.part.html")]
pub struct DraftStatusTemplate<'a> {
    pub status_class: &'a str,
    pub message: &'a str,
}

#[derive(Template)]
#[template(path = "parts/modal_load.part.html")]
pub struct ErrorPage<'a> {
    pub title: &'a str,
    pub code: u32,
    pub error: &'a str,
    pub message: &'a str,
}

impl<'a> ErrorPage<'a> {
    #[must_use]
    pub const fn new(message: &'a str) -> Self {
        Self {
            title: "Error",
            code: 500,
            error: "Internal Server Error",
            message,
        }
    }
}
