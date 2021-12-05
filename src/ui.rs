// jkcoxson

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::theme::{self};
use cursive::traits::{Boxable, Nameable, Scrollable};
use cursive::{CursiveExt, With};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use cursive::views::{Dialog, EditView, OnEventView, SelectView, TextView};
use cursive::Cursive;

use crate::config::ComponentConstructor;
use crate::constants;
use crate::{commands::Command, component::Component, config, create_component, packet::Packet};

pub struct UI {
    pub messages: Vec<String>,
    pub mode: u8, // 0 = menu, 1 = log
    pub hacker_messages: Vec<String>,
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
            hacker_messages: Vec::new(),
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

pub fn tui(
    logger: Arc<std::sync::Mutex<UI>>,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    network_arc: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<TcpStream>>>>,
    config: config::Config,
) {
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
        theme.palette[theme::PaletteColor::Secondary] =
            theme::Color::Light(theme::BaseColor::Green);
    });
    siv.set_theme(theme);
    siv.set_fps(2);

    // Set log data
    siv.set_user_data(logger.clone());

    let data_pack = (
        logger.clone(),
        component_arc,
        command_arc,
        network_arc,
        config,
    );
    let esc_data_pack = data_pack.clone();
    let refresh_data_pack = data_pack.clone();
    let og_data_pack = data_pack.clone();

    siv.add_global_callback(cursive::event::Key::Esc, move |s| {
        let lock = s.user_data::<Arc<std::sync::Mutex<UI>>>().unwrap().clone();
        let mut log = lock.lock().unwrap();

        match log.mode {
            1 => {
                log.mode = 0;
                display_menu(
                    s,
                    Logger {
                        arc_reactor: esc_data_pack.0.clone(),
                        id: "core".to_string(),
                    },
                    esc_data_pack.1.clone(),
                    esc_data_pack.2.clone(),
                    esc_data_pack.3.clone(),
                    esc_data_pack.4.clone(),
                );
            }
            _ => {
                log.mode = 1;
                display_log(s, log.messages.clone());
            }
        }
    });
    siv.add_global_callback(cursive::event::Key::Backspace, move |s| {
        let lock = s.user_data::<Arc<std::sync::Mutex<UI>>>().unwrap().clone();
        let mut log = lock.lock().unwrap();
        log.mode = 2;
        display_log(s, vec!["Hacking the mainframe...".to_string()]);
    });
    siv.add_global_callback(cursive::event::Event::Refresh, |s| {
        let lock = s.user_data::<Arc<std::sync::Mutex<UI>>>().unwrap().clone();
        let log = lock.lock().unwrap();
        if log.mode == 1 {
            display_log(s, log.messages.clone());
        }
        if log.mode == 2 {
            display_log(s, log.hacker_messages.clone());
        }
    });
    siv.add_global_callback(cursive::event::Event::WindowResize, move |s| {
        let lock = s.user_data::<Arc<std::sync::Mutex<UI>>>().unwrap().clone();
        let log = lock.lock().unwrap();
        if log.mode == 1 {
            display_log(s, log.messages.clone());
        } else {
            display_menu(
                s,
                Logger {
                    arc_reactor: refresh_data_pack.0.clone(),
                    id: "core".to_string(),
                },
                refresh_data_pack.1.clone(),
                refresh_data_pack.2.clone(),
                refresh_data_pack.3.clone(),
                refresh_data_pack.4.clone(),
            );
        }
    });

    display_menu(
        &mut siv,
        Logger {
            arc_reactor: og_data_pack.0.clone(),
            id: "core".to_string(),
        },
        og_data_pack.1.clone(),
        og_data_pack.2.clone(),
        og_data_pack.3.clone(),
        og_data_pack.4,
    );
    siv.run();
}

fn display_menu(
    siv: &mut Cursive,
    logger: crate::ui::Logger,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    network_arc: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<TcpStream>>>>,
    config: config::Config,
) {
    siv.pop_layer();
    let (x_size, y_size) = get_term_size();

    let remove_arc = component_arc.clone();
    let reload_arc = component_arc.clone();

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
        .button("Load Component", move |s| {
            choose_component_type(
                s,
                logger.clone("core".to_string()),
                component_arc.clone(),
                command_arc.clone(),
                network_arc.clone(),
                config.clone(),
            )
        })
        .button("Remove Component", move |s| {
            choose_component_removal(s, remove_arc.clone())
        })
        .button("Reload Component", move |s| {
            choose_component_reload(s, reload_arc.clone())
        })
        .button("Exit", |s| s.quit())
        .fixed_size((x_size, y_size)),
    );
}
fn display_log(siv: &mut Cursive, messages: Vec<String>) {
    siv.pop_layer();
    // Only get the last logs the terminal will fit
    let mut to_print = Vec::new();
    let screen_y = get_term_size().1 - 2;
    for i in messages {
        to_print.push(i);
        if to_print.len() > screen_y.into() {
            to_print.remove(0);
        }
    }

    siv.add_layer(
        Dialog::around(TextView::new(to_print.join("\n")))
            .title("Log")
            .fixed_size(get_term_size()),
    );
}

