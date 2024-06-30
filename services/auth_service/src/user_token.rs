use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

static THREE_HOUR: i64 = 60 * 60 * 3;

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // userID
    pub user_name: String,
    pub role: String,
}

impl UserToken {
    pub fn generate_token(username: String, role: String, secret: String) -> String {
        let now = Utc::now().timestamp_nanos_opt().unwrap() / 1000000000;
        let payload = UserToken {
            iat: now,
            exp: now + THREE_HOUR,
            user_name: username,
            role: role,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }
}
