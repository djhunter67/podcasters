use std::fmt::Display;

use base64::{Engine, engine::general_purpose};
use chacha20::{ChaCha20, KeyIvInit, cipher::StreamCipher};
use tracing::instrument;

use crate::security::PEPPER;

/// Encryption Logic
#[derive(Debug, Clone)]
pub struct PassWorder {
    pw: String,
}

impl Display for PassWorder {
    // fn to_string(&self) -> String {
    //     self.pw.clone()
    // }
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pw)
    }
}

impl PassWorder {
    /// Implementor for the password
    #[instrument(
        name = "Password Encryption",
        level = "info",
        target = "sundayLifeServices web app",
        // skip(pw)
    )]
    pub fn new(pw: &str) -> Self {
        Self { pw: pw.to_string() }
    }

    #[instrument(
        name = "User registration attempted",
        level = "info",
        target = "sundayLifeServices web app"
    )]
    pub fn get(self) -> String {
        self.pw
    }

    /// Encrypt the password
    #[instrument(
        name = "Password Encryption",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    #[must_use = "Encrypt plain text passwords"]
    pub fn encrypt(mut self) -> Self {
        let key: [u8; 32] = *b"an example very very secret key!";
        let nonce: [u8; 12] = *b"unique nonce";

        let mut encryptor = ChaCha20::new(&key.into(), &nonce.into());

        let mut cipher_text = self.pw.into_bytes();

        encryptor.apply_keystream(&mut cipher_text);

        tracing::info!("Encrypted");

        // tracing::warn!("The generated cipher: {}", hex::encode(&cipher_text));
        self.pw = hex::encode(&cipher_text);
        self
    }

    #[instrument(
        name = "Password Salting",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    #[must_use = "Salt encrypted passwords"]
    pub fn salt(mut self) -> Self {
        tracing::debug!("Salting");
        // let seed: [u8; 32] = [42u8; 32];

        // let mut random_salt: [u8; 16] = [0; 16];

        // let mut seed_core: ChaCha20Rng = ChaCha20Rng::from_seed(seed);

        // seed_core.fill_bytes(&mut random_salt);

        let random_salt: [u8; 16] = rand::random();

        self.pw
            .insert_str(0, &format!("{}$", hex::encode(random_salt)));
        tracing::info!("Salted");

        // tracing::warn!("The generated Salt: {}", hex::encode(random_salt));

        self
    }

    #[instrument(
        name = "Password Peppering",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    #[must_use = "To pepper encrypted and salted passwords"]
    pub fn pepper(mut self) -> Self {
        // self.pw += "the_pepper";

        let base_64_pepper = general_purpose::STANDARD.encode(PEPPER);

        self.pw += &String::from_utf8_lossy(base_64_pepper.as_bytes());
        // tracing::warn!("The Peppered PW: {}", self.pw);
        tracing::info!("Peppered");
        self
    }

    #[instrument(
        name = "Password deconstructor",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    pub fn deconstruct(&self) -> (String, String, String) {
        // tracing::warn!("the PW to deconstruct: {}", self.pw);
        let (salt, hash) = if let Some((salt, hash)) = self.pw.split_once('$') {
            (salt, hash)
        } else {
            tracing::error!("No split delimeter found in the pw string");
            tracing::warn!("Passing back empty results");
            ("", "")
        };

        match general_purpose::STANDARD.decode(
            self.pw
                .split_at(self.pw.len() - general_purpose::STANDARD.encode(PEPPER).len())
                .1,
        ) {
            Ok(pepper) => (
                String::from(salt),
                String::from(hash),
                String::from_utf8_lossy(&pepper).to_string(),
            ),
            Err(err) => {
                tracing::error!("Base64 decode failure: {err:?}");
                (
                    String::from(salt),
                    String::from(hash),
                    String::from(self.pw.split_at(self.pw.len() - PEPPER.len()).1),
                )
            }
        }
    }
}
