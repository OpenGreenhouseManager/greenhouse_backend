pub use self::error::{Error, Result};

use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

mod error;
static THREE_HOUR: i64 = 60 * 60 * 3;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    pub user_name: String,
    pub role: String,
}

impl UserToken {
    pub fn generate_token(user_name: &str, role: &str, secret: &str) -> Result<String> {
        let now = Utc::now().timestamp_nanos_opt().ok_or(Error::InvalidTime)? / 1_000_000_000;
        let payload = UserToken {
            iat: now,
            exp: now + THREE_HOUR,
            user_name: String::from(user_name),
            role: String::from(role),
        };

        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|_| Error::JwtEncode)
    }

    pub fn get_claims(token: &str, secret: &str) -> Result<UserToken> {
        Ok(jsonwebtoken::decode::<UserToken>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| Error::JwtDecode)?
        .claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SECRET: &str = "XOJ~uQ7&AlPVs?1tm~4bD5nU$~E$iI702st]l|im:p8uTTj+dZX,R_QFvx4`*{r";
    const EXPIRED_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MTk4NTQwANDYsImV4cCI6MTcxOTg2NDg0NiwidXNlcl9uYW1lIjoidGVzdFVzZXIxIiwicm9sZSI6InRlc3QifQ.mEg0pe-AxO3Cn3JnaXDenuN-ZR2BmVS310T_zi6z_M8";

    #[test]
    fn create_token() {
        let token = UserToken::generate_token("testUser1", "test", SECRET).unwrap();

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
        let token = UserToken::generate_token("testUser1", "test", SECRET).unwrap();
        UserToken::get_claims(&token, SECRET).unwrap();
    }

    #[test]
    fn verify_token_different_secret() {
        let token = UserToken::generate_token("testUser1", "test", SECRET).unwrap();

        assert_eq!(
            UserToken::get_claims(&token, "broken").unwrap_err(),
            (Error::JwtDecode)
        );
    }

    #[test]
    fn expired_get_claims() {
        assert_eq!(
            UserToken::get_claims(EXPIRED_TOKEN, SECRET).unwrap_err(),
            (Error::JwtDecode)
        );
    }

    #[test]
    fn get_claims() {
        let token = UserToken::generate_token("testUser1", "test", SECRET).unwrap();

        let claims = UserToken::get_claims(&token, SECRET).unwrap();

        assert_eq!(claims.user_name, "testUser1");
        assert_eq!(claims.role, "test");
    }
}
