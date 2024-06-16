use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize)]
pub enum DeviceStatusResponseDto {
    Online,
    Panic,
}
