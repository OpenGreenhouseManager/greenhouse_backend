pub(crate) mod error;

pub(crate) mod token {
    pub(crate) use super::error::{Error, Result};

    use greenhouse_core::auth_service_dto::user_token::UserToken;

    pub(crate) fn get_claims_dangerous(token: String) -> Result<UserToken> {
        let claims = jsonwebtoken::dangerous::insecure_decode(token)
            .map_err(|_| Error::InvalidToken)?
            .claims;
        Ok(claims)
    }
}
