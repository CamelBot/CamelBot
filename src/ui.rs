// jkcoxson

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use ansi_term::Style;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::config;

pub async fn ui(
    interface_arc: Arc<Mutex<HashMap<String, UnboundedSender<String>>>>,
    plugin_arc: Arc<Mutex<HashMap<String, UnboundedSender<String>>>>,
    kill_switch: UnboundedSender<()>,
    tx: UnboundedSender<String>,
) {
    println!("Initialization complete, all hail camels o7");
    loop {
        let interface_arc = interface_arc.clone();
        match main_menu().await.as_str() {
            "Interfaces" => {
                let option = interface_menu().await;
                match option.as_str() {
                    "Add Interface" => {
                        let interface = new_interface().await;
                        let (local_tx, local_rx) = tokio::sync::mpsc::unbounded_channel();
                        //create_stdin_interface(&interface, tx.clone(), local_rx);
                        interface_arc.lock().await.insert(interface.name, local_tx);
                    }
                    "Remove Interface" => {
                        let interface = choose_option(interface_arc.clone()).await;
                        interface_arc
                            .lock()
                            .await
                            .get(&interface)
                            .unwrap()
                            .send(String::from("kill"))
                            .unwrap();
                        interface_arc.lock().await.remove(&interface);
                    }
                    "Reload Interface" => {
                        let interface = choose_option(interface_arc.clone()).await;
                        interface_arc
                            .lock()
                            .await
                            .get(&interface)
                            .unwrap()
                            .send(String::from("reload"))
                            .unwrap();
                    }
                    "List Interfaces" => {
                        println!("{}", Style::new().underline().bold().paint("\nInterfaces"));
                        list(interface_arc.clone()).await;
                        println!("\n");
                    }
                    _ => {}
                }
            }
            "Plugins" => {
                let option = plugin_menu().await;
                match option.as_str() {
                    "Add Plugin" => {
                        let plugin = new_plugin().await;
                        let (local_tx, local_rx) = tokio::sync::mpsc::unbounded_channel();
                        //create_stdin_plugin(&plugin, tx.clone(), local_rx);
                        plugin_arc.lock().await.insert(plugin.name, local_tx);
                    }
                    "Remove Plugin" => {
                        let plugin = choose_option(plugin_arc.clone()).await;
                        plugin_arc.lock().await.remove(&plugin);
                    }
                    "Reload Plugin" => {
                        let plugin = choose_option(plugin_arc.clone()).await;
                        plugin_arc
                            .lock()
                            .await
                            .get(&plugin)
                            .unwrap()
                            .send(String::from("reload"))
                            .unwrap();
                    }
                    "List Plugins" => {
                        println!("{}", Style::new().underline().bold().paint("\nPlugins"));
                        list(plugin_arc.clone()).await;
                        println!("\n");
                    }
                    _ => {}
                }
            }
            "Exit" => {
                kill_switch.send(()).unwrap(); // o7
                break;
            }
            _ => {} // This will never happen
        }
    }
}

async fn main_menu() -> String {
    tokio::task::spawn_blocking(move || {
        let options = vec!["Interfaces", "Plugins", "Exit"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an option")
            .items(&options)
            .interact()
            .unwrap();
        options[selection].to_string()
    })
    .await
    .unwrap()
}

async fn interface_menu() -> String {
    tokio::task::spawn_blocking(move || {
        let options = vec![
            "Add Interface",
            "Remove Interface",
            "Reload Interface",
            "List Interfaces",
            "Back",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an option")
            .items(&options)
            .interact()
            .unwrap();
        options[selection].to_string()
    })
    .await
    .unwrap()
}

async fn plugin_menu() -> String {
    tokio::task::spawn_blocking(move || {
        let options = vec![
            "Add Plugin",
            "Remove Plugin",
            "Reload Plugin",
            "List Plugins",
            "Back",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an option")
            .items(&options)
            .interact()
            .unwrap();
        options[selection].to_string()
    })
    .await
    .unwrap()
}

async fn list(arc: Arc<Mutex<HashMap<String, UnboundedSender<String>>>>) {
    let map = arc.lock().await;
    let mut keys: Vec<String> = map.keys().map(|x| x.to_string()).collect();
    keys.sort();
    for key in keys {
        println!("• {}", key);
    }
}

async fn new_interface() -> config::InterfaceConstructor {
    tokio::task::spawn_blocking(move || {
        let interface_name = dialoguer::Input::<String>::new()
            .with_prompt("Interface name")
            .interact()
            .unwrap();
        let command = dialoguer::Input::<String>::new()
            .with_prompt("Command to launch interface")
            .interact()
            .unwrap();
        config::InterfaceConstructor {
            name: interface_name,
            command: command,
            network: false,
            key: "".to_string(),
        }
    })
    .await
    .unwrap()
}

async fn new_plugin() -> config::PluginConstructor {
    tokio::task::spawn_blocking(move || {
        let plugin_name = dialoguer::Input::<String>::new()
            .with_prompt("Plugin name")
            .interact()
            .unwrap();
        let command = dialoguer::Input::<String>::new()
            .with_prompt("Command to launch plugin")
            .interact()
            .unwrap();
        config::PluginConstructor {
            name: plugin_name,
            command: command,
            network: false,
            key: "".to_string(),
        }
    })
    .await
    .unwrap()
}

async fn choose_option(arc: Arc<Mutex<HashMap<String, UnboundedSender<String>>>>) -> String {
    let map = arc.lock().await;
    let mut keys: Vec<String> = map.keys().map(|x| x.to_string()).collect();
    keys.sort();
    tokio::task::spawn_blocking(move || {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your target")
            .items(&keys)
            .interact()
            .unwrap();
        keys[selection].to_string()
    })
    .await
    .unwrap()
}
