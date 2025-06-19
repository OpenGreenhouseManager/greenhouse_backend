use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PostDeviceDtoRequest {
    pub name: String,
    pub description: String,
    pub address: String,
    pub can_script: bool,
}
