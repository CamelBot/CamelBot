// CamelBot - Horizon Edition (https://www.youtube.com/watch?v=8rCPtXVOsf8)
// jkcoxson
// All hail camels

use std::{collections::HashMap, process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncReadExt, BufReader, BufWriter},
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};

use crate::packet::Packet;

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

    // Create unbounded receiver and sender
    let (tx, mut interface_rx) = mpsc::unbounded_channel();
    let tx: UnboundedSender<Packet> = tx;

    // Create arc of UnboundedSender
    let component_arc: Arc<Mutex<HashMap<String, UnboundedSender<Packet>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Start interfaces
    for i in config.interfaces.iter() {
        let (local_tx, local_rx) = mpsc::unbounded_channel();
        if i.network {
            if !&config.tcp {
                println!("Interface {} is configured for network mode, but TCP mode is not enabled. It will not be loaded.", i.name);
                continue;
            }
            println!("Waiting for connection from {}", i.name);
        } else {
            //
        }
        // Put local_tx into the the component_arc
        let mut interface_arc = component_arc.lock().await;
        interface_arc.insert(i.name.clone(), local_tx);
    }

    if config.tcp {
        // Start TCP listener
        // Wait for connection
        // Get list of TCP components that aren't gucci
        // Receive key and compare to components
        // If they match, give the component the client
    }

    // Create channel for killing the program
    let (kill_tx, mut kill_rx) = mpsc::unbounded_channel();
    let kill_tx: UnboundedSender<()> = kill_tx;

    // UI loop yeet
    let cloned_tx = tx.clone();
    let cloned_arc = component_arc.clone();
    // This is now blocking to stop the program from exiting
    //ui::ui(cloned_arc, cloned_plugin_arc, kill_tx, cloned_tx).await;
}
