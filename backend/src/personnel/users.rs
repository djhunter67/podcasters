use actix_web::web;
use redis::{AsyncCommands, aio};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    endpoints::login::LoginUser,
    models::mongo,
    security::passworder::{self},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Users {
    email: String,
    password_hash: String,
    password_salt: String,
}

impl Users {
    /// Creates a new [`Users`].
    #[must_use = "Create a new user"]
    pub const fn new(email: String, password_hash: String, password_salt: String) -> Self {
        Self {
            email,
            password_hash,
            password_salt, // Accountability for if two or more users have the same password
        }
    }

    /// Get the email field of the Struct ``Users``
    #[instrument(
        name = "Get the user's password",
        level = "info",
        target = "Personnel",
        skip(self)
    )]
    #[must_use = "Get the email field of the Struct ``Users``"]
    pub fn get_email(&self) -> String {
        self.email.clone()
    }

    /// Set the email field of the Struct ``Users``
    #[instrument(
        name = "Set the user's email",
        level = "info",
        target = "Personnel",
        skip(self, new_email)
    )]
    pub fn set_email(&mut self, new_email: &str) {
        self.email = String::from(new_email)
    }

    /// Get the user's password
    #[instrument(
        name = "Get the user's password",
        level = "info",
        target = "Personnel",
        skip(self)
    )]
    pub fn get_pw(&self) -> String {
        self.password_hash.clone()
    }

    /// Set the user's password
    #[instrument(
        name = "Set the user's password",
        level = "info",
        target = "Personnel",
        skip(self, new_pw)
    )]
    pub fn set_pw(&mut self, new_pw: &str) {
        let new_pw_hash = passworder::PassWorder::new(new_pw)
            .encrypt()
            .salt()
            .pepper();
        self.password_hash = new_pw_hash.to_string();
    }

    /// Take in a plain-text unencrypted password to check against
    /// It is expected the the ``user_to_check` is the ``Users`` pulled directly from the database
    #[instrument(
        name = "Ensure the passwords are equivalent",
        level = "info",
        target = "Personnel",
        skip(self, db_user)
    )]
    fn compare_pw(&self, db_user: &Self) -> bool {
        tracing::debug!("The user passed in for the password comparison: {db_user:#?}");
        // let pw: (_, String, _) = PassWorder::new(db_user.get_pw()).deconstruct();

        let encrypted_pw = passworder::PassWorder::new(&self.get_pw())
            .encrypt()
            .salt()
            .pepper();
        // .get();
        // .pepper()  // Cannot pepper until I redo registration

        // {
        //     // Troubleshooting block
        //     let (salt, pw, _pepper) = passworder::PassWorder::new(&self.get_pw())
        //         .encrypt()
        //         .salt()
        //         .pepper()
        //         .deconstruct();

        //     use base64::Engine;
        //     let base_64_pepper = base64::engine::general_purpose::STANDARD.encode(*b"the_pepperer");

        //     let pepper = String::from_utf8_lossy(base_64_pepper.as_bytes());

        //     tracing::error!("\nPWD: {pw}\nSLT: {salt}\nPEP: {pepper}");
        // }

        let db_pw = db_user.get_pw();

        let (_salt, db_pworder, _pepper) = passworder::PassWorder::new(&db_pw).deconstruct();

        let (_salt, passed_in_pw, _pepper) = encrypted_pw.deconstruct();

        tracing::info!(
            "The pw's match: {} -> \npassed_in: {}\ndb_user: {}",
            // encrypted_pw == db_user.password_hash,
            passed_in_pw == db_pworder,
            // encrypted_pw,
            passed_in_pw,
            // db_user.password_hash
            db_pworder
        );

        // self.password_hash.eq(&db_user.password_hash)
        // encrypted_pw.eq(&db_user.password_hash)
        passed_in_pw.eq(&db_pworder)
    }

    /// # Errors
    ///
    ///   - This will return an `anyhow` error if the connection to the database cannot be established
    #[instrument(
        name = "Password Verifier",
        level = "info",
        target = "User Login Attempt",
        skip(self, redis_conn, mongo_client)
    )]
    pub async fn pw_verify(
        &self,
        mongo_client: &mongodb::Client,
        redis_conn: &mut aio::ConnectionManager,
        test: Option<bool>,
    ) -> anyhow::Result<bool> {
        tracing::debug!("Verifying the user entered password");

        let cache_key = format!("user:auth:{}", self.email);
        tracing::warn!("The cache key to use to get the email to verify the user: {cache_key}");

        // let mut redis_conn = match redis_conf::establish_connection(redis_client) {
        //     Ok(conn) => conn,
        //     Err(err) => {
        //         tracing::error!("Unable to connect to the cache-layer: {err:?}");
        //         return Err(anyhow::Error::msg("Unable to connect to the cache-layer"));
        //     }
        // };
        // Get the user's key from when the user registered
        let user_oid: String = match redis_conn.get(cache_key).await {
            Ok(cached_user) => cached_user,
            Err(err) => {
                tracing::warn!("No registration keys detected: {err}");
                String::new()
            }
        };

        tracing::warn!("The user_oid found: {user_oid:#?}");

        let oid: mongodb::bson::oid::ObjectId =
            serde_json::from_str::<mongodb::bson::oid::ObjectId>(&user_oid)?;

        tracing::warn!("The oid found in the cache-layer: {oid}");

        // Compare the password saved and the password entered
        let filter = mongodb::bson::doc! {
            "_id": oid,
        };

        let mongo_conn = mongo::establish_connection(mongo_client).await?;
        let user: Self = mongo_conn
            .collection::<Self>(if test.unwrap_or_default() {
                eprintln!("\n\nRunning the TEST database\n\n");
                "Test"
            } else {
                "Users"
            })
            .find_one(filter)
            .await?
            .unwrap_or_default();

        tracing::warn!("The user returned: {user:#?}");

        Ok(self.compare_pw(&user))
    }
}

impl From<web::Form<LoginUser>> for Users {
    fn from(value: web::Form<LoginUser>) -> Self {
        // let pwdr: PassWorder = PassWorder::new(&value.password).encrypt().salt().pepper();
        // let (salt, pw, _pep) = pwdr.deconstruct();
        // Self::new(value.email.clone(), pw, salt)
        Self::new(value.email.clone(), value.password.clone(), String::new())
    }
}

#[cfg(test)]
mod test {}
