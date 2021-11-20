use lunatic::{process::Process, Mailbox, Request};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::coordinator::{CoordinatorRequest, CoordinatorResponse};
use crate::{client::ClientMsg, coordinator::Client};

#[derive(Serialize, Deserialize)]
pub enum RoomMessage {
    JoinRoom(Process<ClientMsg>),
    LeaveRoom,
    Drop(u128),
    StateChange(StateChange),
}

#[derive(Serialize, Deserialize)]
pub enum StateChange {}

pub trait Room {
    fn on_join(client: Client, ctx: Context) -> bool;
    fn on_leave(client: Client, ctx: Context);
    fn on_drop(client: Client, ctx: Context);
    fn on_msg(client: ClientMsg, ctx: Context);
    fn on_update(delta_time: Duration, ctx: Context);
    fn update_interval() -> Option<Duration>;
}

pub struct Context {
    room_process: Process<RoomMessage>,
}

impl Context {
    pub fn broadcast() {}

    pub fn msg_to() {}
}

struct GameRoom {}

impl Room for GameRoom {
    fn on_join(client: Client, ctx: Context) -> bool {
        todo!()
    }

    fn on_leave(client: Client, ctx: Context) {
        todo!()
    }

    fn on_drop(client: Client, ctx: Context) {
        todo!()
    }

    fn on_msg(client: ClientMsg, ctx: Context) {
        todo!()
    }

    fn on_update(delta_time: std::time::Duration, ctx: Context) {
        todo!()
    }

    fn update_interval() -> Option<Duration> {
        todo!()
    }
}

pub fn room_process(
    (room_name, coordinator): (
        String,
        Process<Request<CoordinatorRequest, CoordinatorResponse>>,
    ),
    mailbox: Mailbox<RoomMessage>,
) {
    while let Ok(message) = mailbox.receive() {}
}