// Component loading functions
fn choose_component_type(
    siv: &mut Cursive,
    logger: crate::ui::Logger,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    network_arc: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<TcpStream>>>>,
    config: config::Config,
) {
    siv.pop_layer();
    let pack1 = (
        logger.clone("core".to_string()),
        component_arc.clone(),
        command_arc.clone(),
        network_arc.clone(),
        config.clone(),
    );
    let pack2 = (
        logger.clone("core".to_string()),
        component_arc.clone(),
        command_arc.clone(),
        network_arc.clone(),
        config.clone(),
    );
    siv.add_layer(
        Dialog::text("What type of component would you like?")
            .title("Type")
            .button("Interface", move |s| {
                choose_name(
                    s,
                    0,
                    pack1.0.clone("core".to_string()),
                    pack1.1.clone(),
                    pack1.2.clone(),
                    pack1.3.clone(),
                    pack1.4.clone(),
                )
            })
            .button("Plugin", move |s| {
                choose_name(
                    s,
                    1,
                    pack2.0.clone("core".to_string()),
                    pack2.1.clone(),
                    pack2.2.clone(),
                    pack2.3.clone(),
                    pack2.4.clone(),
                )
            })
            .button("Sniffer", move |s| {
                choose_name(
                    s,
                    2,
                    logger.clone("core".to_string()),
                    component_arc.clone(),
                    command_arc.clone(),
                    network_arc.clone(),
                    config.clone(),
                )
            }),
    );
}

fn choose_name(
    siv: &mut Cursive,
    type_: u8,
    logger: crate::ui::Logger,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    network_arc: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<TcpStream>>>>,
    config: config::Config,
) {
    let pack = (
        logger.clone("core".to_string()),
        component_arc.clone(),
        command_arc.clone(),
        network_arc.clone(),
        config.clone(),
    );
    siv.pop_layer();
    siv.add_layer(
        Dialog::new()
            .title("Enter your name")
            // Padding is (left, right, top, bottom)
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    // Call `show_popup` when the user presses `Enter`
                    .on_submit(move |s, name| {
                        choose_command(
                            s,
                            type_,
                            name.to_string(),
                            pack.0.clone("core".to_string()),
                            pack.1.clone(),
                            pack.2.clone(),
                            pack.3.clone(),
                            pack.4.clone(),
                        )
                    })
                    // Give the `EditView` a name so we can refer to it later.
                    .with_name("name")
                    // Wrap this in a `ResizedView` with a fixed width.
                    // Do this _after_ `with_name` or the name will point to the
                    // `ResizedView` instead of `EditView`!
                    .fixed_width(20),
            )
            .button("Ok", move |s| {
                // This will run the given closure, *ONLY* if a view with the
                // correct type and the given name is found.
                let name = s
                    .call_on_name("name", |view: &mut EditView| {
                        // We can return content from the closure!
                        view.get_content()
                    })
                    .unwrap();

                // Run the next step
                choose_command(
                    s,
                    type_,
                    name.to_string(),
                    logger.clone("core".to_string()),
                    component_arc.clone(),
                    command_arc.clone(),
                    network_arc.clone(),
                    config.clone(),
                );
            }),
    );
}

fn choose_command(
    siv: &mut Cursive,
    type_: u8,
    name: String,
    logger: crate::ui::Logger,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    network_arc: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<TcpStream>>>>,
    config: config::Config,
) {
    siv.pop_layer();
    let pack = (
        name.clone(),
        logger.clone("core".to_string()),
        component_arc.clone(),
        command_arc.clone(),
        network_arc.clone(),
        config.clone(),
    );
    siv.add_layer(
        Dialog::new()
            .title("Enter the command to start the component")
            // Padding is (left, right, top, bottom)
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    // Call `show_popup` when the user presses `Enter`
                    .on_submit(move |s, command| {
                        collect_component_options(
                            s,
                            type_,
                            pack.0.clone(),
                            command.to_string(),
                            pack.1.clone("core".to_string()),
                            pack.2.clone(),
                            pack.3.clone(),
                            pack.4.clone(),
                            pack.5.clone(),
                        )
                    })
                    // Give the `EditView` a name so we can refer to it later.
                    .with_name("name")
                    // Wrap this in a `ResizedView` with a fixed width.
                    // Do this _after_ `with_name` or the name will point to the
                    // `ResizedView` instead of `EditView`!
                    .fixed_width(20),
            )
            .button("Ok", move |s| {
                // This will run the given closure, *ONLY* if a view with the
                // correct type and the given name is found.
                let command = s
                    .call_on_name("name", |view: &mut EditView| {
                        // We can return content from the closure!
                        view.get_content()
                    })
                    .unwrap();

                // Run the next step
                collect_component_options(
                    s,
                    type_,
                    name.clone(),
                    command.to_string(),
                    logger.clone("core".to_string()),
                    component_arc.clone(),
                    command_arc.clone(),
                    network_arc.clone(),
                    config.clone(),
                );
            }),
    );
}

