// jkcoxson
// https://horizon.fandom.com/wiki/Subordinate_Functions

// There can be 3 types of components:
// 0 - Interface: it communicates with chat servers
// 1 - Plugin: it reacts and sends events to interfaces
// 2 - Sniffer: it takes every packet before reaching its source and modifies it or drops it
// Note: a sniffer should not be used unless needed because it can be very slow

use std::process::Stdio;

use tokio::{
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
    process::{Child, ChildStdin, ChildStdout},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::{intents::Intent, packet::Packet};

pub struct Component {
    pub id: String,    // An ID that can be referenced by other components
    pub type_: u8,     // 0 - Interface, 1 - Plugin, 2 - Sniffer
    pub network: bool, // Whether the component communicates over TCP. If false, it communicates over STDIN/STDOUT
    pub key: String,   // The key used to authenticate with the component if over TCP
    pub sender: UnboundedSender<Packet>,
    pub intents: Vec<Intent>,
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

    pub async fn run(
        &mut self,
        reader: &impl ComponentRead,
        writer: &impl ComponentWrite,
        receiver: UnboundedReceiver<Packet>,
    ) -> bool // Should the component be automatically restarted on exit
    {
        //
        false
    }
    pub async fn connect(
        &mut self,
        command: &str,
        args: Vec<&str>,
        receiver: UnboundedReceiver<Packet>,
    ) {
        match self.network {
            true => {
                //
            }
            false => {
                loop {
                    // Create new command
                    let mut cmd = match tokio::process::Command::new(command)
                        .args(args.clone())
                        .stdout(Stdio::piped())
                        .stdin(Stdio::piped())
                        .spawn()
                    {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            println!("Failed to start component {}: {}", &self.id, e);
                            return;
                        }
                    };
                }
            }
        }
    }
}

pub trait ComponentRead {}
impl ComponentRead for ReadHalf<'_> {}
impl ComponentRead for ChildStdin {}

pub trait ComponentWrite {}
impl ComponentWrite for WriteHalf<'_> {}
impl ComponentWrite for ChildStdout {}
