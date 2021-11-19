use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromServer {
    RoomJoined(Room),
    RoomFull,
    StateChanged(StateDelta),
    StateReset,
    PlayerWon(Player),
    ChangeNameResult(bool),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClient {
    ChangeName(String), // -> ChangeNameResult(bool)
    LeaveServer,
    JoinRoom(String),
    JoinRandomRoom, // -> Room Joined
    CreateRoom(String),
    LeaveRoom,
    GameAction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameAction {
    PlaceNode(usize, usize),
    Restart,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Player {
    username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Room {
    id: usize,
    player_1: Player,
    player_2: Player,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateDelta {}
