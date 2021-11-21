use lunatic::{
    net::TcpStream,
    process::{self, Process},
    Mailbox, Request,
};
use serde::{Deserialize, Serialize};
use shared::{
    message::{MessageFromClient, MessageFromServer},
    serialize::*,
};
use std::{
    io::{BufRead, BufReader, Write},
    time::Duration,
};

use crate::coordinator::{CoordinatorMsg, CoordinatorRequest, CoordinatorResponse};
use crate::room::RoomMsg;

#[derive(Serialize, Deserialize)]
pub enum ClientMsg {
    ClientDropped,
    MessageFromClient(MessageFromClient),
    RoomMessage(RoomMsg),
}

pub fn client_process(
    (mut stream, coordinator): (TcpStream, Process<CoordinatorMsg>),
    mailbox: Mailbox<ClientMsg>,
) {
    println!("client process created for stream: {:?}", stream);

    let mut current_room: Option<Process<RoomMsg>> = None;

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

            loop {
                let size = reader.read_line(&mut buffer).unwrap();
                println!("read size of {}: {}", size, buffer);
                if size == 0 {
                    client.send(ClientMsg::ClientDropped);
                    break;
                }

                let msg: MessageFromClient = read_serialized(buffer.as_bytes()).unwrap();
                client.send(ClientMsg::MessageFromClient(msg.clone()));
                if let MessageFromClient::LeaveServer = msg {
                    break;
                }

                buffer.clear();
            }
        },
    )
    .unwrap();

    while let Ok(msg) = mailbox.receive() {
        match msg {
            ClientMsg::ClientDropped => {
                coordinator
                    .request(CoordinatorRequest::LeaveServer)
                    .unwrap();
                break;
            }
            ClientMsg::MessageFromClient(client_msg) => match client_msg {
                MessageFromClient::JoinServer(username) => {
                    match coordinator
                        .request(CoordinatorRequest::JoinServer(username))
                        .unwrap()
                    {
                        CoordinatorResponse::ServerJoined => {
                            write_serialized(MessageFromServer::ServerJoined, &mut stream).unwrap();
                        }
                        CoordinatorResponse::UsernameAlreadyTaken => {
                            write_serialized(MessageFromServer::UsernameAlreadyTaken, &mut stream)
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                MessageFromClient::LeaveServer => break,
                MessageFromClient::CreateRoom(room_name) => {
                    match coordinator
                        .request(CoordinatorRequest::CreateRoom(
                            room_name,
                            process::this(&mailbox),
                        ))
                        .unwrap()
                    {
                        CoordinatorResponse::RoomCreated(room_proc) => {
                            if current_room.is_some() {
                                panic!("client is creating a new room when it is already in a existing room")
                            };
                            current_room = Some(room_proc);
                            write_serialized(MessageFromServer::RoomCreated, &mut stream).unwrap();
                        }
                        CoordinatorResponse::RoomNameAlreadyTaken => {
                            write_serialized(MessageFromServer::RoomNameAlreadyTaken, &mut stream)
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                MessageFromClient::JoinRoom(room_name) => {
                    match coordinator
                        .request(CoordinatorRequest::JoinRoom(
                            room_name,
                            process::this(&mailbox),
                        ))
                        .unwrap()
                    {
                        CoordinatorResponse::RoomJoined(room_proc) => {
                            current_room = Some(room_proc);
                            write_serialized::<MessageFromServer, TcpStream>(
                                MessageFromServer::RoomJoined,
                                &mut stream,
                            )
                            .unwrap();
                        }
                        CoordinatorResponse::RoomFull => {
                            write_serialized(MessageFromServer::RoomFull, &mut stream).unwrap();
                        }
                        CoordinatorResponse::RoomDoesNotExist => {
                            write_serialized(MessageFromServer::RoomDoesNotExist, &mut stream)
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                MessageFromClient::LeaveRoom => {
                    coordinator.request(CoordinatorRequest::LeaveRoom).unwrap();
                }
                MessageFromClient::GameAction(action) => {
                    match &current_room {
                        Some(room) => {
                            room.send(RoomMsg::GameAction(action));
                        }
                        None => panic!(
                            "client is creating a new room when it is already in a existing room"
                        ),
                    };
                }
            },
            ClientMsg::RoomMessage(room_msg) => {
                write_serialized(room_msg, &mut stream).unwrap();
            }
        }
    }

    coordinator
        .request(CoordinatorRequest::LeaveServer)
        .unwrap();
}
