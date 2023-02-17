use std::str;

use color_eyre::eyre::WrapErr;
use color_eyre::{eyre::eyre, Result};
use problems::*;
use tokio::process::Command;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod problems;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(tracing_error::ErrorLayer::default())
        .init();

    let problem: u32 = std::env::args()
        .nth(1)
        .ok_or_else(|| eyre!("Must specify problem number"))?
        .parse()
        .wrap_err("Couldn't parse problem number as a number")?;

    let ip = Command::new("hostname").arg("-I").output().await?.stdout;
    let ip = str::from_utf8(&ip)?.trim();

    match problem {
        0 => p00_smoke_test::run_server(ip).await?,
        1 => p01_prime_time::run_server(ip).await?,
        2 => todo!(),
        _ => return Err(eyre!("Invalid problem number")),
    }

    Ok(())
}
