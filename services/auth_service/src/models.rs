use bcrypt::Version;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::user_token;

#[derive(Debug, Queryable, Selectable, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    hash: String,
    pub role: String,
    pub login_session: String,
}

impl User {
    pub fn new(username: String, password: String, role: String) -> Self {
        let password_hash: bcrypt::HashParts = bcrypt::hash_with_result(password, 12).unwrap();

        Self {
            id: Uuid::new_v4(),
            username,
            hash: password_hash.format_for_version(Version::TwoB),
            role,
            login_session: "".to_string(),
        }
    }

    pub fn check_token(&self, jws_secret: String) -> bool {
        user_token::UserToken::check_token(self.login_session.clone(), jws_secret)
    }

    pub fn refresh_token(&mut self, jws_secret: String) -> String {
        let login_session = user_token::UserToken::generate_token(
            self.username.clone(),
            self.role.clone(),
            jws_secret,
        );

        self.login_session = login_session.clone();
        login_session
    }

    pub async fn check_login(&self, password: String) -> bool {
        bcrypt::verify(password, &self.hash).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SECRET: &str = "XOJ~uQ7&AlPVs?1tm~4bD5nU$~E$iI702st]l|im:p8uTTj+dZX,R_QFvx4`*{r";

    #[test]
    fn create_user() {
        let user = User::new(
            "testUser1".to_string(),
            "testPassword1".to_string(),
            "test".to_string(),
        );

        assert_eq!(user.username, "testUser1");
        assert_eq!(user.role, "test");
    }

    #[tokio::test]
    async fn check_login() {
        let user = User::new(
            "testUser1".to_string(),
            "testPassword1".to_string(),
            "test".to_string(),
        );

        assert!(user.check_login("testPassword1".to_string()).await);
        assert!(!user.check_login("wrongPassword".to_string()).await);
    }

    #[test]
    fn check_token() {
        let mut user = User::new(
            "testUser1".to_string(),
            "testPassword1".to_string(),
            "test".to_string(),
        );

        let _ = user.refresh_token(SECRET.to_string());
        assert!(user.check_token(SECRET.to_string()));
        assert!(!user.check_token("wrongSecret".to_string()));
    }
}
