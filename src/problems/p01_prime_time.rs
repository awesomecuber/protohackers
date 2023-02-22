use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info, instrument, warn};

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

#[instrument]
pub async fn run_server(ip: &str, port: u32) -> Result<()> {
    let listener = TcpListener::bind(format!("{ip}:{port}")).await?;
    info!("listening at {ip}:{port}");
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_stream(stream));
    }
}

#[instrument(skip_all)]
async fn handle_stream(mut stream: TcpStream) -> Result<()> {
    let (read_stream, mut write_stream) = stream.split();
    let mut lines = BufReader::new(read_stream).lines();
    while let Some(line) = lines.next_line().await? {
        let mut response = match get_response_to_line(&line) {
            Ok(response) => {
                debug!("responding to valid request",);
                response
            }
            Err(err) => {
                warn!(
                    "request had error, responding with malformed response: {:#}",
                    err
                );
                vec![69, 42]
            }
        };
        response.push(b'\n');
        write_stream.write_all(&response).await?;
    }

    Ok(())
}

#[instrument]
fn get_response_to_line(line: &str) -> Result<Vec<u8>> {
    let request: Request =
        serde_json::from_str(line).wrap_err("Could not parse request into JSON")?;
    debug!("request: {request:?}");

    if &request.method != "isPrime" {
        return Err(eyre!("Method isn't isPrime"));
    }

    let response = Response {
        method: "isPrime".to_owned(),
        prime: is_prime(request.number),
    };
    debug!("response: {response:?}");

    Ok(serde_json::to_vec(&response).unwrap())
}

#[instrument(ret)]
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
