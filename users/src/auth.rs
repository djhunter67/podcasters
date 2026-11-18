use std::time::{self, SystemTime};

use chacha20::{ChaCha20, KeyIvInit, cipher::StreamCipher};
use hex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Token {
    opaque: String,
    expiration: u32,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nExpiration: {:} hr(s)",
            self.opaque,
            self.expiration / 3600
        )
    }
}

impl Token {
    pub fn new(opaque: String, expiration: u32) -> Self {
        Self { opaque, expiration }
    }

    pub fn get(&self) -> String {
        self.opaque.clone()
    }

    pub fn expiration(&self) -> u32 {
        self.expiration
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            opaque: String::from("no token created: DEFAULT"),
            expiration: 60,
        }
    }
}

pub trait TokenGenerator {
    fn generate_token(token_type: TokenType) -> anyhow::Result<Token> {
        tracing::info!("The token type: {token_type:#?}");
        Ok(match Self::time_generator(&token_type) {
            Ok(val) => {
                let expiration: u32 = match token_type {
                    // 1 hour
                    TokenType::Access => 3600,
                    // 30 days => 1hr * (30 days * 24 hours in a day)
                    TokenType::Refresh => 3600 * (24 * 30),
                    // 90 days
                    TokenType::Session => (3600 * (24 * 30)) * 3,
                };
                Token {
                    opaque: val,
                    expiration,
                }
            }
            Err(err) => {
                tracing::error!("{err:#?}");
                Token::default()
            }
        })
    }

    fn _generator(token_string: &str, token: TokenType) -> anyhow::Result<String> {
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

        let mut cipher_text = token_string.as_bytes().to_owned();

        encryptor.apply_keystream(&mut cipher_text);

        tracing::info!("Encrypted");

        // tracing::warn!("The generated cipher: {}", hex::encode(&cipher_text));
        Ok(hex::encode(&cipher_text))
    }

    fn time_generator(token: &TokenType) -> anyhow::Result<String> {
        let seconds = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time travel has occured")
            .as_nanos();

        tracing::debug!("CURRENT TIME DELTA: {seconds} secs");
        tracing::debug!(
            "The time slice as bytes: {:?}",
            seconds.swap_bytes().to_le_bytes()
        );
        let time_slice =
            &seconds.swap_bytes().to_le_bytes()[seconds.to_le_bytes().to_vec().len() - 8..];

        tracing::debug!("The length of the time slice: {}", time_slice.len());
        tracing::debug!("The bytes to inject: {time_slice:?}");

        // let key: [u8; 32] = [0x42; 32];
        let key: [u8; 32] = match token {
            TokenType::Access => *b"access  type  for  Access tokens",
            TokenType::Refresh => *b"access  type  for Refresh tokens",
            TokenType::Session => *b"access  type  for Session tokens",
        };
        let mut nonce: [u8; 12] = [0; 12];

        nonce[0..8].copy_from_slice(time_slice);
        nonce[8..12].copy_from_slice(b"pods");

        let mut cipher = ChaCha20::new(&key.into(), &nonce.into());

        // Data to be used to make the opaque token
        let mut buff = match token {
            TokenType::Access => TokenType::Access.to_string(),
            TokenType::Refresh => TokenType::Refresh.to_string(),
            TokenType::Session => TokenType::Session.to_string(),
        }
        .as_bytes()
        .to_vec();

        cipher.apply_keystream(&mut buff);
        buff.clone().iter().for_each(|letter| buff.push(*letter));
        cipher.apply_keystream_b2b(&buff.clone(), &mut buff);
        cipher.apply_keystream_b2b(&buff.clone(), &mut buff);

        Ok(hex::encode(buff))
    }
}

/// User receives the access token and refresh token upon successful login
pub struct Authenticated {
    // access_token: Option<String>, // Cookie -> Access API and make requests; Will allow user to stay logged without logging back in
    // access_expiration: Option<u16>, // 15 minutes to 1 hour
    // refresh_token: Option<String>, // Cookie -> Used to generate a new access token
    // refresh_expiration: Option<u16>, // 7 to 30 days
}

// impl Authenticated {
//     pub fn new() -> Self {
//         Self {
//             access_token: None,
//             access_expiration: None,
//             refresh_token: None,
//             refresh_expiration: None,
//         }
//     }
// }

// impl Default for Authenticated {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl TokenGenerator for Authenticated {}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
    Session,
}

impl TokenType {
    /// Allow the TokenTypes to be iterated over
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Access, Self::Refresh, Self::Session].iter().copied()
    }
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
