mod client;
mod coordinator;
mod room;
mod tcp;

use client::{client_process, ClientContext};
use coordinator::coordinator_process;
use lunatic::{net, process};

fn main() {
    let coordinator = process::spawn(coordinator_process).unwrap();
    let listener = net::TcpListener::bind("127.0.0.1:1337").unwrap();
    while let Ok((tcp_stream, _peer)) = listener.accept() {
        process::spawn_with(
            ClientContext::new(tcp_stream, coordinator.clone()),
            client_process,
        )
        .unwrap();
    }
}
