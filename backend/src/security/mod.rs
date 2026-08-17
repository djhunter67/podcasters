pub mod passworder;
pub mod session;
pub mod validate;

/// This module tests the encryption, salt, and peppering of passwords
const PEPPER: [u8; 12] = *b"the_pepperer";

// Tests for the PassWorder struct
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use base64::{Engine, engine::general_purpose};
    use chacha20::{ChaCha20, KeyIvInit, cipher::StreamCipher};
    use redis::aio;
    use rstest::{fixture, rstest};

    use crate::{
        personnel::users::{self},
        security::passworder::PassWorder,
    };

    use super::*;

    #[fixture]
    async fn mongo_client() -> mongodb::Client {
        mongodb::Client::with_uri_str(
            "mongodb://admin:secret_passers@10.20.20.205:27017/?authMechanism=SCRAM-SHA-256&directConnection=true&replicaSet=rs0",
        ).await.unwrap()
    }

    #[fixture]
    async fn redis_client() -> aio::ConnectionManager {
        let client =
            redis::Client::open("redis://:secret_passers_redis@10.20.20.202:6379").unwrap();
        aio::ConnectionManager::new(client).await.unwrap()
    }

    #[test]
    fn test_password_encryption() {
        let pw = PassWorder::new("my_secret_password");
        let encrypted_pw = pw.encrypt();
        assert_ne!(encrypted_pw.get(), "my_secret_password");
    }

    #[test]
    fn test_password_peppering() {
        let pw = PassWorder::new("my_secret_password");
        let peppered_pw = pw.pepper();
        assert!(
            peppered_pw
                .get()
                .ends_with(general_purpose::STANDARD.encode(PEPPER).as_str())
        );
    }

    #[test]
    fn test_password_deconstruction() {
        let pw = PassWorder::new("my_secret_password");
        let salted_peppered_pw = pw.salt().pepper();
        let (salt, hash, pepper) = salted_peppered_pw.deconstruct();
        assert!(!salt.is_empty());
        assert!(!hash.is_empty());
        assert_eq!(
            pepper,
            PEPPER.iter().map(|b| *b as char).collect::<String>()
        );
    }

    #[test]
    fn test_pw_encryption_length() {
        let pw = PassWorder::new("my_secret_password");
        let encrypted_pw = pw.encrypt();
        assert_eq!(encrypted_pw.get().len(), 36);
    }

    #[test]
    fn test_pw_salt_uniqueness() {
        let pw1 = PassWorder::new("my_secret_password");
        let salted_pw1 = pw1.salt();
        let pw_2 = PassWorder::new("my_secret_password");
        let salted_pw2 = pw_2.salt();
        assert_ne!(salted_pw1.get(), salted_pw2.get());
    }

    #[test]
    fn test_pw_pepper_consistency() {
        let pw1 = PassWorder::new("my_secret_password");
        let pw_2 = PassWorder::new("my_secret_password");
        let peppered_pw1 = pw1.pepper();
        let peppered_pw2 = pw_2.pepper();
        assert_eq!(peppered_pw1.get(), peppered_pw2.get());
    }

    #[test]
    fn test_pw_contains_dollar_sign() {
        let pw = PassWorder::new("my_secret_password");
        let salted_pw = pw.encrypt().salt().pepper();
        assert!(salted_pw.get().contains('$'));
    }

    #[test]
    fn test_random_is_lenght_before_and_after_conversion() {
        let random_salt: [u8; 16] = rand::random();
        let key: [u8; 32] = *b"an example very very secret key!";
        let nonce: [u8; 12] = *b"unique nonce";

        let mut encryptor = ChaCha20::new(&key.into(), &nonce.into());

        let mut cipher_text = random_salt;

        encryptor.apply_keystream(&mut cipher_text);

        assert_eq!(random_salt.len(), cipher_text.len());
    }

    #[ignore = "Extensive development required to implement correctly; Cache layer session key required"]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_login_checker_verifier(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        let (salt, _pw, _pepper) = encrypted_pw.deconstruct();

        let verifier: users::Users = users::Users::new(
            "the_email@email.com".to_string(),
            encrypted_pw.to_string(),
            salt,
        );

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(match verifier
            .pw_verify(&mongo_client, &mut redis_client, Some(true))
            .await
        {
            Ok(val) => {
                let _ = mongo_conn.drop().await;
                val
            }
            Err(err) => {
                panic!("{}", format!("PW verifier error: {err:#?}"))
            }
        });
    }

    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_login_verifier_failure(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "the_password".to_string(),
            String::new(),
        );

        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(
            !verifier
                .pw_verify(&mongo_client, &mut redis_client, Some(true))
                .await
                .unwrap()
        );
    }
    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_login_verifier_with_salt_and_pepper(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "the_password".to_string(),
            String::new(),
        );

        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(match verifier
            .pw_verify(&mongo_client, &mut redis_client, Some(true))
            .await
        {
            Ok(val) => {
                let _ = mongo_conn.drop().await;
                val
            }
            Err(_err) => {
                false
            }
        });
    }
    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_login_verifier_with_incorrect_password(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "the_password".to_string(),
            String::new(),
        );

        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(
            !verifier
                .pw_verify(&mongo_client, &mut redis_client, Some(true))
                .await
                .unwrap()
        );
    }
    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_login_verifier_with_empty_password(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "the_password".to_string(),
            String::new(),
        );

        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(
            !verifier
                .pw_verify(&mongo_client, &mut redis_client, Some(true))
                .await
                .unwrap()
        );
    }
    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_login_verifier_with_special_characters(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "p@$$w0rd!".to_string(),
            String::new(),
        );

        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(match verifier
            .pw_verify(&mongo_client, &mut redis_client, Some(true))
            .await
        {
            Ok(val) => {
                let _ = mongo_conn.drop().await;
                val
            }
            Err(_err) => {
                false
            }
        });
    }
    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_login_verifier_with_long_password(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "a_very_long_password_that_exceeds_normal_length".to_string(),
            String::new(),
        );

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(match verifier
            .pw_verify(&mongo_client, &mut redis_client, Some(true))
            .await
        {
            Ok(val) => {
                let _ = mongo_conn.drop().await;
                val
            }
            Err(_err) => {
                false
            }
        });
    }
    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_passwords_with_spaces_and_tabs(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "   password_with_spaces_and_tabs\t".to_string(),
            String::new(),
        );

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(match verifier
            .pw_verify(&mongo_client, &mut redis_client, Some(true))
            .await
        {
            Ok(val) => {
                let _ = mongo_conn.drop().await;
                val
            }
            Err(_err) => {
                false
            }
        });
    }
    #[ignore]
    #[rstest]
    #[tokio::test]
    #[awt]
    async fn test_passwords_with_spaces_and_tabs_2(
        #[future] mongo_client: mongodb::Client,
        #[future] mut redis_client: aio::ConnectionManager,
    ) {
        let encrypted_pw: PassWorder = PassWorder::new("the_passwor").encrypt().salt().pepper();

        let mut verifier: users::Users = users::Users::new(
            "the_email".to_string(),
            "   password with spaces and tabs\t".to_string(),
            String::new(),
        );

        verifier.set_pw(&encrypted_pw.to_string());

        let mongo_conn: mongodb::Collection<users::Users> =
            mongo_client.database("personal_blog").collection("Test");

        let _ = mongo_conn.insert_one(&verifier).await;

        assert!(match verifier
            .pw_verify(&mongo_client, &mut redis_client, Some(true))
            .await
        {
            Ok(val) => {
                let _ = mongo_conn.drop().await;
                val
            }
            Err(_err) => {
                false
            }
        });
    }
}
