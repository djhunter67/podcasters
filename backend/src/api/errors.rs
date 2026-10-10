use std::fmt;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum RegistrationError {
    Password(String),
    Database(String),
    Email(String),
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistrationError::Password(err) => write!(f, "Password Error: {err}"),
            RegistrationError::Database(err) => write!(f, "Database Error: {err}"),
            RegistrationError::Email(err) => write!(f, "Username Error: {err}"),
        }
    }
}
