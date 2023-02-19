use std::collections::HashMap;

use color_eyre::Result;
use tokio::sync::mpsc::Receiver;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
    sync::mpsc::{self, Sender},
};
use tracing::{info, instrument, warn};

#[derive(Debug)]
enum Event {
    NewUser(String, OwnedWriteHalf),
    ChatMessage(String, String),
    UserLeaves(String),
}

#[instrument]
pub async fn run_server(ip: &str) -> Result<()> {
    let listener = TcpListener::bind(format!("{ip}:1337")).await?;
    info!("listening at {ip}:1337");

    let (tx, rx) = mpsc::channel::<Event>(100);
    tokio::spawn(run_message_writer(rx));
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_stream(stream, tx.clone()));
    }
}

#[instrument(skip_all)]
async fn run_message_writer(mut rx: Receiver<Event>) -> Result<()> {
    let mut joined_users = HashMap::new();
    while let Some(e) = rx.recv().await {
        match e {
            Event::NewUser(username, mut writer) => {
                // send first so we don't send to new user
                send_to_all(
                    &format!("* {} joined!\n", username),
                    joined_users.values_mut(),
                )
                .await?;

                let current_users = joined_users.keys().cloned().collect::<Vec<_>>();
                let message = format!("* Already here: {}\n", current_users.join(", "));
                writer.write_all(message.as_bytes()).await?;

                joined_users.insert(username, writer);
            }
            Event::ChatMessage(username, message) => {
                let all_but_sender = joined_users
                    .iter_mut()
                    .filter(|(n, _)| **n != username)
                    .map(|(_, w)| w);

                let message = format!("[{username}] {message}\n");
                send_to_all(&message, all_but_sender).await?;
            }
            Event::UserLeaves(username) => {
                // leave first so we don't send to leaving user
                joined_users.remove(&username);
                send_to_all(
                    &format!("* {} left!\n", username),
                    joined_users.values_mut(),
                )
                .await?;
            }
        }
    }
    Ok(())
}

#[instrument(skip(send_to))]
async fn send_to_all(
    message: &str,
    send_to: impl Iterator<Item = &mut OwnedWriteHalf>,
) -> Result<()> {
    for writer in send_to {
        writer.write_all(message.as_bytes()).await?;
    }
    Ok(())
}

#[instrument(skip_all)]
async fn handle_stream(stream: TcpStream, event_sender: Sender<Event>) -> Result<()> {
    let (read_stream, mut write_stream) = stream.into_split();

    write_stream.write_all(b"What is your name?\n").await?;
    let mut lines = BufReader::new(read_stream).lines();
    let Some(name) = lines.next_line().await? else {
        return Ok(());
    };

    if name.bytes().any(|b| !b.is_ascii_alphanumeric()) {
        warn!("Invalid name: {name}");
        write_stream.write_all(b"Invalid name!\n").await?;
        return Ok(());
    }
    event_sender
        .send(Event::NewUser(name.clone(), write_stream))
        .await?;

    while let Some(message) = lines.next_line().await? {
        event_sender
            .send(Event::ChatMessage(name.clone(), message))
            .await?;
    }

    event_sender.send(Event::UserLeaves(name)).await?;
    Ok(())
}
