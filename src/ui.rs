// jkcoxson

use cursive::theme::{self};
use cursive::traits::Boxable;
use cursive::{CursiveExt, With};
use dialoguer::{theme::ColorfulTheme, Select};
use std::{collections::HashMap, convert::TryInto, sync::Arc};
use tokio::sync::Mutex;

use cursive::views::{Dialog, TextView};
use cursive::Cursive;

use crate::constants;
use crate::{commands::Command, component::Component, config, create_interface, packet::Packet};

pub struct UI {
    pub messages: Vec<String>,
    pub mode: u8, // 0 = menu, 1 = log
}

pub struct Logger {
    pub arc_reactor: Arc<std::sync::Mutex<UI>>,
    pub id: String,
}

impl UI {
    pub fn new() -> Self {
        UI {
            messages: Vec::new(),
            mode: 0,
        }
    }
}

impl Logger {
    pub fn new(arc: Arc<std::sync::Mutex<UI>>) -> Self {
        Logger {
            arc_reactor: arc,
            id: "core".to_string(),
        }
    }
    pub fn clone(&self, id: String) -> Self {
        Logger {
            arc_reactor: self.arc_reactor.clone(),
            id: id.to_string(),
        }
    }
    pub fn debug(&self, message: &str) {
        let mut ui = self.arc_reactor.lock().unwrap();
        ui.messages
            .push(format!("DEBUG: [{}] {}", self.id, message));
    }
    pub fn info(&self, message: &str) {
        let mut ui = self.arc_reactor.lock().unwrap();
        ui.messages.push(format!("INFO: [{}] {}", self.id, message));
    }
    pub fn warn(&self, message: &str) {
        let mut ui = self.arc_reactor.lock().unwrap();
        ui.messages.push(format!("WARN: [{}] {}", self.id, message));
    }
    pub fn error(&self, message: &str) {
        let mut ui = self.arc_reactor.lock().unwrap();
        ui.messages
            .push(format!("ERROR: [{}] {}", self.id, message));
    }
}

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
                // create_interface(
                //     &constructor,
                //     component_arc.clone(),
                //     command_arc.clone(),
                //     config.clone(),
                // )
                // .await;
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

pub async fn tui(logger: Arc<std::sync::Mutex<UI>>) {
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
    siv.set_fps(1);

    // Set log data
    siv.set_user_data(logger);

    siv.add_global_callback(cursive::event::Key::Esc, |s| {
        let lock = s.user_data::<Arc<std::sync::Mutex<UI>>>().unwrap().clone();
        let mut log = lock.lock().unwrap();
        if log.mode == 0 {
            log.mode = 1;
            display_log(s, log.messages.clone());
        } else {
            log.mode = 0;
            display_menu(s);
        }
    });
    siv.add_global_callback(cursive::event::Event::Refresh, |s| {
        let lock = s.user_data::<Arc<std::sync::Mutex<UI>>>().unwrap().clone();
        let log = lock.lock().unwrap();
        if log.mode == 1 {
            display_log(s, log.messages.clone());
        }
    });
    siv.add_global_callback(cursive::event::Event::WindowResize, |s| {
        let lock = s.user_data::<Arc<std::sync::Mutex<UI>>>().unwrap().clone();
        let log = lock.lock().unwrap();
        if log.mode == 1 {
            display_log(s, log.messages.clone());
        } else {
            display_menu(s);
        }
    });

    display_menu(&mut siv);

    siv.run();
}

fn display_menu(siv: &mut Cursive) {
    siv.pop_layer();
    let (x_size, y_size) = get_term_size();

    siv.add_layer(
        Dialog::around(Dialog::text(format!(
            "Select an option\n{}",
            if x_size > 109 {
                constants::SPLASH_SCREEN
            } else {
                constants::SMOL_CAMEL
            }
        )))
        .title("CamelBot Menu")
        .button("Load Component", |_| {})
        .button("Remove Component", |_| {})
        .button("Reload Component", |_| {})
        .button("Exit", |s| s.quit())
        .fixed_size((x_size, y_size)),
    );
}
fn display_log(siv: &mut Cursive, messages: Vec<String>) {
    siv.pop_layer();
    siv.add_layer(
        Dialog::around(TextView::new(messages.join("\n")))
            .title("Log")
            .fixed_size(get_term_size()),
    );
}

fn show_next(s: &mut Cursive) {
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
            .button("Finish", |s| {
                s.pop_layer();
            })
            .fixed_size(get_term_size()),
    );
}

fn get_term_size() -> (u16, u16) {
    termsize::get()
        .map(|size| return (size.cols - 2, size.rows - 2))
        .unwrap_or((80, 24))
}
