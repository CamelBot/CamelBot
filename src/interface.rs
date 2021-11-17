// jkcoxson

use std::process::Stdio;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
    process::{Child, ChildStdin, ChildStdout},
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::config::InterfaceConstructor;

pub struct Interface {
    network: bool,
    stream: Option<TcpStream>,
    stdin: Option<BufWriter<ChildStdin>>,
    stdout: Option<BufReader<ChildStdout>>,
    sender: UnboundedSender<String>,
    console_handle: Option<Child>,
    receiver: UnboundedReceiver<String>,
    constructor: InterfaceConstructor,
}

impl Interface {
    pub fn new_stdin(
        stdin: BufWriter<ChildStdin>,
        stdout: BufReader<ChildStdout>,
        handler: Child,
        sender: UnboundedSender<String>,
        receiver: UnboundedReceiver<String>,
        constructor: InterfaceConstructor,
    ) -> Interface {
        Interface {
            network: false,
            stream: None,
            stdin: Some(stdin),
            stdout: Some(stdout),
            console_handle: Some(handler),
            sender,
            receiver,
            constructor,
        }
    }
    pub fn new_network(
        stream: TcpStream,
        sender: UnboundedSender<String>,
        receiver: UnboundedReceiver<String>,
        constructor: InterfaceConstructor,
    ) -> Interface {
        Interface {
            network: true,
            stream: Some(stream),
            stdin: None,
            stdout: None,
            console_handle: None,
            sender,
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
                                match msg.as_str() {
                                    "kill" => break,
                                    "reload" => {
                                        // Kill old process
                                        self.console_handle.unwrap().kill().await.unwrap();

                                        // Spawn new process
                                        let command = self.constructor.command.split(" ").collect::<Vec<&str>>()[0];
                                        let args = self.constructor.command.split(" ").skip(1).collect::<Vec<&str>>();
                                        let mut cmd = match tokio::process::Command::new(command)
                                            .args(args)
                                            .stdout(Stdio::piped())
                                            .stdin(Stdio::piped())
                                            .spawn()
                                        {
                                            Ok(cmd) => cmd,
                                            Err(e) => {
                                                println!("Failed to start plugin {}: {}", self.constructor.name, e);
                                                self.console_handle = None;
                                                break;
                                            }
                                        };

                                        // Set values to new constructor
                                        self.stdin = Some(BufWriter::new(cmd.stdin.take().unwrap()));
                                        self.stdout = Some(BufReader::new(cmd.stdout.take().unwrap()));
                                        self.console_handle = Some(cmd);
                                        stdout = self.stdout.take().unwrap();
                                        stdin = self.stdin.take().unwrap();

                                    },
                                    _ => {
                                        stdin.write(msg.as_bytes()).await.unwrap();
                                    }
                                };
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
                        sender.send(buf.to_string()).unwrap();
                    }
                }
            }
            self.console_handle.unwrap().kill().await.unwrap();
            println!(
                "Interface '{}' destroyed, all hail camels",
                self.constructor.name
            );
        }
    }
}
