use super::{Error, Result};
use bcrypt::Version;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::user_token;

#[derive(Debug, Queryable, Selectable, Deserialize, Insertable)]
#[diesel(table_name = crate::database::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub login_session: String,
    hash: String,
}

impl User {
    pub fn new(username: &str, password: &str, role: &str) -> Result<Self> {
        let password_hash =
            bcrypt::hash_with_result(password, 12).map_err(|_| Error::InvalidHash)?;

        Ok(Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            hash: password_hash.format_for_version(Version::TwoB),
            role: role.to_string(),
            login_session: "".to_string(),
        })
    }

    pub fn refresh_token(&mut self, jws_secret: &str) -> Result<String> {
        let login_session =
            user_token::UserToken::generate_token(&self.username, &self.role, jws_secret)?;

        self.login_session = login_session.clone();
        Ok(login_session)
    }

    pub async fn check_login(&self, password: &str) -> Result<bool> {
        bcrypt::verify(password, &self.hash).map_err(|_| Error::InvalidHash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user() {
        let user = User::new("testUser1", "testPassword1", "test").expect("Failed to create user");

        assert_eq!(user.username, "testUser1");
        assert_eq!(user.role, "test");
    }

    #[tokio::test]
    async fn check_login() {
        let user = User::new("testUser1", "testPassword1", "test").expect("Failed to create user");

        assert!(user
            .check_login("testPassword1")
            .await
            .expect("Failed to check login"));
        assert!(!user
            .check_login("wrongPassword")
            .await
            .expect("Failed to check login"));
    }
}
