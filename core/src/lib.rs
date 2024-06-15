use serde::{ Deserialize, Serialize };
pub mod error;
#[cfg(feature = "smart_device_dto")]
pub mod smart_device_dto;
#[cfg(feature = "smart_device_interface")]
pub mod smart_device_interface;

#[derive(Serialize, Deserialize, Debug)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

#[cfg(feature = "service_dto")]
impl Rectangle {
    pub fn circumference(&self) -> u32 {
        2 * (self.width + self.height)
    }
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}
