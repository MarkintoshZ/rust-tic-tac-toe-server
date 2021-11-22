use crate::client::ClientMsg;
use crate::room::{room_process, Client, Room, RoomMsg};

use lunatic::{
    process::{self, spawn_with, Process},
    Mailbox, Message, Request, Tag, TransformMailbox,
};
use serde::{Deserialize, Serialize};
use shared::message::Username;
use std::collections::HashMap;

pub type RoomName = String;
pub type RoomSize = usize; // number of clients in the room

#[derive(Serialize, Deserialize, Debug)]
pub enum CoordinatorRequest {
    // Server related messages
    JoinServer(Username), // -> ServerJoined or UsernameAlreadyTaken
    LeaveServer,          // -> no response

    // Room related messages
    JoinRoom(RoomName, Process<ClientMsg>), // -> RoomJoined or RoomFull or RoomDoesNotExist
    CreateRoom(String, Process<ClientMsg>), // -> RoomCreated or RoomNameAlreadyTaken
    LeaveRoom(Process<ClientMsg>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CoordinatorResponse {
    // Server related messages
    ServerJoined,
    UsernameAlreadyTaken,
    ServerLeft,

    // Room related messages
    RoomJoined(Process<RoomMsg>),
    RoomFull,
    RoomDoesNotExist,
    RoomCreated(Process<RoomMsg>),
    RoomNameAlreadyTaken,
    RoomLeft,
}

#[derive(Debug)]
pub(crate) struct ClientInfo {
    tag: Tag,
    username: String,
    room: Option<Process<RoomMsg>>,
}

pub(crate) type CoordinatorMsg = Request<CoordinatorRequest, CoordinatorResponse>;

pub fn coordinator_process<T: Room>(mailbox: Mailbox<CoordinatorMsg>) -> () {
    let mut clients = HashMap::<u128, ClientInfo>::new();
    let mut rooms = HashMap::<RoomName, (Process<RoomMsg>, RoomSize)>::new();

    let mailbox = mailbox.catch_link_panic();

    let this_proc = process::this(&mailbox);

    loop {
        println!("\nclients: {:?}\nrooms: {:?}", clients, rooms);
        let message = mailbox.receive();

        if let Message::Signal(tag) = message {
            // Find the correct link
            let id =
                if let Some((id, client)) = clients.iter().find(|(_, client)| client.tag == tag) {
                    if let Some(room) = &client.room {
                        println!("Send drop msg to room");
                        let username = clients.get(id).unwrap().username.clone();
                        room.send(RoomMsg::Drop(username));
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
                CoordinatorRequest::JoinServer(username) => {
                    if clients.values().find(|c| c.username == *username).is_some() {
                        request.reply(CoordinatorResponse::UsernameAlreadyTaken);
                    } else {
                        clients.insert(
                            request.sender().id(),
                            ClientInfo {
                                tag: request.sender().link(),
                                username: username.clone(),
                                room: None,
                            },
                        );
                        request.reply(CoordinatorResponse::ServerJoined);
                    }
                }
                CoordinatorRequest::LeaveServer => {
                    let mut client = clients.get_mut(&request.sender().id()).unwrap();
                    if let Some(room_proc) = client.room.as_ref() {
                        room_proc.send(RoomMsg::Drop(client.username.clone()));
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
                        client.room = None;
                        if let Some(room_name) = room_to_remove {
                            rooms.remove(&room_name);
                        }
                    }
                    clients.remove(&request.sender().id());
                    request.reply(CoordinatorResponse::ServerLeft);
                }
                CoordinatorRequest::CreateRoom(room_name, client_proc) => {
                    if let Some(_) = rooms.get(room_name) {
                        request.reply(CoordinatorResponse::RoomNameAlreadyTaken);
                    } else {
                        let room_proc =
                            spawn_with((room_name.clone(), this_proc.clone()), room_process::<T>)
                                .unwrap();
                        room_proc.send(RoomMsg::JoinRoom(Client::new(
                            clients
                                .get_mut(&request.sender().id())
                                .unwrap()
                                .username
                                .clone(),
                            client_proc.clone(),
                        )));
                        let room_name = room_name.to_string();
                        rooms.insert(room_name.clone(), (room_proc.clone(), 1));
                        clients.get_mut(&request.sender().id()).unwrap().room =
                            Some(room_proc.clone());
                        request.reply(CoordinatorResponse::RoomCreated(room_proc.clone()));
                    }
                }
                CoordinatorRequest::JoinRoom(room_name, client_proc) => {
                    if let Some((room_proc, room_size)) = rooms.get_mut(&room_name.clone()) {
                        let max_client = T::max_client();
                        if max_client.is_none() || *room_size < max_client.unwrap() {
                            *room_size += 1;
                            room_proc.send(RoomMsg::JoinRoom(Client::new(
                                clients
                                    .get_mut(&request.sender().id())
                                    .unwrap()
                                    .username
                                    .clone(),
                                client_proc.clone(),
                            )));
                            clients.get_mut(&request.sender().id()).unwrap().room =
                                Some(room_proc.clone());
                            request.reply(CoordinatorResponse::RoomJoined(room_proc.clone()));
                        } else {
                            request.reply(CoordinatorResponse::RoomFull);
                            return;
                        }
                    } else {
                        request.reply(CoordinatorResponse::RoomDoesNotExist);
                    }
                }
                CoordinatorRequest::LeaveRoom(client_proc) => {
                    let mut client = clients.get_mut(&request.sender().id()).unwrap();
                    let room_proc = client.room.as_ref().unwrap();
                    room_proc.send(RoomMsg::LeaveRoom(Client::new(
                        client.username.clone(),
                        client_proc.clone(),
                    )));
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
                    client.room = None;
                    if let Some(room_name) = room_to_remove {
                        rooms.remove(&room_name);
                    }
                    request.reply(CoordinatorResponse::RoomLeft);
                }
            }
        }
    }
}
