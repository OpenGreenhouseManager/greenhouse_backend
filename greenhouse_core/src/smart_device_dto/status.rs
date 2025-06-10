use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum DeviceStatusDto {
    Online,
    Panic,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceStatusResponseDto {
    pub status: DeviceStatusDto,
    pub datasource_id: String,
}
