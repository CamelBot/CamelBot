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
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::tcp::{ReadHalf, WriteHalf},
    process::{ChildStdin, ChildStdout},
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};

use crate::{
    commands::{self, Command},
    packet::Packet,
};

pub struct Component {
    pub id: String,    // An ID that can be referenced by other components
    pub type_: u8,     // 0 - Interface, 1 - Plugin, 2 - Sniffer
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
            type_,
            network: if key == "" { false } else { true },
            key,
            sender,
            intents: Vec::new(),
            gucci: false,
        }
    }

    pub async fn connect(
        id: String,
        command: String,
        args: Vec<String>,
        components: Arc<Mutex<HashMap<String, Component>>>,
        commands: Arc<Mutex<Vec<Command>>>,
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
                    // TODO
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
                                println!("Failed to start component {}: {}", id, e);
                                return;
                            }
                        };

                        let stdin = cmd.stdin.take().unwrap();
                        let stdout = cmd.stdout.take().unwrap();
                        let stdout = BufReader::new(stdout);
                        let (rep, rec) = Component::run(
                            id.clone(),
                            stdout,
                            stdin,
                            cloned_components.clone(),
                            commands.clone(),
                            receiver,
                        )
                        .await;
                        if !rep {
                            break;
                        }
                        receiver = rec;
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
        id: String,
        reader: impl ComponentRead,
        writer: impl ComponentWrite,
        components: Arc<Mutex<HashMap<String, Component>>>,
        commands: Arc<Mutex<Vec<Command>>>,
        receiver: UnboundedReceiver<Packet>,
    ) -> (bool, UnboundedReceiver<Packet>) // Should the component be automatically restarted on exit
    {
        println!("{} has started", id);
        let mut reader = reader;
        let mut writer = writer;
        let mut receiver = receiver;

        // We are caching components to avoid bottleknecks and unecessary locks
        let mut component_cache = cache_components(components.clone()).await;

        loop {
            tokio::select! {
                msg = reader.read() => {
                    // Attempt to parse msg as JSON
                    let msg = match serde_json::from_str::<serde_json::Value>(&msg) {
                        Ok(msg) => msg,
                        _ => {
                            continue;
                        }
                    };

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
                            return(false, receiver);
                        }
                        "restart" => {
                            return(true, receiver);
                        }
                        "intent update" => {
                            component_cache = cache_components(components.clone()).await;
                        }
                        "command update" => {
                            writer.write(commands::create_packet(commands.lock().await)).await;
                        }
                        _ => {
                            // Write packet to component
                            writer.write(packet.data).await;
                        }
                    }

                }
            }
        }
    }
}

impl Clone for Component {
    fn clone(&self) -> Self {
        Component {
            id: self.id.clone(),
            type_: self.type_,
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
        todo!()
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
    async fn write(&mut self, _msg: String) {
        todo!()
    }
}
#[async_trait]
impl ComponentWrite for ChildStdin {
    async fn write(&mut self, msg: String) {
        self.write_all(msg.as_bytes()).await.unwrap();
        self.flush().await.unwrap();
    }
}
