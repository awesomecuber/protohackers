use std::{
    error::Error,
    io::Write,
    io::{BufRead, BufReader},
    net::TcpListener,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Debug, Serialize)]
struct Response {
    method: String,
    prime: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("172.24.179.176:1337")?;
    println!("{:?}", listener.local_addr());
    for stream in listener.incoming() {
        println!("stream incoming!");
        let stream = stream?;
        let reader = BufReader::new(stream.try_clone()?);
        std::thread::spawn(|| handle_lines(reader, stream).unwrap());
    }
    Ok(())
}

fn handle_lines(reader: impl BufRead, mut writer: impl Write) -> Result<(), Box<dyn Error>> {
    for line in reader.lines() {
        println!("line incoming! {line:?}");
        let request: Request = serde_json::from_str(&line?)?;
        println!("parsed! {request:?}");
        if &request.method != "isPrime" {
            writer.write_all(&[69, 42])?;
            return Ok(());
        }
        let response: Response = Response {
            method: "isPrime".to_owned(),
            prime: is_prime(request.number),
        };
        println!("response constructed! {response:?}");
        let mut bytes = serde_json::to_vec(&response)?;
        bytes.push(b'\n');
        writer.write_all(&bytes)?;
        println!("response sent!");
    }
    Ok(())
}

fn is_prime(num: f64) -> bool {
    if num != num.round() {
        return false;
    }
    let num = num as i64;

    if num <= 1 {
        return false;
    }
    if num == 2 {
        return true;
    }

    let mut is_prime = true;
    for i in 2..=((num as f64).sqrt() as i64) {
        if num % i == 0 {
            // not prime, get out
            is_prime = false;
            break;
        }
    }
    is_prime
}
