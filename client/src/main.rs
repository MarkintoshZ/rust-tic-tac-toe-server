use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1337")?;

    stream.write("hello 1\n".as_bytes())?;

    let mut buf = String::new();
    let mut reader = BufReader::new(&stream);
    reader.read_line(&mut buf)?;
    println!("{:?}", buf);

    Ok(())
}
