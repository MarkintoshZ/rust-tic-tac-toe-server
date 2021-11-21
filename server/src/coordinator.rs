use crate::client::ClientMsg;
use crate::room::{room_process, RoomMsg};

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
    LeaveRoom,
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
pub struct Client {
    tag: Tag,
    username: String,
    room: Option<Process<RoomMsg>>,
}

pub(crate) type CoordinatorMsg = Request<CoordinatorRequest, CoordinatorResponse>;

pub fn coordinator_process(mailbox: Mailbox<CoordinatorMsg>) -> () {
    let mut clients = HashMap::<u128, Client>::new();
    let mut rooms = HashMap::<RoomName, (Process<RoomMsg>, RoomSize)>::new();

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
                        room.send(RoomMsg::Drop(*id));
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
                            Client {
                                tag: request.sender().link(),
                                username: username.clone(),
                                room: None,
                            },
                        );
                        request.reply(CoordinatorResponse::ServerJoined);
                    }
                }
                CoordinatorRequest::LeaveServer => {
                    request.sender().send(CoordinatorResponse::ServerLeft);
                    clients.remove(&request.sender().id());
                }
                CoordinatorRequest::CreateRoom(room_name, client) => {
                    if let Some(_) = rooms.get(room_name) {
                        request.reply(CoordinatorResponse::RoomNameAlreadyTaken);
                    } else {
                        let room_proc =
                            spawn_with((room_name.clone(), this_proc.clone()), room_process)
                                .unwrap();
                        room_proc.send(RoomMsg::JoinRoom(client.clone()));
                        let room_name = room_name.to_string();
                        rooms.insert(room_name.clone(), (room_proc.clone(), 1));
                        request.reply(CoordinatorResponse::RoomCreated(room_proc.clone()));
                    }
                }
                CoordinatorRequest::JoinRoom(room_name, client) => {
                    if let Some((existing_room, room_size)) = rooms.get_mut(&room_name.clone()) {
                        if *room_size < 2 {
                            *room_size += 1;
                            existing_room.send(RoomMsg::JoinRoom(client.clone()));
                            request.reply(CoordinatorResponse::RoomJoined(existing_room.clone()));
                        } else {
                            request.reply(CoordinatorResponse::RoomFull);
                            return;
                        }
                    } else {
                        request.reply(CoordinatorResponse::RoomDoesNotExist);
                    }
                }
                CoordinatorRequest::LeaveRoom => {
                    let client = clients.get(&request.sender().id()).unwrap();
                    let room_proc = client.room.as_ref().unwrap();
                    room_proc.send(RoomMsg::LeaveRoom);
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
