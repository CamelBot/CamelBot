// CamelBot - Rust edition
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
    let tx: UnboundedSender<String> = tx;

    // Create arc of UnboundedSender
    let interface_arc: Arc<Mutex<HashMap<String, UnboundedSender<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let plugin_arc: Arc<Mutex<HashMap<String, UnboundedSender<String>>>> =
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
            create_stdin_interface(i, tx.clone(), local_rx);
        }
        // Put local_tx into the interface_arc
        let mut interface_arc = interface_arc.lock().await;
        interface_arc.insert(i.name.clone(), local_tx);
    }

    // Start plugins
    for i in config.plugins.iter() {
        let (local_tx, local_rx) = mpsc::unbounded_channel();
        if i.network {
            if !&config.tcp {
                println!("Plugin {} is configured for network mode, but TCP mode is not enabled. It will not be loaded.", i.name);
                continue;
            }
            println!("Waiting for connection from {}", i.name);
        } else {
            create_stdin_plugin(i, tx.clone(), local_rx);
        }
        // Put local_tx into the interface_arc
        let mut plugin_arc = plugin_arc.lock().await;
        plugin_arc.insert(i.name.clone(), local_tx);
    }

    if config.tcp {
        let interface_arc = interface_arc.clone();
        let plugin_arc = plugin_arc.clone();
        let config = config.clone();
        println!("Starting TCP server...");
        tokio::spawn(async move {
            // Start TCP server
            let listener = tokio::net::TcpListener::bind((config.host, config.port))
                .await
                .unwrap();
            loop {
                let mut client = listener.accept().await.unwrap().0;
                let mut buf = [0; 1024];
                let size = client.read(&mut buf).await.unwrap();

                // Check if the message is a key
                let key = String::from_utf8(buf[..size].to_vec()).unwrap();
                config.interfaces.iter().for_each(|i| {
                    if i.name == key {
                        todo!();
                    }
                });
                config.plugins.iter().for_each(|i| {
                    if i.name == key {
                        todo!();
                    }
                });
            }
        });
    }

    // Create channel for killing the program
    let (kill_tx, mut kill_rx) = mpsc::unbounded_channel();
    let kill_tx: UnboundedSender<()> = kill_tx;

    //UI loop yeet
    let cloned_tx = tx.clone();
    let cloned_arc = interface_arc.clone();
    let cloned_plugin_arc = plugin_arc.clone();
    tokio::spawn(async move {
        println!("Starting UI...");
        ui::ui(cloned_arc, cloned_plugin_arc, kill_tx, cloned_tx).await;
    });

    // Keep track of plugin intents
    let mut plugin_intents: HashMap<String, intents::Intent> = HashMap::new();

    loop {
        tokio::select! {
            msg = interface_rx.recv() => {
                // Try to parse the message as JSON
                let msg = match serde_json::from_str::<serde_json::Value>(&msg.unwrap()) {
                    Ok(msg) => msg,
                    Err(_) => continue
                };
                let source = msg["source"].as_str().unwrap();
                let name = msg["name"].as_str().unwrap();
                let data = msg["data"].as_str().unwrap();

                // Parse data as JSON
                let data = match serde_json::from_str::<serde_json::Value>(data) {
                    Ok(data) => data,
                    Err(_) => continue
                };

                match data["packet"].as_str() {
                    Some(packet) => {
                        match packet {
                            "message" => {
                                match msg["source"].as_str().unwrap() {
                                    "interface" => {
                                        // Check if the plugin has the message intent
                                        for (plugin_name, intent) in plugin_intents.iter() {
                                            if intent.messages {
                                                let mut plugin_arc = plugin_arc.lock().await;
                                                let plugin_tx = plugin_arc.get_mut(plugin_name).unwrap();
                                                plugin_tx.send(msg["data"].as_str().unwrap().to_string()).unwrap();
                                            }
                                        }
                                    },
                                    _ => continue
                                }
                            },
                            "sendMessage" => {
                                if msg["source"].as_str().unwrap() != "plugin" {
                                    continue;
                                }
                                let data = &msg["data"].as_str().unwrap();
                                // Parse data as JSON
                                let data = match serde_json::from_str::<serde_json::Value>(data) {
                                    Ok(data) => data,
                                    Err(_) => continue
                                };
                                let target = data["target"].as_str().unwrap();
                                // Get the target interface
                                let mut interface_arc = interface_arc.lock().await;
                                let interface_tx = match interface_arc.get_mut(target){
                                    Some(interface_tx) => interface_tx,
                                    None => continue
                                };
                                // Send the message to the interface
                                interface_tx.send(msg["data"].as_str().unwrap().to_string()).unwrap();
                            }
                            "intents" => {
                                // Check if source is a plugin
                                if msg["source"].as_str().unwrap() != "plugin" {
                                    continue;
                                }
                                // Create a new intent
                                let intent = intents::Intent::new(msg["data"].as_str().unwrap().to_string());
                                // Put the intent into the map
                                plugin_intents.insert(name.to_string(), intent);
                            }
                            _ => continue
                        }
                    },
                    None => continue,
                }

            }
            _ = kill_rx.recv() => {
                println!("Exiting CamelBot...");
                // Send kill message to all interfaces
                let mut interface_arc = interface_arc.lock().await;
                for (_, tx) in interface_arc.iter_mut() {
                    tx.send("kill".to_string()).unwrap();
                }
                // Send kill message to all plugins
                let mut plugin_arc = plugin_arc.lock().await;
                for (_, tx) in plugin_arc.iter_mut() {
                    tx.send("kill".to_string()).unwrap();
                }
                break;
            }
        }
    }
}

pub fn create_stdin_interface(
    i: &config::InterfaceConstructor,
    tx: UnboundedSender<String>,
    local_rx: UnboundedReceiver<String>,
) {
    println!("Starting interface {}...", i.name);
    // Start a new process with i.command
    let command = i.command.split(" ").collect::<Vec<&str>>()[0];
    let args = i.command.split(" ").skip(1).collect::<Vec<&str>>();
    let mut cmd = match tokio::process::Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(cmd) => cmd,
        Err(e) => {
            println!("Failed to start interface {}: {}", i.name, e);
            return;
        }
    };
    // Put cmd into a bufstream
    let stdout = BufReader::new(cmd.stdout.take().unwrap());
    let stdin = BufWriter::new(cmd.stdin.take().unwrap());

    let interface =
        interface::Interface::new_stdin(stdin, stdout, cmd, tx.clone(), local_rx, i.clone());

    tokio::spawn(async move {
        interface.handle().await;
    });
}

pub fn create_stdin_plugin(
    i: &config::PluginConstructor,
    tx: UnboundedSender<String>,
    local_rx: UnboundedReceiver<String>,
) {
    println!("Starting plugin {}...", i.name);
    // Start a new process with i.command
    let command = i.command.split(" ").collect::<Vec<&str>>()[0];
    let args = i.command.split(" ").skip(1).collect::<Vec<&str>>();
    let mut cmd = match tokio::process::Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(cmd) => cmd,
        Err(e) => {
            println!("Failed to start plugin {}: {}", i.name, e);
            return;
        }
    };
    // Put cmd into a bufstream
    let stdout = BufReader::new(cmd.stdout.take().unwrap());
    let stdin = BufWriter::new(cmd.stdin.take().unwrap());

    let plugin = plugin::Plugin::new_stdin(stdin, stdout, tx.clone(), cmd, local_rx, i.clone());

    tokio::spawn(async move {
        plugin.handle().await;
    });
}
