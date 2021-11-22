// jkcoxson
// https://horizon.fandom.com/wiki/Subordinate_Functions

// There can be 3 types of components:
// 0 - Interface: it communicates with chat servers
// 1 - Plugin: it reacts and sends events to interfaces
// 2 - Sniffer: it takes every packet before reaching its source and modifies it or drops it
// Note: a sniffer should not be used unless needed because it can be very slow

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::packet::Packet;

struct Component {
    id: String,    // An ID that can be referenced by other components
    type_: u8,     // 0 - Interface, 1 - Plugin, 2 - Sniffer
    network: bool, // Whether the component communicates over TCP. If false, it communicates over STDIN/STDOUT
    key: String,   // The key used to authenticate with the component if over TCP
    sender: UnboundedSender<Packet>,
}

impl Component {
    /// Creates a new component and starts it
    /// # Arguments
    /// * `id` - The ID of the component
    /// * `type_` - The type of the component
    /// * `network` - Whether the component communicates over TCP
    /// * `key` - The key used to authenticate with the component if over TCP
    /// * `sender` - The sender to send packets to the component
    /// * `receiver` - The receiver to receive packets from the component
    pub async fn new(
        id: String,
        type_: u8,
        key: String,
        sender: UnboundedSender<Packet>,
        receiver: UnboundedReceiver<Packet>,
    ) {
    }
}
