use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Users {
    pub email: String,
    password: String,
    subscriptions: Vec<String>, // To be Episode ID's
}

impl Users {
    pub fn new(email: String, password: String) -> Self {
        Self {
            email,
            password,
            subscriptions: vec![],
        }
    }
}
