use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub enum Mode {
    Input,
    Output,
    InputOutput,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    Number,
    String,
    Stream,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    address: String,
    port: u32,
    mode: Mode,
    input_type: Option<Type>,
    output_type: Option<Type>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    name: String,
    description: String,
    config: Config,
}
