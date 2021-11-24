// CamelBot - Horizon Edition (https://www.youtube.com/watch?v=8rCPtXVOsf8)
// jkcoxson
// All hail camels

use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::component::Component;

mod component;
mod config;
mod constants;
mod intents;
mod interface;
mod packet;
mod plugin;
mod ui;

#[tokio::main]
async fn main() {
    println!("{}", constants::SPLASH_SCREEN);

    // Try to load the config file
    let config = match config::Config::load().await {
        Some(config) => config,
        None => {
            println!("Config file not found, generating a new one.");
            let conf = config::Config::new();
            conf.save().await;
            conf
        }
    };

    // Create arc of UnboundedSender for components
    let component_arc: Arc<Mutex<HashMap<String, Component>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Start componenents
    for i in config.interfaces.iter() {
        if i.network && !config.tcp {
            println!("Interface {} is configured for network mode, but TCP mode is not enabled. It will not be loaded.", i.name);
            continue;
        }
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut comp = component::Component::new(i.name.clone(), 0, i.key.clone(), tx);
        let command = i.command.split(" ").collect::<Vec<&str>>()[0];
        let args = i.command.split(" ").skip(1).collect::<Vec<&str>>();
        comp.connect(command, args, rx).await;
    }

    if config.tcp {
        // Start TCP listener
        // Wait for connection
        // Get list of TCP components that aren't gucci
        // Receive key and compare to components
        // If they match, give the component the client
    }

    // UI loop yeet
    // This is now blocking to stop the program from exiting
    //ui::ui(cloned_arc, cloned_plugin_arc, kill_tx, cloned_tx).await;
}