fn collect_component_options(
    siv: &mut Cursive,
    type_: u8,
    name: String,
    command: String,
    logger: crate::ui::Logger,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
    command_arc: Arc<Mutex<Vec<Command>>>,
    network_arc: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<TcpStream>>>>,
    config: config::Config,
) {
    siv.pop_layer();
    let constructor = ComponentConstructor {
        network: false,
        command,
        name,
        type_,
        key: "".to_string(),
    };
    let pack = (
        logger.clone("core".to_string()),
        component_arc.clone(),
        command_arc.clone(),
        network_arc.clone(),
        config.clone(),
    );
    tokio::spawn(async move {
        create_component(&constructor, pack.0, pack.1, pack.2, pack.3, pack.4).await
    });
    display_menu(siv, logger, component_arc, command_arc, network_arc, config);
}

// Component removal functions
fn choose_component_removal(
    siv: &mut Cursive,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
) {
    let cloned_component_arc = component_arc.clone();

    let list = get_component_list(component_arc);
    if list.len() == 0 {
        siv.add_layer(Dialog::info("No components to remove"));
        return;
    }

    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump()
        .on_submit(move |s, choice: &str| {
            let cloned_component_arc = cloned_component_arc.clone();
            let choice = choice.to_string();
            tokio::spawn(async move {
                // Send kill the component
                let mut lock = cloned_component_arc.lock().await;
                match lock.get_mut(choice.as_str()).unwrap().sender.send(Packet {
                    source: "core".to_string(),
                    destination: "".to_string(),
                    event: "".to_string(),
                    data: "kill".to_string(),
                    sniffers: vec![],
                }) {
                    _ => {}
                }
            });
            s.pop_layer();
        });
    select.add_all_str(list);

    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        });

    siv.add_layer(
        Dialog::around(select.scrollable().fixed_size((20, 10)))
            .title("Which component would you like to remove?\n"),
    );
}

// Component reload functions
fn choose_component_reload(
    siv: &mut Cursive,
    component_arc: Arc<Mutex<HashMap<String, Component>>>,
) {
    let cloned_component_arc = component_arc.clone();

    let list = get_component_list(component_arc);
    if list.len() == 0 {
        siv.add_layer(Dialog::info("No components to reload"));
        return;
    }

    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump()
        .on_submit(move |s, choice: &str| {
            let cloned_component_arc = cloned_component_arc.clone();
            let choice = choice.to_string();
            tokio::spawn(async move {
                // Send kill the component
                let mut lock = cloned_component_arc.lock().await;
                match lock.get_mut(choice.as_str()).unwrap().sender.send(Packet {
                    source: "core".to_string(),
                    destination: "".to_string(),
                    event: "".to_string(),
                    data: "reload".to_string(),
                    sniffers: vec![],
                }) {
                    _ => {}
                }
            });
            s.pop_layer();
        });
    select.add_all_str(list);

    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        });

    siv.add_layer(
        Dialog::around(select.scrollable().fixed_size((20, 10)))
            .title("Which component would you like to reload?\n"),
    );
}

/// Wraps Tokio's mutex in a blocking function
/// I don't know if this is really stupid or not
/// Someone pls tell me if it's really stupid
fn get_component_list(component_arc: Arc<Mutex<HashMap<String, Component>>>) -> Vec<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let mut list = Vec::new();
        let component_arc = component_arc.lock().await;
        for (name, _) in component_arc.iter() {
            list.push(name.clone());
        }
        tx.send(list).unwrap();
    });
    rx.recv().unwrap()
}

fn get_term_size() -> (u16, u16) {
    termsize::get()
        .map(|size| return (size.cols - 2, size.rows - 2))
        .unwrap_or((80, 24))
}
