pub use self::error::{Error, Result};

use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

mod error;
static THREE_HOUR: i64 = 60 * 60 * 3;

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    pub user_name: String,
    pub role: String,
}

impl UserToken {
    pub fn generate_token(user_name: String, role: String, secret: String) -> Result<String> {
        let now = match Utc::now().timestamp_nanos_opt() {
            Some(time) => time / 1_000_000_000,
            None => return Err(Error::InvalidTime),
        };
        let payload = UserToken {
            iat: now,
            exp: now + THREE_HOUR,
            user_name,
            role,
        };

        match jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(secret.as_bytes()),
        ) {
            Ok(token) => Ok(token),
            Err(_) => Err(Error::JwtEncode),
        }
    }

    pub fn get_claims(token: String, secret: String) -> Result<UserToken> {
        Ok(jsonwebtoken::decode::<UserToken>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| Error::JwtDecode)?
        .claims)
    }

    pub fn check_token(token: String, secret: String) -> bool {
        jsonwebtoken::decode::<UserToken>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SECRET: &str = "XOJ~uQ7&AlPVs?1tm~4bD5nU$~E$iI702st]l|im:p8uTTj+dZX,R_QFvx4`*{r";
    const EXPIRED_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MTk4NTQwNDYsImV4cCI6MTcxOTg2NDg0NiwidXNlcl9uYW1lIjoidGVzdFVzZXIxIiwicm9sZSI6InRlc3QifQ.mEg0pe-AxO3Cn3JnaXDenuN-ZR2BmVS310T_zi6z_M8";

    #[test]
    fn create_token() {
        let token = UserToken::generate_token(
            "testUser1".to_string(),
            "test".to_string(),
            SECRET.to_string(),
        )
        .unwrap();

        let token = jsonwebtoken::decode::<UserToken>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(SECRET.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();
        assert_eq!(token.claims.user_name, "testUser1");
        assert_eq!(token.claims.role, "test");
        assert_eq!(token.claims.exp - token.claims.iat, THREE_HOUR);
    }

    #[test]
    fn verify_token() {
        let token = UserToken::generate_token(
            "testUser1".to_string(),
            "test".to_string(),
            SECRET.to_string(),
        )
        .unwrap();
        assert!(UserToken::check_token(token, SECRET.to_string()));
    }

    #[test]
    fn verify_token_different_secret() {
        let token = UserToken::generate_token(
            "testUser1".to_string(),
            "test".to_string(),
            SECRET.to_string(),
        )
        .unwrap();
        assert!(!UserToken::check_token(token, "broken".to_string()));
    }

    #[test]
    fn expired_token() {
        assert!(!UserToken::check_token(
            EXPIRED_TOKEN.to_string(),
            SECRET.to_string()
        ));
    }

    #[test]
    fn get_claims() {
        let token = UserToken::generate_token(
            "testUser1".to_string(),
            "test".to_string(),
            SECRET.to_string(),
        )
        .unwrap();

        let claims = UserToken::get_claims(token, SECRET.to_string()).unwrap();

        assert_eq!(claims.user_name, "testUser1");
        assert_eq!(claims.role, "test");
    }
}
