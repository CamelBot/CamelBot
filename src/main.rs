// CamelBot - Horizon Edition (https://www.youtube.com/watch?v=8rCPtXVOsf8)
// jkcoxson
// All hail camels

use commands::Command;
use config::ComponentConstructor;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc::UnboundedSender, Mutex},
};

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

    // Network Arc
    let network_arc: Arc<Mutex<HashMap<String, UnboundedSender<TcpStream>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Start componenents
    for i in config.components.iter() {
        create_component(
            i,
            logger.clone("core".to_string()),
            component_arc.clone(),
            command_arc.clone(),
            network_arc.clone(),
            config.clone(),
        )
        .await;
    }

    if config.tcp {
        let host = config.host.clone();
        let port = config.port;
        let logger = logger.clone("core".to_string());
        tokio::spawn(async move {
            // Start TCP listener
            let listener = match tokio::net::TcpListener::bind(format!("{}:{}", &host, &port)).await
            {
                Ok(listener) => listener,
                Err(e) => {
                    logger.error(&format!("Failed to start TCP listener: {}", e));
                    return;
                }
            };
            logger.info(&format!("Listening on {}:{}", host, port));
            loop {
                // Wait for connection
                let (mut socket, _) = listener.accept().await.unwrap();
                socket.write(b"").await.unwrap();
                // Read the first line from socket
                let mut key = "".to_string();
                loop {
                    let mut buffer = [1];
                    let _ = socket.read(&mut buffer).await.unwrap();
                    let char = buffer[0] as char;
                    if char == '\n' {
                        break;
                    } else {
                        key.push(char);
                    }
                }

                // Compare key to waiting components
                logger.debug(&format!("Received connection with key {}", key));
                // If they match, give the component the client
            }
        });
    }

    // UI loop yeet
    // This is now blocking to stop the program from exiting
    ui::tui(
        arc_reactor,
        component_arc.clone(),
        command_arc.clone(),
        network_arc.clone(),
        config,
    );
}

pub async fn create_component(
    i: &ComponentConstructor,
    logger: ui::Logger,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    network_arc: Arc<Mutex<HashMap<String, UnboundedSender<TcpStream>>>>,
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
    let key = i.key.clone();

    component::Component::connect(
        i.name.clone(),
        logger.clone(i.name.clone()),
        command.to_string(),
        args,
        key,
        component_arc,
        command_arc,
        network_arc,
        rx,
    )
    .await;
}
