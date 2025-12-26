use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscriptionKeysDto {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscriptionDto {
    pub endpoint: String,
    #[serde(rename = "expirationTime")]
    pub expiration_time: Option<i64>,
    pub keys: PushSubscriptionKeysDto,
}


