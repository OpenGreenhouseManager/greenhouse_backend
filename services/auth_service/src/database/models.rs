use crate::token;

use super::{Error, Result};
use bcrypt::Version;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

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
        let password_hash = bcrypt::hash_with_result(password, 12).map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(String::from(username)),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), username.into());
                map.insert(String::from("role"), role.into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::InvalidHash
        })?;

        Ok(Self {
            id: Uuid::new_v4(),
            username: String::from(username),
            hash: password_hash.format_for_version(Version::TwoB),
            role: String::from(role),
            login_session: String::from(""),
        })
    }

    pub fn refresh_token(&mut self, jws_secret: &str) -> Result<String> {
        let login_session =
            token::user_token::generate_token(&self.username, &self.role, jws_secret)?;

        self.login_session = login_session.clone();
        Ok(login_session)
    }

    pub async fn check_login(&self, password: &str) -> Result<bool> {
        bcrypt::verify(password, &self.hash).map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(self.username.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), self.username.clone().into());
                map.insert(String::from("role"), self.role.clone().into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });

            sentry::capture_error(&e);
            Error::InvalidHash
        })
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

    #[allow(clippy::needless_return)] // https://github.com/rust-lang/rust-clippy/issues/13458
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
