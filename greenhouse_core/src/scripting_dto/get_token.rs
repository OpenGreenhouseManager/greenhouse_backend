use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTokenDtoResponse {
    pub token: String,
}
