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
    pub plugins: Vec<PluginConstructor>,
    pub interfaces: Vec<InterfaceConstructor>,
}

#[derive(Serialize, Deserialize)]
pub struct PluginConstructor {
    pub network: bool,
    pub command: String,
    pub name: String,
    pub key: String,
}

#[derive(Serialize, Deserialize)]
pub struct InterfaceConstructor {
    pub network: bool,
    pub command: String,
    pub name: String,
    pub key: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            tcp: false,
            port: 0,
            host: "".to_string(),
            plugins: Vec::new(),
            interfaces: Vec::new(),
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
            plugins: self.plugins.clone(),
            interfaces: self.interfaces.clone(),
        }
    }
}

impl Clone for PluginConstructor {
    fn clone(&self) -> PluginConstructor {
        PluginConstructor {
            network: self.network,
            command: self.command.clone(),
            name: self.name.clone(),
            key: self.key.clone(),
        }
    }
}

impl Clone for InterfaceConstructor {
    fn clone(&self) -> InterfaceConstructor {
        InterfaceConstructor {
            network: self.network,
            command: self.command.clone(),
            name: self.name.clone(),
            key: self.key.clone(),
        }
    }
}
