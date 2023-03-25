use color_eyre::Result;
use problems::*;
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
        .with(HierarchicalLayer::new(4))
        .with(tracing_error::ErrorLayer::default())
        .init();

    let tcp_ip = "0.0.0.0";
    // let udp_ip = "fly-global-services";
    let udp_ip = "0.0.0.0";

    tokio::try_join!(
        p00_smoke_test::run_server(tcp_ip, 9000),
        p01_prime_time::run_server(tcp_ip, 9001),
        p02_means_to_an_end::run_server(tcp_ip, 9002),
        p03_budget_chat::run_server(tcp_ip, 9003),
        p04_unusual_database_program::run_server(udp_ip, 9004),
    )?;

    Ok(())
}
