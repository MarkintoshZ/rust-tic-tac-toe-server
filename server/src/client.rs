use lunatic::{
    net::TcpStream,
    process::{self, Process},
    Mailbox,
};
use serde::{Deserialize, Serialize};
use shared::{
    message::{MessageFromClient, MessageFromServer, Username},
    serialize::*,
};
use std::{
    io::{BufRead, BufReader},
    time::Duration,
};

use crate::coordinator::{CoordinatorMsg, CoordinatorRequest, CoordinatorResponse};
use crate::room::{Client, RoomMsg};

#[derive(Serialize, Deserialize)]
pub enum ClientMsg {
    ClientDropped,
    MessageFromClient(MessageFromClient),
    RoomMessage(MessageFromServer),
}

pub fn client_process(
    (mut stream, coordinator): (TcpStream, Process<CoordinatorMsg>),
    mailbox: Mailbox<ClientMsg>,
) {
    println!("client process created for stream: {:?}", stream);

    let mut current_room: Option<Process<RoomMsg>> = None;

    let mut username: Option<Username> = None;

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
                if let Ok(size) = reader.read_line(&mut buffer) {
                    if size == 0 {
                        client.send(ClientMsg::ClientDropped);
                        break;
                    }

                    let msg: MessageFromClient = read_serialized(buffer.as_bytes()).unwrap();
                    let should_break = matches!(msg, MessageFromClient::LeaveServer);
                    client.send(ClientMsg::MessageFromClient(msg));
                    if should_break {
                        break;
                    }

                    buffer.clear();
                } else {
                    // read timeout
                    client.send(ClientMsg::ClientDropped);
                    break;
                }
            }
        },
    )
    .unwrap();

    while let Ok(msg) = mailbox.receive() {
        match msg {
            ClientMsg::ClientDropped => {
                if username.is_some() {
                    coordinator
                        .request(CoordinatorRequest::LeaveServer)
                        .unwrap();
                }
                // do not go though the regular leave server procedure
                return;
            }
            ClientMsg::MessageFromClient(client_msg) => match client_msg {
                MessageFromClient::JoinServer(username_) => {
                    match coordinator
                        .request(CoordinatorRequest::JoinServer(username_.clone()))
                        .unwrap()
                    {
                        CoordinatorResponse::ServerJoined => {
                            username = Some(username_);
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
                    current_room = None;
                    coordinator
                        .request(CoordinatorRequest::LeaveRoom(process::this(&mailbox)))
                        .unwrap();
                }
                MessageFromClient::GameAction(action) => {
                    match &current_room {
                        Some(room) => {
                            room.send(RoomMsg::Action(
                                Client::new(username.clone().unwrap(), process::this(&mailbox)),
                                action,
                            ));
                        }
                        None => panic!("client cannot execute game actions before joining a room"),
                    };
                }
            },
            ClientMsg::RoomMessage(room_msg) => {
                write_serialized(room_msg, &mut stream).unwrap();
            }
        }
    }

    // only request leave server if client has joined server in the first place
    if username.is_some() {
        coordinator
            .request(CoordinatorRequest::LeaveServer)
            .unwrap();
    }
}
