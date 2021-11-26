// jkcoxson

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub tcp: bool,
    pub port: u16,
    pub host: String,
    pub components: Vec<ComponentConstructor>,
}

#[derive(Serialize, Deserialize)]
pub struct ComponentConstructor {
    pub network: bool,
    pub command: String,
    pub name: String,
    pub type_: u8,
    pub key: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            tcp: false,
            port: 0,
            host: "".to_string(),
            components: Vec::new(),
        }
    }
    pub async fn save(&self) {
        let mut file = File::create("config.json").await.unwrap();
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        file.write_all(serialized.as_bytes()).await.unwrap();
    }
    pub async fn load() -> Option<Config> {
        // Check if the config file exists
        let mut file = match File::open("config.json").await {
            Ok(f) => f,
            Err(_) => return None,
        };
        let mut contents = String::new();
        file.read_to_string(&mut contents).await.unwrap();
        let config: Config = serde_json::from_str(&contents).unwrap();
        Some(config)
    }
}

impl Clone for Config {
    fn clone(&self) -> Config {
        Config {
            tcp: self.tcp,
            port: self.port,
            host: self.host.clone(),
            components: self.components.clone(),
        }
    }
}

impl Clone for ComponentConstructor {
    fn clone(&self) -> ComponentConstructor {
        ComponentConstructor {
            network: self.network,
            command: self.command.clone(),
            name: self.name.clone(),
            type_: self.type_,
            key: self.key.clone(),
        }
    }
}
