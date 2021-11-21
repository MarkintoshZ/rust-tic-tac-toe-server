use shared::{
    message::{MessageFromClient, MessageFromServer},
    serialize::{read_serialized, write_serialized},
};
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1337")?;

    let mut stdin_buf = String::new();

    loop {
        std::io::stdin().read_line(&mut stdin_buf)?;
        match stdin_buf[..stdin_buf.len() - 1].as_ref() {
            "join server" => {
                print!("username: ");
                std::io::stdout().flush()?;
                stdin_buf.clear();
                std::io::stdin().read_line(&mut stdin_buf)?;
                write_serialized(
                    MessageFromClient::JoinServer(stdin_buf[..stdin_buf.len() - 1].to_string()),
                    &mut stream,
                )
                .unwrap();
                receive_and_print(&mut stream)?;
            }
            "leave server" => {
                write_serialized(MessageFromClient::LeaveServer, &mut stream).unwrap();
                break;
            }
            "create room" => {
                print!("room name: ");
                std::io::stdout().flush()?;
                stdin_buf.clear();
                std::io::stdin().read_line(&mut stdin_buf)?;
                write_serialized(
                    MessageFromClient::CreateRoom(stdin_buf[..stdin_buf.len() - 1].to_string()),
                    &mut stream,
                )
                .unwrap();
                receive_and_print(&mut stream)?;
            }
            "join room" => {
                print!("room name: ");
                std::io::stdout().flush()?;
                stdin_buf.clear();
                std::io::stdin().read_line(&mut stdin_buf)?;
                write_serialized(
                    MessageFromClient::JoinRoom(stdin_buf[..stdin_buf.len() - 1].to_string()),
                    &mut stream,
                )
                .unwrap();
                receive_and_print(&mut stream)?;
            }
            "leave room" => {
                write_serialized(MessageFromClient::LeaveRoom, &mut stream).unwrap();
            }
            "exit" => {
                break;
            }
            _ => {
                println!("Invalid command");
            }
        }
        stdin_buf.clear();
    }

    Ok(())
}

fn receive_and_print(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buf = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut buf)?;
    println!(
        "{:?}",
        read_serialized::<MessageFromServer>(buf.as_bytes()).expect("Server Error!")
    );
    Ok(())
}
