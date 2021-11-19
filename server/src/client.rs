use shared::{message::MessageFromClient, serialize::*};
use std::{
    io::{BufRead, BufReader, Write},
    time::Duration,
};

use bincode;
use lunatic::{
    net::TcpStream,
    process::{self, Process},
    Mailbox, Request,
};
use serde::{Deserialize, Serialize};

use crate::coordinator::{CoordinatorRequest, CoordinatorResponse};
use crate::room::RoomMessage;

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    MessageFromClient(MessageFromClient),
    RoomMessage(RoomMessage),
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

    // Spawn another actor to handle message reception and deserialization.
    // This actor will send the deserialized client message to the client's
    // (this) mailbox
    let client_proc = process::this(&mailbox);
    let (_, mailbox) = process::spawn_link_unwrap_with(
        mailbox,
        (client_proc, stream.clone()),
        |(client, mut stream), _: Mailbox<()>| {
            // set five minute timeout
            stream.set_read_timeout(Some(Duration::new(5 * 60, 0)));

            let mut reader = BufReader::new(stream.clone());
            let mut buffer = String::new();

            while let Ok(size) = reader.read_line(&mut buffer) {
                println!("read something: {:}", buffer);
                if size == 0 {
                    break;
                }

                if let Ok(msg) = read_serialized(buffer.as_bytes()) {
                    client.send(msg);
                }

                buffer.clear();
            }
        },
    )
    .unwrap();

    let username = if let CoordinatorResponse::ServerJoined(username) =
        coordinator.request(CoordinatorRequest::JoinServer).unwrap()
    {
        username
    } else {
        unreachable!()
    };

    println!("generated username: {:?}", username);

    stream.write((username + "\n").as_bytes()).unwrap();

    while let Ok(msg) = mailbox.receive() {
        match msg {
            ClientMessage::MessageFromClient(client_msg) => {}
            ClientMessage::RoomMessage(room_msg) => {
                write_serialized(room_msg, &mut stream).unwrap();
            }
        }
    }

    coordinator
        .request(CoordinatorRequest::LeaveServer)
        .unwrap();
}
