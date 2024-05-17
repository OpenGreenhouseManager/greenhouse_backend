use serde::{Deserialize, Serialize};
pub mod error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn circumference(&self) -> u32 {
        2 * (self.width + self.height)
    }
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}
