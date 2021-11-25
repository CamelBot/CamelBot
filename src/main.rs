// CamelBot - Horizon Edition (https://www.youtube.com/watch?v=8rCPtXVOsf8)
// jkcoxson
// All hail camels

use std::{collections::HashMap, sync::Arc};
use tokio::{fs::File, sync::Mutex};

use crate::{commands::Command, component::Component};

mod commands;
mod component;
mod config;
mod constants;
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

    // Component Arc
    let component_arc: Arc<Mutex<HashMap<String, Component>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Command Arc
    let file = File::open("./command_cache.json").await;
    let command_arc = match file {
        Ok(_) => Arc::new(Mutex::new(commands::load_cache().await)),
        Err(_) => Arc::new(Mutex::new(Vec::new())),
    };

    // Start componenents
    for i in config.interfaces.iter() {
        if i.network && !config.tcp {
            println!("Interface {} is configured for network mode, but TCP mode is not enabled. It will not be loaded.", i.name);
            continue;
        }

        // Create component
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let comp = component::Component::new(i.name.clone(), 0, i.key.clone(), tx);

        // Insert component into map
        component_arc.lock().await.insert(i.name.clone(), comp);

        // Start component
        let command = i.command.split(" ").collect::<Vec<&str>>()[0];
        let args = i.command.split(" ").skip(1).collect::<Vec<&str>>();
        let args: Vec<String> = args.iter().map(|x| x.to_string()).collect();

        component::Component::connect(
            i.name.clone(),
            command.to_string(),
            args,
            component_arc.clone(),
            rx,
        )
        .await;
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
    loop {}
}
