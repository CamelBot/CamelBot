// jkcoxson
// https://horizon.fandom.com/wiki/Subordinate_Functions

// There can be 3 types of components:
// 0 - Interface: it communicates with chat servers
// 1 - Plugin: it reacts and sends events to interfaces
// 2 - Sniffer: it takes every packet before reaching its source and modifies it or drops it
// Note: a sniffer should not be used unless needed because it can be very slow

use async_trait::async_trait;
use serde_json;
use std::{collections::HashMap, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
    process::{ChildStdin, ChildStdout},
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};

use crate::{
    commands::{self, Command},
    packet::Packet,
    ui,
};

pub struct Component {
    pub id: String,         // An ID that can be referenced by other components
    pub component_type: u8, // 0 - Interface, 1 - Plugin, 2 - Sniffer
    pub network: bool, // Whether the component communicates over TCP. If false, it communicates over STDIN/STDOUT
    pub key: String,   // The key used to authenticate with the component if over TCP
    pub sender: UnboundedSender<Packet>,
    pub intents: Vec<String>, // The events that the component wants to receive
    pub gucci: bool,
}

impl Component {
    /// Creates a new component
    /// # Arguments
    /// * `id` - The ID of the component
    /// * `type_` - The type of the component: 0 - Interface, 1 - Plugin, 2 - Sniffer
    /// * `network` - Whether the component communicates over TCP
    /// * `key` - The key used to authenticate with the component if over TCP
    /// * `sender` - The sender to send packets to the component
    /// * `receiver` - The receiver to receive packets from the component
    pub fn new(id: String, type_: u8, key: String, sender: UnboundedSender<Packet>) -> Component {
        // Create the component
        Component {
            id,
            component_type: type_,
            network: if key == "" { false } else { true },
            key,
            sender,
            intents: Vec::new(),
            gucci: false,
        }
    }

    pub async fn connect(
        mut id: String,
        logger: ui::Logger,
        command: String,
        args: Vec<String>,
        key: String,
        components: Arc<Mutex<HashMap<String, Component>>>,
        commands: Arc<Mutex<Vec<Command>>>,
        network_arc: Arc<Mutex<HashMap<String, UnboundedSender<TcpStream>>>>,
        receiver: UnboundedReceiver<Packet>,
    ) {
        // Get command information
        let cloned_components = components.clone();
        let mut components = components.lock().await;
        let component = components.get_mut(&id).unwrap();
        let network = component.network;
        drop(components);

        tokio::spawn(async move {
            let mut receiver = receiver;
            match network {
                true => {
                    loop {
                        // Create channel to receive client from listener
                        let (sender, mut cli_rec) = unbounded_channel();
                        // Add key to network_arc
                        let mut network_arc = network_arc.lock().await;
                        network_arc.insert(key.clone(), sender);
                        drop(network_arc);

                        // Wait for client
                        let mut client = cli_rec.recv().await.unwrap();

                        // Take the halves of the client
                        let (read, write) = client.split();

                        // Run
                        if !Component::run(
                            &mut id,
                            logger.clone(logger.id.clone()),
                            read,
                            write,
                            cloned_components.clone(),
                            commands.clone(),
                            &mut receiver,
                        )
                        .await
                        {
                            // Remove self from components
                            let mut components = cloned_components.lock().await;
                            components.remove(&id);
                            // Notify other components of the change
                            for (_, v) in components.iter_mut() {
                                match v.sender.send(Packet {
                                    source: id.clone(),
                                    destination: "".to_string(),
                                    event: "".to_string(),
                                    data: "update".to_string(),
                                    sniffers: vec![],
                                }) {
                                    _ => {} // Don't care
                                }
                            }
                            break;
                        }
                    }
                }
                false => {
                    loop {
                        // Create new command
                        let mut cmd = match tokio::process::Command::new(command.clone())
                            .args(args.clone())
                            .stdout(Stdio::piped())
                            .stdin(Stdio::piped())
                            .spawn()
                        {
                            Ok(cmd) => cmd,
                            Err(e) => {
                                logger.error(
                                    format!("Failed to start component {}: {}", id, e).as_str(),
                                );
                                return;
                            }
                        };

                        let stdin = cmd.stdin.take().unwrap();
                        let stdout = cmd.stdout.take().unwrap();
                        let stdout = BufReader::new(stdout);
                        if !Component::run(
                            &mut id,
                            logger.clone(logger.id.clone()),
                            stdout,
                            stdin,
                            cloned_components.clone(),
                            commands.clone(),
                            &mut receiver,
                        )
                        .await
                        {
                            // Remove self from components
                            let mut components = cloned_components.lock().await;
                            components.remove(&id);
                            // Notify other components of the change
                            for (_, v) in components.iter_mut() {
                                match v.sender.send(Packet {
                                    source: id.clone(),
                                    destination: "".to_string(),
                                    event: "".to_string(),
                                    data: "update".to_string(),
                                    sniffers: vec![],
                                }) {
                                    _ => {} // Don't care
                                }
                            }
                            break;
                        }
                        cmd.kill().await.unwrap();
                        cmd.wait().await.unwrap();
                    }
                }
            }
        });
    }

