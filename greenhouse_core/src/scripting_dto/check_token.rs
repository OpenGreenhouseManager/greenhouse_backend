use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckTokenDtoRequest {
    pub token: String,
}
