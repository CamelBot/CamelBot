// jkcoxson

use serde::Serialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
    process::{Child, ChildStdin, ChildStdout},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::config::PluginConstructor;

pub struct Plugin {
    network: bool,
    stream: Option<TcpStream>,
    stdin: Option<BufWriter<ChildStdin>>,
    stdout: Option<BufReader<ChildStdout>>,
    sender: UnboundedSender<String>,
    console_handle: Option<Child>,
    receiver: UnboundedReceiver<String>,
    constructor: PluginConstructor,
}

#[derive(Serialize)]
struct PacketFormat {
    name: String,
    source: String,
    data: String,
}

impl Plugin {
    pub fn new_stdin(
        stdin: BufWriter<ChildStdin>,
        stdout: BufReader<ChildStdout>,
        sender: UnboundedSender<String>,
        console_handle: Child,
        receiver: UnboundedReceiver<String>,
        constructor: PluginConstructor,
    ) -> Plugin {
        Plugin {
            network: false,
            stream: None,
            stdin: Some(stdin),
            stdout: Some(stdout),
            sender,
            console_handle: Some(console_handle),
            receiver,
            constructor,
        }
    }
    pub fn new_network(
        stream: TcpStream,
        sender: UnboundedSender<String>,
        receiver: UnboundedReceiver<String>,
        constructor: PluginConstructor,
    ) -> Plugin {
        Plugin {
            network: true,
            stream: Some(stream),
            stdin: None,
            stdout: None,
            sender,
            console_handle: None,
            receiver,
            constructor,
        }
    }
    pub async fn handle(mut self) {
        let mut receiver = self.receiver;
        let sender = self.sender;
        if self.network {
            let stream = self.stream.unwrap();
            let mut stream = BufReader::new(stream);
            loop {
                let mut buf = String::new();
                tokio::select! {
                    msg = receiver.recv() => {
                        match msg {
                            Some(msg) => {
                                stream.write(msg.as_bytes()).await.unwrap();
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    buf = stream.read_line(&mut buf) => {
                        let msg = buf.unwrap();
                        sender.send(msg.to_string()).unwrap();
                    }

                }
            }
        } else {
            let mut stdout = self.stdout.take().unwrap();
            let mut stdin = self.stdin.take().unwrap();
            loop {
                let mut buf = String::new();
                tokio::select! {
                    msg = receiver.recv() => {
                        match msg {
                            Some(msg) => {
                                if msg == "kill" {
                                    break;
                                }
                                let msg = msg + "\n";
                                stdin.write(msg.as_bytes()).await.unwrap();
                                stdin.flush().await.unwrap();
                            }
                            None => {
                                //break;
                            }
                        }
                    }
                    _ = stdout.read_line(&mut buf) => {
                        if buf.to_string().len() < 1 {
                            break;
                        }
                        let msg = PacketFormat {
                            name: self.constructor.name.clone(),
                            source: "plugin".to_string(),
                            data: buf.to_string(),
                        };
                        let msg = serde_json::to_string(&msg).unwrap();
                        sender.send(msg).unwrap();
                    }
                }
            }
            self.console_handle.unwrap().kill().await.unwrap();
            println!(
                "Plugin '{}' destroyed, all hail camels",
                self.constructor.name
            );
        }
    }
}
