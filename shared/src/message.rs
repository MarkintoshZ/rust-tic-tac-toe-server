use serde::{Deserialize, Serialize};

pub type Username = String;
pub type RoomName = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromServer {
    // Server related messages
    ServerJoined,
    UsernameAlreadyTaken,

    // Room related messages
    RoomJoined,
    RoomFull,
    RoomDoesNotExist,
    RoomCreated,
    RoomNameAlreadyTaken,

    // Game state broadcast messages
    StateChanged(StateDelta),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClient {
    // Server related messages
    JoinServer(Username), // -> ServerJoined or UsernameAlreadyTaken
    LeaveServer,          // -> no response

    // Room related messages
    JoinRoom(RoomName), // -> RoomJoined or RoomFull or RoomDoesNotExist
    CreateRoom(String), // -> RoomCreated or RoomNameAlreadyTaken
    LeaveRoom,

    // Game-specific messages
    GameAction(GameAction),
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
