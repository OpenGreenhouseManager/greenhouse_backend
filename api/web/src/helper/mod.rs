pub(crate) mod error;

pub(crate) mod token {
    pub(crate) use super::error::{Error, Result};

    use greenhouse_core::auth_service_dto::user_token::UserToken;
    use jsonwebtoken::{dangerous::insecure_decode, get_current_timestamp};

    // Same leeway as `jsonwebtoken::Validation::default()` used by auth_service.
    const EXP_LEEWAY_SECS: i64 = 60;

    // The signature is verified by auth_service; here we only read the claims.
    // `insecure_decode` skips all validation, so check expiry ourselves.
    pub(crate) fn get_claims(token: String) -> Result<UserToken> {
        let claims = insecure_decode::<UserToken>(&token)
            .map_err(|_| Error::InvalidToken)?
            .claims;

        if claims.exp < get_current_timestamp() as i64 - EXP_LEEWAY_SECS {
            return Err(Error::InvalidToken);
        }

        Ok(claims)
    }
}
