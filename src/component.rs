// jkcoxson

use tokio::sync::mpsc::UnboundedSender;

struct Component {
    id: String,
    command: String,
    network: bool,
    key: String,
    sender: UnboundedSender<String>,
}
