use shared::{
    message::{MessageFromClient, MessageFromServer},
    serialize::{read_serialized, write_serialized},
};
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1337")?;

    write_serialized(MessageFromClient::JoinServer("Mark".into()), &mut stream).unwrap();

    let mut buf = String::new();
    let mut reader = BufReader::new(&stream);
    reader.read_line(&mut buf)?;
    println!(
        "{:?}",
        read_serialized::<MessageFromServer>(buf.as_bytes()).unwrap()
    );

    std::thread::sleep(std::time::Duration::new(15, 0));

    Ok(())
}
