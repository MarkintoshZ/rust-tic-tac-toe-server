use lunatic::{process::Process, Mailbox, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::client::ClientMsg;
use crate::coordinator::{CoordinatorRequest, CoordinatorResponse};
use shared::message::{GameAction, MessageFromServer, Username};

#[derive(Serialize, Deserialize)]
pub enum RoomMsg {
    JoinRoom(Client),
    LeaveRoom(Client),
    Drop(Username),
    Action(Client, GameAction),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Client {
    pub username: String,
    process: Process<ClientMsg>,
}

impl Client {
    pub fn new(username: Username, client_proc: Process<ClientMsg>) -> Self {
        Self {
            username,
            process: client_proc,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum StateChange {}

pub trait Room {
    fn new(room_name: String) -> Self;
    fn on_join(&mut self, client: Client, ctx: &Context);
    fn on_leave(&mut self, client: Client, ctx: &Context);
    fn on_drop(&mut self, client_username: Username, ctx: &Context);
    fn on_msg(&mut self, client: Client, msg: GameAction, ctx: &Context);
    fn on_update(&mut self, delta_time: Duration, ctx: &Context);
    fn update_interval() -> Option<Duration>;
    fn max_client() -> Option<usize>;
}

pub struct Context<'a> {
    clients: &'a HashMap<Username, Process<ClientMsg>>,
    coordinator: &'a Process<Request<CoordinatorRequest, CoordinatorResponse>>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        clients: &'a HashMap<Username, Process<ClientMsg>>,
        coordinator: &'a Process<Request<CoordinatorRequest, CoordinatorResponse>>,
    ) -> Self {
        Self {
            clients,
            coordinator,
        }
    }

    pub fn broadcast(&self, msg: MessageFromServer) {
        self.clients.values().for_each(|client| {
            client.send(ClientMsg::RoomMessage(msg.clone()));
        });
    }

    pub fn msg_to(&self, username: &Username, msg: MessageFromServer) {
        self.clients
            .get(username)
            .unwrap()
            .send(ClientMsg::RoomMessage(msg));
    }
}

pub fn room_process<T: Room>(
    (room_name, coordinator): (
        String,
        Process<Request<CoordinatorRequest, CoordinatorResponse>>,
    ),
    mailbox: Mailbox<RoomMsg>,
) {
    let mut clients = HashMap::<Username, Process<ClientMsg>>::new();
    let mut room = T::new(room_name);

    while let Ok(message) = mailbox.receive() {
        match message {
            RoomMsg::JoinRoom(client) => {
                clients.insert(client.username.clone(), client.process.clone());
                let context = Context::new(&clients, &coordinator);
                room.on_join(client, &context);
            }
            RoomMsg::LeaveRoom(client) => {
                let context = Context::new(&clients, &coordinator);
                room.on_leave(client.clone(), &context);
                clients.remove(&client.username);
                if clients.len() == 0 {
                    break;
                }
            }
            RoomMsg::Drop(username) => {
                let context = Context::new(&clients, &coordinator);
                room.on_drop(username.clone(), &context);
                clients.remove(&username);
                if clients.len() == 0 {
                    break;
                }
            }
            RoomMsg::Action(client, action) => {
                let context = Context::new(&clients, &coordinator);
                room.on_msg(client, action, &context);
            }
        }
    }
}
