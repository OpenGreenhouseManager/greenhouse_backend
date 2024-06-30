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
    pub password: String,
    pub salt: String,
    pub role: String,
    pub login_session: String,
}

impl User {
    pub fn new(username: String, password: String, role: String, jws_secret: String) -> Self {
        let password_hash: bcrypt::HashParts = bcrypt::hash_with_result(password, 12).unwrap();
        let token = user_token::UserToken::generate_token(
            username.clone(),
            role.clone(),
            jws_secret.clone(),
        );

        Self {
            id: Uuid::new_v4(),
            username,
            password: password_hash.format_for_version(Version::TwoB),
            salt: password_hash.get_salt(),
            role,
            login_session: token,
        }
    }

    pub fn check_token() -> bool {
        todo!()
    }

    pub fn refresh_token(&self) -> String {
        user_token::UserToken::generate_token(
            self.username.clone(),
            self.role.clone(),
            self.login_session.clone(),
        )
    }

    pub async fn check_login(&self, password: String) -> bool {
        bcrypt::verify(password, &self.password).is_ok()
    }
}
