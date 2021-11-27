use dipa::Patchable;
use shared::{
    message::{GameState, Username},
    serialize::deserialize,
};

pub(crate) struct State {
    pub username: Option<Username>,
    pub room: Option<String>,
    pub game_state: GameState,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            username: None,
            room: None,
            game_state: GameState::default(),
        }
    }

    pub(crate) fn print_game_state(&self) {
        if self.game_state.started {
            // print board
            if let Some(winner_idx) = self.game_state.winner {
                let winner = &self.game_state.players[winner_idx as usize];
                if winner == self.username.as_ref().unwrap() {
                    println!("You won!",);
                } else {
                    println!("{} won!", winner);
                }
                println!("type command \"restart\" to restart");
            } else if self.game_state.is_gameover() {
                println!("Tied!");
                println!("type command \"restart\" to restart");
            } else {
                let turn = &self.game_state.players[self.game_state.turn as usize];
                if turn == self.username.as_ref().unwrap() {
                    println!("Your turn!")
                } else {
                    println!("{}'s turn!", turn);
                }
            }
            // print board
            println!("–––––––");
            for row in &self.game_state.board {
                for block in row {
                    let c = match block {
                        -1 => ' ',
                        0 => 'X',
                        1 => 'O',
                        _ => unreachable!(),
                    };
                    print!("|{}", c);
                }
                println!("|");
                println!("–––––––");
            }
            println!();
        } else {
            println!("Waiting for players...");
        }
    }

    pub(crate) fn can_place_at(&self, x: usize, y: usize) -> bool {
        self.game_state.board[x][y] == -1
    }

    pub(crate) fn is_my_turn(&self) -> bool {
        let turn = &self.game_state.players[self.game_state.turn as usize];
        turn == self.username.as_ref().unwrap()
    }

    pub(crate) fn apply_patch(&mut self, bytes: Vec<u8>) {
        let delta: <GameState as dipa::Diffable<'_, '_, GameState>>::DeltaOwned =
            deserialize(&bytes).expect("Failed to deserialize game state");
        self.game_state.apply_patch(delta);
    }
}
