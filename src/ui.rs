// jkcoxson

use cursive::theme::{self};
use cursive::traits::Boxable;
use cursive::{CursiveExt, With};
use dialoguer::{theme::ColorfulTheme, Select};
use std::{collections::HashMap, convert::TryInto, sync::Arc};
use tokio::sync::Mutex;

use cursive::views::Dialog;
use cursive::Cursive;

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
    // Create the cursive TUI
    let mut siv = Cursive::default();

    // Chang color scheme to hacker theme
    let theme = siv.current_theme().clone().with(|theme| {
        theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Black);
        theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::Green);
        theme.palette[theme::PaletteColor::TitlePrimary] =
            theme::Color::Light(theme::BaseColor::Green);
        theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Green);
        theme.palette[theme::PaletteColor::Background] =
            theme::Color::Dark(theme::BaseColor::Black);
    });
    siv.set_theme(theme);

    // Bind 'q' to quit I guess. TODO make this a confirmation dialog
    siv.add_global_callback('q', |s| s.quit());

    siv.add_layer(
        Dialog::text("This is a survey!\nPress <Next> when you're ready.")
            .title("Important survey")
            .button("Next", show_next)
            .fixed_size(get_term_size()),
    );

    // Run I guess?
    siv.run();
}

fn show_next(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::text("Did you do the thing?")
            .title("Question 1")
            .button("Yes!", |s| show_answer(s, "I knew it! Well done!"))
            .button("No!", |s| show_answer(s, "I knew you couldn't be trusted!"))
            .button("Uh?", |s| s.add_layer(Dialog::info("Try again!")))
            .fixed_size(get_term_size()),
    );
}

fn show_answer(s: &mut Cursive, msg: &str) {
    s.pop_layer();
    s.add_layer(
        Dialog::text(msg)
            .title("Results")
            .button("Finish", |s| s.quit())
            .fixed_size(get_term_size()),
    );
}

fn get_term_size() -> (u16, u16) {
    termsize::get()
        .map(|size| return (size.cols - 2, size.rows - 2))
        .unwrap_or((80, 24))
}
