use color_eyre::Result;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, instrument};

#[instrument]
pub async fn run_server(ip: &str) -> Result<()> {
    let listener = TcpListener::bind(format!("{ip}:1337")).await?;
    info!("listening at {ip}:1337");
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_stream(stream));
    }
}

#[instrument(skip_all)]
async fn handle_stream(mut stream: TcpStream) -> Result<()> {
    let (mut read_stream, mut write_stream) = stream.split();
    tokio::io::copy(&mut read_stream, &mut write_stream).await?;
    Ok(())
}
