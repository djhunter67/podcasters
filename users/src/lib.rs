use std::fmt;

use serde::{Deserialize, Serialize};
use shared::security::passworder;
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    email: String,
    password: String,
    subscriptions: Vec<String>, // To be Episode ID's
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

impl Users {
    pub fn new(email: &str, password: String) -> Self {
        Self {
            email: email.to_string(),
            password,
            subscriptions: vec![],
        }
    }

    pub fn get_email(&self) -> String {
        self.email.clone()
    }

    pub fn get_pw(&self) -> String {
        self.password.clone()
    }

    pub fn set_pw(&mut self, pw: passworder::PassWorder) {
        self.password = pw.to_string();
    }

    /// # Errors
    ///
    ///   - This will return an `anyhow` error if the connection to the database cannot be established
    #[instrument(
        name = "Password Verifier",
        level = "info",
        target = "User Login Attempt",
        skip(self, user_creds)
    )]
    pub fn pw_verify(&self, user_creds: &str) -> anyhow::Result<bool> {
        tracing::info!("Verifying the user entered password");

        tracing::info!(
            "The user passed in for the password comparison: {}",
            self.get_email()
        );

        tracing::warn!("The user entered PW: {user_creds}");

        let db_pw = self.get_pw();

        tracing::info!(
            "The pw's match: {} -> \npassed_in: {}\nUser:\t   {}",
            user_creds.eq(&db_pw),
            user_creds,
            &db_pw,
        );

        Ok(user_creds == db_pw)
    }
}

impl fmt::Display for Users {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let has_some = !self.subscriptions.is_empty();
        write!(
            f,
            "\nEmail: {}\nPassword: ********\nSubscriptions: {has_some}",
            self.get_email()
        )
    }
}

impl Default for Users {
    fn default() -> Self {
        Self {
            email: String::from("default_email@default_user.com"),
            password: String::from("**default**"),
            subscriptions: Default::default(),
        }
    }
}

impl From<&Login> for Users {
    fn from(value: &Login) -> Self {
        Self {
            email: value.email.clone(),
            password: value.password.clone(),
            subscriptions: vec![],
        }
    }
}
