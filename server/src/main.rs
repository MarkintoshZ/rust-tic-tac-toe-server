mod client;
mod coordinator;
mod room;

use client::client_process;
use coordinator::coordinator_process;
use lunatic::{net, process};
use room::Room;
use shared::message::{GameAction, GameState};

struct GameRoom {
    room_name: String,
    state: GameState,
}

impl Room for GameRoom {
    fn new(room_name: String) -> Self {
        Self {
            room_name,
            state: GameState::default(),
        }
    }

    fn on_join(&mut self, client: room::Client, ctx: &room::Context) {
        println!("Client {} joined room {}", client.username, self.room_name);
        self.state.players.push(client.username.clone());
        if self.state.players.len() == 2 {
            self.state.started = true;
        }
        ctx.broadcast(&self.state);
    }

    fn on_leave(&mut self, client: room::Client, ctx: &room::Context) {
        println!("Client {} left room {}", client.username, self.room_name);
        self.state.players.retain(|p| *p != client.username);
        self.state.started = false;
        self.state.reset_game();
        ctx.broadcast(&self.state);
    }

    fn on_drop(&mut self, client_username: shared::message::Username, ctx: &room::Context) {
        println!(
            "Client {} dropped from room {}",
            client_username, self.room_name
        );
        self.state.players.retain(|p| *p != client_username);
        self.state.started = false;
        self.state.reset_game();
        ctx.broadcast(&self.state);
    }

    fn on_msg(&mut self, client: room::Client, msg: GameAction, ctx: &room::Context) {
        println!(
            "Client {} messaged {:?} in room {}",
            client.username, msg, self.room_name
        );
        match msg {
            GameAction::PlaceNode(x, y) => {
                self.state.place_node(x, y, &client.username);
                self.state.turn = (self.state.turn == 0) as u8;
            }
            GameAction::Restart => {
                if self.state.winner.is_some() || self.state.is_gameover() {
                    self.state.reset_game();
                }
            }
        }
        ctx.broadcast(&self.state);
    }

    fn max_client() -> Option<usize> {
        Some(2)
    }
}

fn main() {
    let coordinator = process::spawn(coordinator_process::<GameRoom>).unwrap();
    let listener = net::TcpListener::bind("127.0.0.1:1337").unwrap();
    while let Ok((tcp_stream, _peer)) = listener.accept() {
        process::spawn_with((tcp_stream, coordinator.clone()), client_process).unwrap();
    }
}