    /// Runs the component chain
    /// # Arguments
    /// * `id` - The ID of the component
    /// * `reader` - The reader to read from the component
    /// * `writer` - The writer to write to the component
    /// * `components` - The components to send packets to
    /// * `receiver` - The receiver to receive packets from other components
    /// # Returns
    /// * `(bool, UnboundedReceiver<Packet>)` - Whether the component should continue running and the receiver to receive packets from other components
    pub async fn run(
        id: &mut String,
        logger: ui::Logger,
        reader: impl ComponentRead,
        writer: impl ComponentWrite,
        components: Arc<Mutex<HashMap<String, Component>>>,
        commands: Arc<Mutex<Vec<Command>>>,
        receiver: &mut UnboundedReceiver<Packet>,
    ) -> bool // Should the component be automatically restarted on exit
    {
        logger.info(format!("{} has started", id).as_str());
        let mut reader = reader;
        let mut writer = writer;
        let component_type = components
            .lock()
            .await
            .get(id)
            .unwrap_or_else(|| {
                logger.error("CamelBot has dropped the component thread due to an ID mismatch. Your component will crash and you will be unable to restart it. Have a nice day!");
                panic!("stupid CamelBot");
            })
            .component_type;

        // We are caching components to avoid bottleknecks and unecessary locks
        let mut component_cache = cache_components(components.clone()).await;

        loop {
            tokio::select! {
                msg = reader.read() => {
                    if msg.len() == 0 {
                        // Component has exited
                        return true;
                    }
                    let msg = msg.replace("type_", "type"); // TODO figure out a better way to do this
                    // Attempt to parse msg as JSON
                    let mut msg = match serde_json::from_str::<serde_json::Value>(&msg) {
                        Ok(msg) => msg,
                        _ => {
                            continue;
                        }
                    };
                    // Add source to msg
                    msg.as_object_mut().unwrap().insert("source".to_string(), id.clone().into());
                    let packet_type = match msg["type"].as_str() {
                        Some(type_) => type_,
                        _ => {
                            continue;
                        }
                    };
                    match packet_type {
                        "event" => {
                            // Get list of sniffers
                            let mut sniffers = vec![];
                            for (v, k) in component_cache.iter_mut() {
                                if k.component_type == 2 {
                                    sniffers.push(v.clone());
                                }
                            }
                            let to_send = crate::packet::Packet {
                                source: id.clone(),
                                destination: "".to_string(),
                                event: msg["event"].to_string(),
                                data: msg.to_string(),
                                sniffers: sniffers.clone(),
                            };

                            if sniffers.len() > 0 {
                                // Send packet to the first sniffer
                                let sniffer = sniffers.remove(0);
                                match component_cache.get_mut(&sniffer).unwrap().sender.send(to_send) {
                                    _ => {} // Don't care
                                }
                            } else {
                                // Broadcast the event to all components that want it
                                for (_, k) in component_cache.iter_mut() {
                                    if k.id.as_str() == id {
                                        continue;
                                    }
                                    if k.intents.contains(&msg["event"].as_str().unwrap().to_string()) {
                                        match k.sender.send(to_send.clone()) {
                                            Ok(_) => {},
                                            Err(e) => {
                                                logger.error(format!("Failed to send event {} to {}: {}", msg["event"], k.id, e).as_str());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "send" => {
                            // Get the destination
                            let destination = match msg["target"].as_str() {
                                Some(destination) => destination,
                                _ => {
                                    continue;
                                }
                            };

                            // Get the sniffers
                            let mut sniffers = vec![];
                            for (v, k) in component_cache.iter_mut() {
                                if k.component_type == 2 {
                                    sniffers.push(v.clone());
                                }
                            }
                            let to_send = crate::packet::Packet {
                                source: id.clone(),
                                destination: destination.to_string(),
                                event: "".to_string(),
                                data: msg.to_string(),
                                sniffers: sniffers.clone(),
                            };

                            if sniffers.len() > 0 {
                                // Send packet to the first sniffer
                                let sniffer = sniffers.remove(0);
                                match component_cache.get_mut(&sniffer).unwrap().sender.send(to_send) {
                                    _ => {} // Don't care
                                }
                            } else {
                                // Send the packet to the destination
                                match component_cache.get_mut(destination).unwrap().sender.send(to_send) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        logger.error(format!("Failed to send packet to {}: {}", destination, e).as_str());
                                    }
                                }
                            }
                        }
                        "sniffer" => {
                            // Reconstruct the packet
                            // Determine if there are any more sniffers to send to
                            // If there are, send the packet to the next sniffer
                            // If not, either broadcast the event or send the packet to the destination
                            todo!();
                        }
                        "intents" => {
                            // Get the events
                            let events: Vec<String> = match msg["events"].as_array() {
                                Some(events) => {
                                    let mut to_return: Vec<String> = vec![];
                                    for event in events {
                                        to_return.push(event.as_str().unwrap().to_string());
                                    }
                                    to_return
                                }
                                _ => {
                                    continue;
                                }
                            };

                            let mut found_commands: Vec<Command> = match msg["commands"].as_array() {
                                Some(cmds) => {
                                    let mut to_return: Vec<Command> = vec![];
                                    for i in cmds {
                                        to_return.push(match serde_json::from_value(i.clone()) {
                                            Ok(value) => Command {
                                                source: id.clone(),
                                                structure: value,
                                            },
                                            _ => {
                                                continue;
                                            }
                                        })
                                    }
                                    to_return
                                },
                                None => continue,
                            };
                            // Put the events in the arc
                            let mut lock = commands.lock().await;
                            lock.append(&mut found_commands);
                            drop(lock);
                            let mut lock = components.lock().await;
                            lock.get_mut(id).unwrap().intents = events;
                            drop(lock);
                            // Send an update packet to each component
                            for (_, v) in component_cache.iter_mut() {
                                match v.sender.send(Packet {
                                    source: id.clone(),
                                    destination: "".to_string(),
                                    event: "".to_string(),
                                    data: "update".to_string(),
                                    sniffers: vec![],
                                }) {
                                    _ => {} // Don't care
                                }
                            }
                            // Save the command cache
                            commands::save_cache(commands.lock().await.to_vec()).await;
                        }
                        "id" => {
                            // Get the id
                            let changed_id = match msg["id"].as_str() {
                                Some(id) => id,
                                _ => {
                                    continue;
                                }
                            };


                            // Get self from the cache
                            let component = component_cache.get_mut(id).unwrap().clone();
                            // Remove self from the cache
                            let mut lock = components.lock().await;
                            lock.remove(id);
                            // Add self to the cache
                            lock.insert(changed_id.to_string(), component);
                            *id = changed_id.to_string();

                            // Notify all components of the change
                            for (_, v) in component_cache.iter_mut() {
                                match v.sender.send(Packet {
                                    source: id.clone(),
                                    destination: "".to_string(),
                                    event: "".to_string(),
                                    data: "update".to_string(),
                                    sniffers: vec![],
                                }) {
                                    _ => {} // Don't care
                                }
                            }

                        }
                        "debug" => {
                            // Get the debug message
                            let debug_message = match msg["message"].as_str() {
                                Some(message) => message,
                                _ => {
                                    continue;
                                }
                            };
                            logger.debug(debug_message);
                        },
                        "error" => {
                            // Get the error message
                            let error_message = match msg["message"].as_str() {
                                Some(message) => message,
                                _ => {
                                    continue;
                                }
                            };
                            logger.error(error_message);
                        },
                        "warn" => {
                            // Get the warn message
                            let warn_message = match msg["message"].as_str() {
                                Some(message) => message,
                                _ => {
                                    continue;
                                }
                            };
                            logger.warn(warn_message);
                        },
                        "info" => {
                            // Get the info message
                            let info_message = match msg["message"].as_str() {
                                Some(message) => message,
                                _ => {
                                    continue;
                                }
                            };
                            logger.info(info_message);
                        },
                        _ => {
                            continue;
                        }
                    }

                }
                packet = receiver.recv() => {
                    let packet = match packet {
                        Some(packet) => packet,
                        None => {
                            continue;
                        }
                    };
                    match packet.data.as_str() {
                        "kill" => {
                            return false;
                        }
                        "reload" => {
                            return true;
                        }
                        "update" => {
                            component_cache = cache_components(components.clone()).await;
                            let lock = commands.lock().await;
                            let command_clone = lock.clone();
                            drop(lock);
                            writer.write(commands::create_packet(command_clone)).await;
                        }
                        _ => {
                            if component_type == 2 {
                                // We do be a sniffer
                                let mut sniffers = packet.sniffers.clone();
                                // Remove self from sniffers
                                let index = sniffers.iter().position(|x| x == id).unwrap();
                                sniffers.remove(index);

                                let to_send = crate::packet::SnifferPacket {
                                    type_: "sniffer".to_string(),
                                    source: packet.source,
                                    destination: packet.destination,
                                    event: packet.event,
                                    sniffers,
                                    packet: packet.data,
                                };
                                // Convert to JSON
                                let to_send = serde_json::to_string(&to_send).unwrap();
                                writer.write(to_send).await;

                            } else{
                                writer.write(packet.data).await;
                            }
                        }
                    }

                }
            }
        }
    }
    pub async fn kill(&self) {
        match self.sender.send(Packet {
            source: "".to_string(),
            destination: "".to_string(),
            event: "".to_string(),
            data: "kill".to_string(),
            sniffers: vec![],
        }) {
            _ => {} // Don't care
        }
    }
}

impl Clone for Component {
    fn clone(&self) -> Self {
        Component {
            id: self.id.clone(),
            component_type: self.component_type,
            network: self.network,
            key: self.key.clone(),
            sender: self.sender.clone(),
            intents: self.intents.clone(),
            gucci: self.gucci,
        }
    }
}

pub async fn cache_components(
    components: Arc<Mutex<HashMap<String, Component>>>,
) -> HashMap<String, Component> {
    let components = components.lock().await;
    let mut component_cache = HashMap::new();
    for (k, v) in components.iter() {
        component_cache.insert(k.clone(), v.clone());
    }
    drop(components);
    component_cache
}

#[async_trait]
pub trait ComponentRead {
    async fn read(&mut self) -> String;
}
#[async_trait]
impl ComponentRead for ReadHalf<'_> {
    async fn read(&mut self) -> String {
        let mut to_return = "".to_string();
        loop {
            let mut buffer = [1];
            let _ = match AsyncReadExt::read(&mut self, &mut buffer).await {
                Ok(n) => n,
                Err(_) => {
                    return "".to_string();
                }
            };
            let char = buffer[0] as char;
            if char == '\n' {
                break;
            } else {
                to_return.push(char);
                if to_return.len() > 10000 {
                    // We dead bro
                    return "".to_string();
                }
            }
        }
        to_return
    }
}
#[async_trait]
impl ComponentRead for BufReader<ChildStdout> {
    async fn read(&mut self) -> String {
        let mut buf = String::new();
        self.read_line(&mut buf).await.unwrap();
        buf
    }
}

#[async_trait]
pub trait ComponentWrite {
    async fn write(&mut self, msg: String);
}
#[async_trait]
impl ComponentWrite for WriteHalf<'_> {
    async fn write(&mut self, msg: String) {
        let msg = msg.replace("type_", "type");
        let msg = format!("{}\n", msg);
        AsyncWriteExt::write(&mut self, msg.as_bytes())
            .await
            .unwrap();
        self.flush().await.unwrap();
    }
}
#[async_trait]
impl ComponentWrite for ChildStdin {
    async fn write(&mut self, msg: String) {
        let msg = msg.replace("type_", "type");
        let msg = format!("{}\n", msg);
        self.write_all(msg.as_bytes()).await.unwrap();
        self.flush().await.unwrap();
    }
}
