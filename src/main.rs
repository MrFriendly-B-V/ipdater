use clap::Parser;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::args::Args;
use crate::config::Config;
use color_eyre::eyre::Result;
use crate::storage::Storage;

mod storage;
mod config;
mod args;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();

    info!("Running..");
    debug!("Parsing args");
    let args = Args::parse();

    debug!("Reading config");
    let config = match args.config {
        Some(d) => Config::open_with_path(&d).await,
        None => Config::open().await
    }?;

    debug!("Reading storage file");
    let storage = match args.storage_dir {
        Some(d) => Storage::open_with_path(&d).await,
        None => Storage::open().await
    }?;

    
}


fn setup_tracing() {
    tracing_subscriber::registry()
        .with(layer().compact())
        .with(EnvFilter::from_default_env())
        .init();
}