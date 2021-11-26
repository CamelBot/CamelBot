// jkcoxson

use dialoguer::{theme::ColorfulTheme, Select};
use std::{collections::HashMap, convert::TryInto, sync::Arc};
use tokio::sync::Mutex;

use crate::{commands::Command, component::Component, config, create_interface, packet::Packet};

pub async fn ui(
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    config: config::Config,
) {
    println!("Initialization complete, all hail camels o7");
    loop {
        let component_arc = component_arc.clone();
        match main_menu().await.as_str() {
            "Add Component" => {
                let constructor = new_component().await;
                create_interface(
                    &constructor,
                    component_arc.clone(),
                    command_arc.clone(),
                    config.clone(),
                )
                .await;
            }
            "Remove Component" => {
                let target = choose_component(component_arc.clone()).await;
                // Send kill to component
                let mut lock = component_arc.lock().await;
                match lock.get_mut(&target).unwrap().sender.send(Packet {
                    source: "core".to_string(),
                    destination: "".to_string(),
                    event: "".to_string(),
                    data: "kill".to_string(),
                    sniffers: vec![],
                }) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Failed to send kill to {}", target);
                        println!("{}", e);
                    }
                }
            }
            "Reload Component" => {
                let target = choose_component(component_arc.clone()).await;
                // Send kill to component
                let mut lock = component_arc.lock().await;
                match lock.get_mut(&target).unwrap().sender.send(Packet {
                    source: "core".to_string(),
                    destination: "".to_string(),
                    event: "".to_string(),
                    data: "reload".to_string(),
                    sniffers: vec![],
                }) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Failed to send kill to {}", target);
                        println!("{}", e);
                    }
                }
            }
            "Exit" => {
                let lock = component_arc.lock().await;
                for (_, k) in lock.iter() {
                    match k.sender.send(Packet {
                        source: "core".to_string(),
                        destination: "".to_string(),
                        event: "".to_string(),
                        data: "kill".to_string(),
                        sniffers: vec![],
                    }) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Error sending kill packet to component");
                        }
                    }
                }
                break;
            }
            _ => {} // This will never happen
        }
    }
}

async fn main_menu() -> String {
    tokio::task::spawn_blocking(move || {
        let options = vec![
            "Add Component",
            "Remove Component",
            "Reload Component",
            "Exit",
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

async fn new_component() -> config::ComponentConstructor {
    tokio::task::spawn_blocking(move || {
        let type_options = vec!["Interface", "Plugin", "Sniffer"];
        let type_ = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose what type of component")
            .items(&type_options)
            .interact()
            .unwrap();
        let plugin_name = dialoguer::Input::<String>::new()
            .with_prompt("Component name")
            .interact()
            .unwrap();
        let command = dialoguer::Input::<String>::new()
            .with_prompt("Command to launch component")
            .interact()
            .unwrap();

        config::ComponentConstructor {
            name: plugin_name,
            command: command,
            type_: type_.try_into().unwrap(),
            network: false,
            key: "".to_string(),
        }
    })
    .await
    .unwrap()
}

async fn choose_component(components: Arc<Mutex<HashMap<String, Component>>>) -> String {
    let options = components
        .lock()
        .await
        .keys()
        .cloned()
        .collect::<Vec<String>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a component")
        .items(&options)
        .interact()
        .unwrap();
    options[selection].to_string()
}
