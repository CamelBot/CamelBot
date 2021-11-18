// jkcoxson

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Intent {
    pub components: Vec<String>,
    pub messages: bool,
    pub events: Vec<String>,
    pub commands: Vec<Command>,
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_: String,
    pub description: String,
    pub required: bool,
    pub options: Vec<String>,
}

impl Intent {
    pub fn new(json: String) -> Intent {
        serde_json::from_str(&json).unwrap()
    }
}
