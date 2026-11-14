pub mod auth;

use std::fmt;

use mongodb::bson::oid;
use serde::{Deserialize, Serialize};
use shared::security::passworder;
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    _id: Option<oid::ObjectId>,
    email: String,
    password: String,
    subscriptions: Vec<String>, // To be Episode ID's
    status: AccountStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AccountStatus {
    PendingVerification,
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Auth {
    pub email: String,
    pub password: Option<String>,
    pub oid: Option<String>,
}

impl Users {
    pub fn new(email: &str, password: String) -> Self {
        Self {
            email: email.to_string(),
            password,
            subscriptions: vec![],
            _id: None,
            status: AccountStatus::PendingVerification,
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

    pub fn get_oid(&self) -> String {
        match self._id {
            Some(id) => id.to_string(),
            None => {
                tracing::error!("Unable to get the OID of the user");
                String::from("None")
            }
        }
    }

    pub fn get_status(&self) -> String {
        self.status.to_string()
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
            "\nEmail: {}\nPassword: ********\nSubscriptions: {has_some}\noid: {:#?}",
            self.get_email(),
            self._id
        )
    }
}

impl Default for Users {
    fn default() -> Self {
        Self {
            email: String::from("default_email@default_user.com"),
            password: String::from("**default**"),
            subscriptions: Default::default(),
            _id: None,
            status: AccountStatus::PendingVerification,
        }
    }
}

impl From<&Auth> for Users {
    fn from(value: &Auth) -> Self {
        Self {
            email: value.email.clone(),
            password: value.password.clone().expect("password not passed"),
            subscriptions: vec![],
            _id: None,
            status: AccountStatus::PendingVerification,
        }
    }
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Suspended => write!(f, "Suspended"),
            Self::PendingVerification => write!(f, "Pending Verification"),
            Self::Deleted => write!(f, "Deleted"),
        }
    }
}
