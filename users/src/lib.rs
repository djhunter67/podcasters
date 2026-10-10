pub struct Users {
    pub email: String,
    password: String,
}

impl Users {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}
