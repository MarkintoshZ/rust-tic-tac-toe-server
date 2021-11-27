use dipa::Diffable;
use lunatic::{process::Process, Mailbox};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::client::ClientMsg;
use shared::message::{GameAction, GameState, MessageFromServer, Username};

#[derive(Serialize, Deserialize)]
pub enum RoomMsg {
    JoinRoom(Client),
    LeaveRoom(Client),
    Drop(Username),
    Action(Client, GameAction),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Client {
    pub username: Username,
    process: Process<ClientMsg>,
}

pub(crate) type HaveFullState = bool;

impl Client {
    pub fn new(username: Username, client_proc: Process<ClientMsg>) -> Self {
        Self {
            username,
            process: client_proc,
        }
    }
}

pub trait Room {
    fn new(room_name: String) -> Self;
    fn on_join(&mut self, client: Client, ctx: &Context);
    fn on_leave(&mut self, client: Client, ctx: &Context);
    fn on_drop(&mut self, client_username: Username, ctx: &Context);
    fn on_msg(&mut self, client: Client, msg: GameAction, ctx: &Context);
    fn on_update(&mut self, delta_time: Duration, ctx: &Context) {}
    fn update_interval() -> Option<Duration> {
        None
    }
    fn max_client() -> Option<usize> {
        None
    }
}

pub struct Context<'a> {
    clients: &'a mut HashMap<Username, (Process<ClientMsg>, HaveFullState)>,
    prev_state: &'a mut Option<GameState>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        clients: &'a mut HashMap<Username, (Process<ClientMsg>, HaveFullState)>,
        state_checkpoint: &'a mut Option<GameState>,
    ) -> Self {
        Self {
            clients,
            prev_state: state_checkpoint,
        }
    }

    pub fn broadcast(&self, state: &GameState) {
        if let Some(prev_state) = &self.prev_state {
            let delta = prev_state.create_delta_towards(state).delta;
            let bytes = bincode::serialize(&delta).unwrap();
            self.clients.values().for_each(|(client, have_full_state)| {
                let msg = {
                    if *have_full_state {
                        MessageFromServer::StateChanged(bytes.clone())
                    } else {
                        MessageFromServer::State(state.clone())
                    }
                };
                client.send(ClientMsg::RoomMessage(msg));
            });
        } else {
            self.clients.values().for_each(|(client, _)| {
                client.send(ClientMsg::RoomMessage(MessageFromServer::State(
                    state.clone(),
                )));
            });
        }
    }
}

pub fn room_process<T: Room>(room_name: String, mailbox: Mailbox<RoomMsg>) {
    let mut clients = HashMap::<Username, (Process<ClientMsg>, HaveFullState)>::new();
    let mut state_checkpoint: Option<GameState> = None;
    let mut room = T::new(room_name);

    while let Ok(message) = mailbox.receive() {
        match message {
            RoomMsg::JoinRoom(client) => {
                clients.insert(client.username.clone(), (client.process.clone(), false));
                let context = Context::new(&mut clients, &mut state_checkpoint);
                room.on_join(client, &context);
            }
            RoomMsg::LeaveRoom(client) => {
                clients.remove(&client.username);
                if clients.len() == 0 {
                    break;
                }
                let context = Context::new(&mut clients, &mut state_checkpoint);
                room.on_leave(client.clone(), &context);
            }
            RoomMsg::Drop(username) => {
                clients.remove(&username);
                let context = Context::new(&mut clients, &mut state_checkpoint);
                room.on_drop(username.clone(), &context);
                if clients.len() == 0 {
                    break;
                }
            }
            RoomMsg::Action(client, action) => {
                let context = Context::new(&mut clients, &mut state_checkpoint);
                room.on_msg(client, action, &context);
            }
        }
    }
}
