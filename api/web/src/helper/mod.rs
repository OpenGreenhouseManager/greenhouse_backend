mod error;
pub use self::error::{Error, Result};

pub mod token {
    pub use super::error::{Error, Result};

    use greenhouse_core::auth_service_dto::user_token::UserToken;
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

    pub fn get_claims(token: String) -> Result<UserToken> {
        let key = DecodingKey::from_secret(&[]);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.insecure_disable_signature_validation();

        Ok(decode::<UserToken>(&token, &key, &validation)
            .map_err(|_| Error::InvalidToken)?
            .claims)
    }
}
