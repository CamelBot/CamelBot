// jkcoxson

use dialoguer::{theme::ColorfulTheme, Select};
use std::{collections::HashMap, convert::TryInto, sync::Arc};
use tokio::sync::Mutex;
use tui::text::Text;

use std::io;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph, Widget};
use tui::Terminal;

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
                let constructor = match constructor {
                    Some(constructor) => constructor,
                    None => {
                        println!("Canceled");
                        continue;
                    }
                };
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
                let target = match target {
                    Some(target) => target,
                    None => continue,
                };
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
                let target = match target {
                    Some(target) => target,
                    None => continue,
                };
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

async fn new_component() -> Option<config::ComponentConstructor> {
    tokio::task::spawn_blocking(move || {
        let type_options = vec!["Interface", "Plugin", "Sniffer", "Cancel"];
        let type_ = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose what type of component")
            .items(&type_options)
            .default(3)
            .interact()
            .unwrap();
        if type_ == 3 {
            return None;
        }
        let plugin_name = dialoguer::Input::<String>::new()
            .with_prompt("Component name")
            .interact()
            .unwrap();
        if plugin_name.len() < 2 {
            return None;
        }
        let command = dialoguer::Input::<String>::new()
            .with_prompt("Command to launch component")
            .interact()
            .unwrap();
        if command.len() < 2 {
            return None;
        }
        Some(config::ComponentConstructor {
            name: plugin_name,
            command: command,
            type_: type_.try_into().unwrap(),
            network: false,
            key: "".to_string(),
        })
    })
    .await
    .unwrap()
}

async fn choose_component(components: Arc<Mutex<HashMap<String, Component>>>) -> Option<String> {
    let options = components
        .lock()
        .await
        .keys()
        .cloned()
        .collect::<Vec<String>>();
    if options.len() == 0 {
        println!("No components to choose from");
        return None;
    }
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a component")
        .items(&options)
        .interact()
        .unwrap();
    Some(options[selection].to_string())
}

pub async fn tui() {
    let stdout = io::stdout().into_raw_mode().unwrap();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
            let block = Block::default().title("Block 2").borders(Borders::ALL);
            f.render_widget(block, chunks[1]);
        })
        .unwrap();
    loop {}
}
