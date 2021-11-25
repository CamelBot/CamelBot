// jkcoxson
// Stores commands and their sources

use serde_json::Value;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Command {
    pub name: String,
    pub plugin: String,
    pub description: String,
    pub options: Vec<Option>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Option {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub choices: Vec<Choice>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Choice {
    pub name: String,
    pub value: Value,
}

/// Saves the current commands to a file for restart caching.
/// When the bot is restarted, the command cache will be used until components update their own plugins.
/// This is to prevent interfaces removing commands only to immediately replace them.
/// Example: https://discord.com/developers/docs/interactions/application-commands#registering-a-command
/// "There is a global rate limit of 200 application command creates per day, per guild"
pub async fn save_cache(commands: Vec<Command>) {
    let mut file = File::create("cache/commands.json").await.unwrap();
    let json = serde_json::to_string(&commands).unwrap();
    file.write_all(json.as_bytes()).await.unwrap();
}

pub async fn load_cache() -> Vec<Command> {
    let mut file = File::open("cache/commands.json").await.unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();
    serde_json::from_str(&contents).unwrap()
}
