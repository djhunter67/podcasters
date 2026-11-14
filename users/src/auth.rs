use chacha20::{ChaCha20, KeyIvInit, cipher::StreamCipher};
use hex;

pub struct Token {
    opaque: String,
}

impl Token {
    pub fn new(opaque: String) -> Self {
        Self { opaque }
    }
}

trait TokenGenerator {
    fn generate_token(&self, token: TokenType, expiration: u16) -> Token {
        tracing::info!("The token type: {token:#?}");
        tracing::info!("The Expiration of the token: {expiration}");
        Token {
            opaque: String::new(),
        }
    }

    fn generate(pw_nonce: &str) -> String {
        let key: [u8; 32] = *b"an example very very secret key!";
        let nonce: [u8; 12] = *b"unique nonce";

        let mut encryptor = ChaCha20::new(&key.into(), (&nonce).into());

        let mut cipher_text = pw_nonce.as_bytes().to_owned();

        encryptor.apply_keystream(&mut cipher_text);

        tracing::info!("Encrypted");

        // tracing::warn!("The generated cipher: {}", hex::encode(&cipher_text));
        hex::encode(&cipher_text).to_string()
    }
}

/// User receives the access token and refresh token upon successful login
pub struct Authentication {
    access_token: Option<String>, // Cookie -> Access API and make requests; Will allow user to stay logged without logging back in
    access_expiration: Option<u16>, // 15 minutes to 1 hour
    refresh_token: Option<String>, // Cookie -> Used to generate a new access token
    refresh_expiration: Option<u16>, // 7 to 30 days
}

impl Authentication {
    pub fn new() -> Self {
        Self {
            access_token: None,
            access_expiration: None,
            refresh_token: None,
            refresh_expiration: None,
        }
    }
}

impl Default for Authentication {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenGenerator for Authentication {}

pub struct Authorization {
    pub session_key: String,
    pub expiration: u16, // 3600 seconds
}

impl Authorization {
    pub fn new() -> Self {
        Self {
            session_key: String::from("Opaque token generated"),
            expiration: 3600,
        }
    }
}

impl TokenGenerator for Authorization {}

impl Default for Authorization {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum TokenType {
    Access,
    Refresh,
    Session,
}
