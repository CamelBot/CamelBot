// CamelBot - Horizon Edition (https://www.youtube.com/watch?v=8rCPtXVOsf8)
// jkcoxson
// All hail camels

use commands::Command;
use config::ComponentConstructor;
use std::{collections::HashMap, sync::Arc};
use tokio::{fs::File, sync::Mutex};

use crate::{component::Component, packet::Packet};

mod commands;
mod component;
mod config;
mod constants;
mod packet;
mod ui;

#[tokio::main]
async fn main() {
    // Set up the logger
    let arc_reactor = Arc::new(std::sync::Mutex::new(ui::UI::new()));
    let logger = ui::Logger::new(arc_reactor.clone());

    // Try to load the config file
    let config = match config::Config::load().await {
        Some(config) => config,
        None => {
            logger.warn("Config file not found, generating a new one.");
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
    for i in config.components.iter() {
        create_component(
            i,
            logger.clone("core".to_string()),
            component_arc.clone(),
            command_arc.clone(),
            config.clone(),
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
    //ui::ui(component_arc.clone(), command_arc.clone(), config).await;
    ui::tui(
        arc_reactor,
        component_arc.clone(),
        command_arc.clone(),
        config,
    );
}

pub async fn create_component(
    i: &ComponentConstructor,
    logger: ui::Logger,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    config: config::Config,
) {
    if i.network && !config.tcp {
        logger.warn(format!("Interface {} is configured for network mode, but TCP mode is not enabled. It will not be loaded.", i.name).as_str());
        return;
    }

    // Create component
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let comp = component::Component::new(i.name.clone(), 0, i.key.clone(), tx);

    // Insert component into map
    component_arc.lock().await.insert(i.name.clone(), comp);
    // Notify each component of an update
    for j in component_arc.lock().await.values() {
        if j.id == i.name {
            continue;
        }
        match j.sender.send(Packet {
            source: "".to_string(),
            destination: "".to_string(),
            event: "".to_string(),
            data: "update".to_string(),
            sniffers: vec![],
        }) {
            _ => {} // Don't care
        }
    }

    // Start component
    let command = i.command.split(" ").collect::<Vec<&str>>()[0];
    let args = i.command.split(" ").skip(1).collect::<Vec<&str>>();
    let args: Vec<String> = args.iter().map(|x| x.to_string()).collect();

    component::Component::connect(
        i.name.clone(),
        logger.clone(i.name.clone()),
        command.to_string(),
        args,
        component_arc.clone(),
        command_arc.clone(),
        rx,
    )
    .await;
}
