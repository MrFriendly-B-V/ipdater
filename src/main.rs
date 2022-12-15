use crate::args::Args;
use crate::cloudflare_api::{
    create_record, get_dns_record_a_id, get_domain_clients, get_zone_id, update_record,
};
use crate::config::Config;
use crate::ip::get_current_ip;
use clap::Parser;
use cloudflare::framework::async_api::Client;
use color_eyre::eyre::{Error, Result};
use tracing::{debug, info, trace, warn};
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod args;
mod cloudflare_api;
mod config;
mod ip;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();

    info!("Running..");
    debug!("Parsing args");
    let args = Args::parse();

    if args.dry_run {
        warn!("Running in dry-run mode. Not actually updating any DNS records.");
    }

    debug!("Reading config");
    let config = match &args.config {
        Some(d) => Config::open_with_path(&d).await,
        None => Config::open().await,
    }?;
    debug!("Fetching current address");
    let current_address = get_current_ip(&config).await?;
    debug!("Current address is {current_address}");

    let domain_clients = get_domain_clients(&config)?;
    info!("Will update {} records.", domain_clients.len());

    for (domain, client) in domain_clients {
        handle_domain(domain, &client, args.clone(), current_address.clone()).await?;
    }

    info!("Done");

    Ok(())
}

async fn handle_domain(
    domain: String,
    client: &Client,
    args: Args,
    current_address: String,
) -> Result<()> {
    debug!("Updating domain: {domain}");

    // E.g. example.com for foo.bar.example.com
    let root_domain = get_root_domain(&domain)?;

    trace!("Fetching zone for root domain {root_domain}");
    let zone_id = get_zone_id(&client, &root_domain)
        .await?
        .ok_or(Error::msg(format!(
            "No Cloudflare zone exists for root domain {root_domain}"
        )))?;
    trace!("Found zone ID '{zone_id}' for zone with root domain {root_domain}");

    // All DNS records for the domain `domain`
    trace!("Listing DNS records in zone '{root_domain}' with ID '{zone_id}' for name: '{domain}'");
    let dns_id = get_dns_record_a_id(&client, &zone_id, &domain).await?;

    if let Some(dns_id) = dns_id {
        // Record exists, update it
        trace!("DNS record already exists. Updating it");

        if args.dry_run {
            info!("Dry run: would update {zone_id}/{dns_id} (domain) to {current_address}");
            return Ok(());
        }

        debug!("Updating DNS record: A {domain} '{current_address}'");
        update_record(&client, &zone_id, &dns_id, &domain, &current_address).await?;
        trace!("Record updated");
    } else {
        // Record does not exist, create it
        trace!("Record does not yet exist. Creating one.");

        if args.dry_run {
            info!("Dry run: would create in zone {zone_id} ({root_domain}) record {domain} with '{current_address}'");
            return Ok(());
        }

        debug!("Creating DNS record: A '{domain}' '{current_address}' in zone '{root_domain}' with zone ID '{zone_id}'");
        create_record(&client, &zone_id, &domain, &current_address).await?;
        trace!("Record created");
    }

    Ok(())
}

fn get_root_domain(domain: &str) -> Result<String> {
    let mut split = domain.split('.').collect::<Vec<_>>();

    // Top level domain 'com'
    let tld = split
        .pop()
        .ok_or(Error::msg("Invalid domain definition".to_string()))?;
    // second level domain 'example'
    let sld = split
        .pop()
        .ok_or(Error::msg("Invalid domain defintion".to_string()))?;

    Ok(format!("{sld}.{tld}"))
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(layer().compact())
        .with(EnvFilter::from_default_env())
        .init();
}
