mod state;

use state::State;

use shared::{
    message::{GameAction, MessageFromClient, MessageFromServer},
    serialize::{deserialize, serialize},
};
use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};
use structopt::StructOpt;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

#[derive(Debug, StructOpt)]
#[structopt(name = "tic tac toe client", about = "A tic tac toe client, probably")]
struct Opt {
    #[structopt(default_value = "127.0.0.1")]
    ip_addr: IpAddr,

    #[structopt(default_value = "1337")]
    port: u16,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let (tcp_read, tcp_write) = {
        let stream = TcpStream::connect((opt.ip_addr, opt.port)).await.expect(
            &format!(
                "cannot establish TCP connection with {}:{}",
                opt.ip_addr, opt.port
            )[..],
        );

        stream.into_split()
    };

    let state = Arc::new(Mutex::new(State::new()));

    tokio::select!(
        _ = receive_tcp(tcp_read, Arc::clone(&state)) => {
            println!("TCP connection closed (probably because of timeout)");
            return Ok(());
        },
        _ = send_tcp(tcp_write, state) => {
            println!("Connection closed");
            return Ok(());
        },
    );
}

async fn receive_tcp(tcp_read: OwnedReadHalf, state: Arc<Mutex<State>>) -> Result<()> {
    let mut buf = Vec::<u8>::new();

    let mut reader = BufReader::new(tcp_read);
    while let Ok(size) = reader.read_until('\n' as u8, &mut buf).await {
        if size == 0 {
            break;
        }
        let msg = deserialize::<MessageFromServer>(&buf)
            .expect("Failed to deserialize message from server");
        match msg {
            MessageFromServer::State(game_state) => {
                let mut state = state.lock().unwrap();
                state.game_state = game_state;
                state.print_game_state();
            }
            MessageFromServer::StateChanged(serialized_state_delta) => {
                let mut state = state.lock().unwrap();
                state.apply_patch(serialized_state_delta);
                state.print_game_state();
            }
            MessageFromServer::RoomFull
            | MessageFromServer::RoomDoesNotExist
            | MessageFromServer::RoomNameAlreadyTaken => {
                state.lock().unwrap().room = None;
                println!("{:?}", msg);
            }
            MessageFromServer::UsernameAlreadyTaken => {
                state.lock().unwrap().username = None;
                println!("{:?}", msg);
            }
            _ => {
                println!("{:?}", msg);
            }
        }
        buf.clear();
    }
    Ok(())
}

async fn send_tcp(mut tcp_write: OwnedWriteHalf, state: Arc<Mutex<State>>) -> Result<()> {
    while let Ok(response) = prompt(None).await {
        let msg = match response.as_ref() {
            "join server" => {
                let username = prompt(Some("username: ")).await?;
                let mut state = state.lock().unwrap();
                state.username = Some(username.clone());
                Some(MessageFromClient::JoinServer(username))
            }
            "leave server" => Some(MessageFromClient::LeaveServer),
            "create room" => {
                let room_name = prompt(Some("room name: ")).await?;
                let mut state = state.lock().unwrap();
                state.room = Some(room_name.clone());
                Some(MessageFromClient::CreateRoom(room_name))
            }
            "join room" => {
                let room_name = prompt(Some("room name: ")).await?;
                let mut state = state.lock().unwrap();
                state.room = Some(room_name.clone());
                Some(MessageFromClient::JoinRoom(room_name))
            }
            "leave room" => {
                let mut state = state.lock().unwrap();
                state.room = None;
                Some(MessageFromClient::LeaveRoom)
            }
            "restart" => {
                let state = state.lock().unwrap();
                if !state.game_state.started {
                    println!("Still waiting for players!");
                    None
                } else {
                    Some(MessageFromClient::GameAction(GameAction::Restart))
                }
            }
            "" => None,
            _ => {
                if response.starts_with("place at ") {
                    let positions: Vec<_> = response[9..]
                        .split_whitespace()
                        .map(|s| s.parse::<usize>())
                        .collect();
                    match positions[..] {
                        [Ok(x), Ok(y)] => {
                            let state = state.lock().unwrap();
                            if state.room.is_none() {
                                println!("Invalid place at command. e.g.: place at 0 0");
                                None
                            } else if !state.is_my_turn() {
                                println!("Not your turn!");
                                None
                            } else if !state.game_state.started {
                                println!("Still waiting for players!");
                                None
                            } else if x > 2 || y > 2 {
                                println!("Invalid position(s)! Position must be between 0 and 2 inclusive");
                                None
                            } else if !state.can_place_at(x, y) {
                                println!("Invalid position!");
                                None
                            } else {
                                Some(MessageFromClient::GameAction(GameAction::PlaceNode(x, y)))
                            }
                        }
                        _ => {
                            println!("Invalid place at command. e.g.: place at 0 0");
                            None
                        }
                    }
                } else {
                    println!("Invalid command");
                    None
                }
            }
        };
        if let Some(msg) = msg {
            let should_break = matches!(msg, MessageFromClient::LeaveServer);
            let bytes = serialize(msg)?;
            tcp_write.write_all(&bytes[..]).await?;

            if should_break {
                break;
            }
        }
    }
    Ok(())
}

async fn prompt(prompt: Option<&str>) -> Result<String> {
    if let Some(prompt) = prompt {
        println!("{}", prompt);
    }
    let mut reader = BufReader::new(io::stdin());
    let mut buf = String::new();
    reader.read_line(&mut buf).await?;
    Ok(buf[..buf.len() - 1].to_string())
}
