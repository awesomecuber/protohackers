use color_eyre::eyre::Context;
use color_eyre::{eyre::eyre, Result};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

fn main() -> Result<()> {
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

    let day: u32 = std::env::args()
        .nth(1)
        .ok_or_else(|| eyre!("Must specify problem number"))?
        .parse()
        .context("Couldn't parse problem number as a number")?;

    Ok(())
}
