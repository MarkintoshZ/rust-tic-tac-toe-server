mod client;
mod coordinator;
mod room;

use client::client_process;
use coordinator::coordinator_process;
use lunatic::{net, process};
use room::Room;

struct GameRoom {
    room_name: String,
}

impl Room for GameRoom {
    fn new(room_name: String) -> Self {
        Self { room_name }
    }

    fn on_join(&mut self, client: room::Client, ctx: &room::Context) {
        println!("Client {} joined room {}", client.username, self.room_name);
    }

    fn on_leave(&mut self, client: room::Client, ctx: &room::Context) {
        println!("Client {} left room {}", client.username, self.room_name);
    }

    fn on_drop(&mut self, client_username: shared::message::Username, ctx: &room::Context) {
        println!(
            "Client {} dropped from room {}",
            client_username, self.room_name
        );
    }

    fn on_msg(
        &mut self,
        client: room::Client,
        msg: shared::message::GameAction,
        ctx: &room::Context,
    ) {
        println!(
            "Client {} messaged {:?} in room {}",
            client.username, msg, self.room_name
        );
    }

    fn on_update(&mut self, delta_time: std::time::Duration, ctx: &room::Context) {
        println!("Room {} on update", self.room_name);
    }

    fn update_interval() -> Option<std::time::Duration> {
        None
    }

    fn max_client() -> Option<usize> {
        None
    }
}

fn main() {
    let coordinator = process::spawn(coordinator_process::<GameRoom>).unwrap();
    let listener = net::TcpListener::bind("127.0.0.1:1337").unwrap();
    while let Ok((tcp_stream, _peer)) = listener.accept() {
        process::spawn_with((tcp_stream, coordinator.clone()), client_process).unwrap();
    }
}
