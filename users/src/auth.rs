use chacha20::{ChaCha20, KeyIvInit, cipher::StreamCipher};
use hex;

pub struct Token {
    opaque: String,
}

impl Token {
    pub fn new(opaque: String) -> Self {
        Self { opaque }
    }

    fn default() -> Token {
        todo!()
    }
}

trait TokenGenerator {
    fn generate_token(&self, token: TokenType, expiration: u16) -> Token {
        tracing::info!("The token type: {token:#?}");
        tracing::info!("The Expiration of the token: {expiration}");
        match Self::generate("password_string", token) {
            Ok(val) => Token { opaque: val },
            Err(err) => {
                tracing::error!("{err:#?}");
                Token::default()
            }
        }
    }

    fn generate(pw: &str, token: TokenType) -> anyhow::Result<String> {
        let key: [u8; 32] = match token {
            TokenType::Access => *b"the access type for access token",
            TokenType::Refresh => {
                let item: String = format!("the access type for {} token", TokenType::Refresh);

                item.into_bytes()
                    .try_into()
                    .expect("Fail to get Refresh token")
            }
            TokenType::Session => {
                let item: String = format!("the access type for {} token", TokenType::Session);
                item.into_bytes()
                    .try_into()
                    .expect("Fail to get Refresh token")
            }
        };
        // let key: [u8; 32] = *b"token !                         ";
        let nonce: [u8; 12] = *b"unique nonce";

        let mut encryptor = ChaCha20::new(&key.into(), (&nonce).into());

        // let vals = [32_u8; 8];

        let mut cipher_text = pw.as_bytes().to_owned();

        encryptor.apply_keystream(&mut cipher_text);

        tracing::info!("Encrypted");

        // tracing::warn!("The generated cipher: {}", hex::encode(&cipher_text));
        Ok(hex::encode(&cipher_text))
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

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Access => write!(f, "Access"),
            TokenType::Refresh => write!(f, "Refresh"),
            TokenType::Session => write!(f, "Session"),
        }
    }
}
