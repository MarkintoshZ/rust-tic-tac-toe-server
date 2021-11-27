use dipa::DiffPatch;
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
    // stores the serialized byte version of CreatedDelta<GameState>
    // this is so that repeated cloning and serialization could be prevented
    StateChanged(Vec<u8>),
    State(GameState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClient {
    // Server related messages
    JoinServer(Username), // -> ServerJoined or UsernameAlreadyTaken
    LeaveServer,          // -> no response

    // Room related messages
    JoinRoom(RoomName),   // -> RoomJoined or RoomFull or RoomDoesNotExist
    CreateRoom(RoomName), // -> RoomCreated or RoomNameAlreadyTaken
    LeaveRoom,

    // Game-specific messages
    GameAction(GameAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameAction {
    PlaceNode(usize, usize),
    Restart,
}

#[derive(DiffPatch, Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub players: Vec<String>,
    pub board: Vec<Vec<i8>>,
    pub winner: Option<i8>,
    pub turn: u8,
    pub started: bool,
}

impl Default for GameState {
    fn default() -> Self {
        let board = (0..3).map(|_| (0..3).map(|_| -1).collect()).collect();
        Self {
            players: Vec::new(),
            board,
            winner: None,
            turn: 0,
            started: false,
        }
    }
}

impl GameState {
    pub fn reset_game(&mut self) {
        self.winner = None;
        self.board = (0..3).map(|_| (0..3).map(|_| -1).collect()).collect();
        self.turn = 0;
    }

    pub fn place_node(&mut self, x: usize, y: usize, username: &Username) {
        if self.board[x][y] == -1 {
            self.board[x][y] = (self.players[1] == *username) as i8;
            self.winner = self.check_winner(x, y);
        }
    }

    fn check_winner(&mut self, x: usize, y: usize) -> Option<i8> {
        let color = self.board[x][y];
        if self.board[0][y] == self.board[1][y] && self.board[1][y] == self.board[2][y] {
            Some(color)
        } else if self.board[x][0] == self.board[x][1] && self.board[x][1] == self.board[x][2] {
            Some(color)
        } else {
            if self.board[0][0] == self.board[1][1]
                && self.board[1][1] == self.board[2][2]
                && self.board[1][1] == color
            {
                Some(color)
            } else if self.board[0][2] == self.board[1][1]
                && self.board[1][1] == self.board[2][0]
                && self.board[1][1] == color
            {
                Some(color)
            } else {
                None
            }
        }
    }

    pub fn is_gameover(&self) -> bool {
        for row in &self.board {
            for block in row {
                if *block == -1 {
                    return false;
                }
            }
        }
        return true;
    }
}
