use std::{error::Error, net::TcpListener};

use protohackers::say_hello;

fn main() -> Result<(), Box<dyn Error>> {
    say_hello();
    let listener = TcpListener::bind("172.24.179.176:1337")?;
    println!("{:?}", listener.local_addr());
    for stream in listener.incoming() {
        let mut read_stream = stream?;
        let mut write_stream = read_stream.try_clone()?;
        std::thread::spawn(move || std::io::copy(&mut read_stream, &mut write_stream));
    }
    Ok(())
}
