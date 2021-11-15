use lunatic::{process::Process, Mailbox};
use serde::{Deserialize, Serialize};

use crate::client::ClientMessage;

#[derive(Serialize, Deserialize)]
pub enum RoomMessage {
    JoinRoom(Process<ClientMessage>),
    LeaveRoom,
    Drop(u128),
}

pub fn room_process(room_name: String, mailbox: Mailbox<RoomMessage>) {}
