use std::collections::BinaryHeap;

use color_eyre::{eyre::eyre, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{info, instrument, warn};

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
    let mut prices = BinaryHeap::new();
    loop {
        let mut bytes = [0; 9];
        stream.read_exact(&mut bytes).await?;
        match parse_bytes(bytes)? {
            Message::Insert(timestamp, price) => prices.push((timestamp, price)),
            Message::Query(low, high) => {
                let sorted_prices = prices.clone().into_sorted_vec();
                let average = get_average(low, high, &sorted_prices);
                stream.write_all(&average.to_be_bytes()).await?;
            }
        }
    }
}

#[derive(Debug)]
enum Message {
    Insert(i32, i32),
    Query(i32, i32),
}

#[instrument(ret)]
fn parse_bytes(bytes: [u8; 9]) -> Result<Message> {
    let message_type = bytes[0];
    let first_num = i32::from_be_bytes(bytes[1..5].try_into().unwrap());
    let second_num = i32::from_be_bytes(bytes[5..9].try_into().unwrap());
    match message_type {
        b'I' => Ok(Message::Insert(first_num, second_num)),
        b'Q' => Ok(Message::Query(first_num, second_num)),
        _ => Err(eyre!("Invalid message type")),
    }
}

#[instrument(ret, skip(sorted_prices))]
fn get_average(min_time: i32, max_time: i32, sorted_prices: &[(i32, i32)]) -> i32 {
    if min_time > max_time {
        warn!("mintime comes after maxtime");
        return 0;
    }
    let low_i = sorted_prices.partition_point(|&(t, _)| t < min_time);
    let high_i = sorted_prices.partition_point(|&(t, _)| t <= max_time);
    if low_i == high_i {
        warn!("no samples in that period");
        return 0;
    }
    let sum: i128 = sorted_prices[low_i..high_i]
        .iter()
        .map(|&(_, p)| p as i128)
        .sum();
    (sum / (high_i - low_i) as i128) as i32
}
