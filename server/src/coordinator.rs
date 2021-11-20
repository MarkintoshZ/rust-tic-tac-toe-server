use crate::client::ClientMsg;
use crate::room::{room_process, RoomMessage};

use lunatic::{
    process::{self, spawn_with, Process},
    Mailbox, Message, Request, Tag, TransformMailbox,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type RoomName = String;
pub type RoomSize = usize; // number of clients in the room

#[derive(Serialize, Deserialize, Debug)]
pub enum CoordinatorRequest {
    JoinServer,
    LeaveServer,
    ChangeName(String),
    CreateRoom(RoomName, Process<ClientMsg>),
    JoinRoom(RoomName, Process<ClientMsg>),
    JoinRandomRoom(Process<ClientMsg>),
    LeaveRoom,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CoordinatorResponse {
    ServerJoined(String),
    ServerLeft,
    NewName(String),
    RoomJoined(Process<RoomMessage>),
    RoomFull,
    RoomDoesNotExist,
    RoomCreated(Process<RoomMessage>),
    RoomAlreadyExist,
    RoomLeft,
}

#[derive(Debug)]
pub struct Client {
    tag: Tag,
    username: String,
    room: Option<Process<RoomMessage>>,
}

pub fn coordinator_process(
    mailbox: Mailbox<Request<CoordinatorRequest, CoordinatorResponse>>,
) -> () {
    let mut clients = HashMap::<u128, Client>::new();
    let mut rooms = HashMap::<RoomName, (Process<RoomMessage>, RoomSize)>::new();

    let mailbox = mailbox.catch_link_panic();

    let this_proc = process::this(&mailbox);

    loop {
        println!("clients: {:?}\nrooms: {:?}", clients, rooms);
        let message = mailbox.receive();

        if let Message::Signal(tag) = message {
            // Find the correct link
            let id =
                if let Some((id, client)) = clients.iter().find(|(_, client)| client.tag == tag) {
                    if let Some(room) = &client.room {
                        room.send(RoomMessage::Drop(*id));
                    }
                    Some(*id)
                } else {
                    None
                };
            if let Some(id) = id {
                clients.remove(&id);
            }
        }

        if let Message::Normal(request) = message {
            let request = request.unwrap();
            let data = request.data();
            match data {
                CoordinatorRequest::JoinServer => {
                    let name: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(6)
                        .map(char::from)
                        .collect();
                    clients.insert(
                        request.sender().id(),
                        Client {
                            tag: request.sender().link(),
                            username: name.clone(),
                            room: None,
                        },
                    );
                    request.reply(CoordinatorResponse::ServerJoined(name));
                }
                CoordinatorRequest::LeaveServer => {
                    request.sender().send(CoordinatorResponse::ServerLeft);
                    clients.remove(&request.sender().id());
                }
                CoordinatorRequest::ChangeName(name) => {
                    let client = request.sender();
                    if let Some((_, c)) = clients.iter().find(|(_, c)| c.username == *name) {
                        // name is not taken yet
                        let old_name = c.username.clone();
                        request.reply(CoordinatorResponse::NewName(old_name));
                    } else {
                        let new_name = name.to_string();
                        clients.get_mut(&client.id()).unwrap().username = new_name.clone();
                        request.reply(CoordinatorResponse::NewName(new_name));
                    }
                }
                CoordinatorRequest::CreateRoom(room_name, _client) => {
                    if let Some(_) = rooms.get(room_name) {
                        request.reply(CoordinatorResponse::RoomAlreadyExist);
                    } else {
                        let room_proc =
                            spawn_with((room_name.clone(), this_proc.clone()), room_process)
                                .unwrap();
                        let room_name = room_name.to_string();
                        rooms.insert(room_name.clone(), (room_proc.clone(), 1));
                        request.reply(CoordinatorResponse::RoomCreated(room_proc.clone()));
                    }
                }
                CoordinatorRequest::JoinRoom(room_name, client) => {
                    if let Some((existing_room, room_size)) = rooms.get_mut(&room_name.clone()) {
                        if *room_size < 2 {
                            *room_size += 1;
                            existing_room.send(RoomMessage::JoinRoom(client.clone()));
                            request.reply(CoordinatorResponse::RoomJoined(existing_room.clone()));
                        } else {
                            request.reply(CoordinatorResponse::RoomFull);
                            return;
                        }
                    } else {
                        request.reply(CoordinatorResponse::RoomDoesNotExist);
                    }
                }
                CoordinatorRequest::JoinRandomRoom(client) => {
                    if let Some((_, (existing, _))) = rooms.iter_mut().find(|(_, (_, s))| *s < 2) {
                        existing.send(RoomMessage::JoinRoom(client.clone()));
                        request.reply(CoordinatorResponse::RoomJoined(existing.clone()));
                    } else {
                        // create new room with random name if no room available
                        let room_name: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(6)
                            .map(char::from)
                            .collect();
                        let room_proc =
                            spawn_with((room_name.clone(), this_proc.clone()), room_process)
                                .unwrap();
                        rooms.insert(room_name, (room_proc.clone(), 1));
                        request.reply(CoordinatorResponse::RoomCreated(room_proc.clone()));
                    }
                }
                CoordinatorRequest::LeaveRoom => {
                    let client = clients.get(&request.sender().id()).unwrap();
                    let room_proc = client.room.as_ref().unwrap();
                    room_proc.send(RoomMessage::LeaveRoom);
                    let room_to_remove = if let Some((room_name, (_, room_size))) =
                        rooms.iter_mut().find(|(_, (p, _))| *p == *room_proc)
                    {
                        *room_size -= 1;
                        if *room_size == 0 {
                            Some(room_name.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(room_name) = room_to_remove {
                        rooms.remove(&room_name);
                    }
                    request.reply(CoordinatorResponse::RoomLeft);
                }
            }
        }
    }
}
