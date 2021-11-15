use std::io::{BufRead, BufReader, Write};

use lunatic::{net::TcpStream, process::Process, Mailbox, Request};
use serde::{Deserialize, Serialize};

use crate::coordinator::{CoordinatorRequest, CoordinatorResponse};

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    JoinRoom,
    LeaveRoom,
    Drop(u128),
}

#[derive(Serialize, Deserialize)]
pub struct ClientContext {
    stream: TcpStream,
    coordinator_proc: Process<Request<CoordinatorRequest, CoordinatorResponse>>,
}

impl ClientContext {
    pub fn new(
        stream: TcpStream,
        coordinator_proc: Process<Request<CoordinatorRequest, CoordinatorResponse>>,
    ) -> Self {
        Self {
            stream,
            coordinator_proc,
        }
    }
}

pub fn client_process(context: ClientContext, mailbox: Mailbox<ClientMessage>) {
    let mut stream = context.stream;
    let coordinator = context.coordinator_proc;

    println!("client process created for stream: {:?}", stream);

    let username = if let CoordinatorResponse::ServerJoined(username) =
        coordinator.request(CoordinatorRequest::JoinServer).unwrap()
    {
        username
    } else {
        unreachable!()
    };

    println!("generated username: {:?}", username);

    stream.write((username + "\n").as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut reader = BufReader::new(&mut stream);
    let mut buffer = String::new();

    while let Ok(size) = reader.read_line(&mut buffer) {
        if size == 0 {
            break;
        }
    }
}
