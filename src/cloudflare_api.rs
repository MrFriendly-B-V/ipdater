use crate::config::Config;
use cloudflare::endpoints::dns::{
    CreateDnsRecord, CreateDnsRecordParams, DnsContent, DnsRecord, ListDnsRecords,
    ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams,
};
use cloudflare::endpoints::zone::{ListZones, ListZonesParams, Zone};
use cloudflare::framework::async_api::{ApiClient, Client};
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiSuccess;
use cloudflare::framework::{Environment, HttpApiClientConfig};
use color_eyre::eyre::{Error, Result};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str::FromStr;
use tracing::trace;

pub async fn create_record(
    client: &Client,
    zone_id: &str,
    domain: &str,
    address: &str,
) -> Result<()> {
    client
        .request_handle(&CreateDnsRecord {
            zone_identifier: &zone_id,
            params: CreateDnsRecordParams {
                name: &domain,
                content: DnsContent::A {
                    content: Ipv4Addr::from_str(&address)?,
                },
                proxied: Some(false),
                ttl: Some(60),
                priority: None,
            },
        })
        .await?;

    Ok(())
}

pub async fn update_record(
    client: &Client,
    zone_id: &str,
    dns_id: &str,
    domain: &str,
    address: &str,
) -> Result<()> {
    client
        .request_handle(&UpdateDnsRecord {
            zone_identifier: &zone_id,
            identifier: &dns_id,
            params: UpdateDnsRecordParams {
                content: DnsContent::A {
                    content: Ipv4Addr::from_str(&address)?,
                },
                ttl: Some(60),
                proxied: Some(false),
                name: &domain,
            },
        })
        .await?;
    Ok(())
}

pub async fn get_dns_record_a_id(
    client: &Client,
    zone_id: &str,
    domain: &str,
) -> Result<Option<String>> {
    let records: ApiSuccess<Vec<DnsRecord>> = client
        .request(&ListDnsRecords {
            zone_identifier: &zone_id,
            params: ListDnsRecordsParams {
                name: Some(domain.to_string()),
                ..Default::default()
            },
        })
        .await?;

    trace!("Found {} results of any kind", records.result.len());

    // CloudFlare DNS id
    let dns_id = records
        .result
        .into_iter()
        .filter(|x| match x.content {
            DnsContent::A { .. } => true,
            _ => false,
        })
        .map(|x| x.id)
        .collect::<Vec<_>>();

    trace!("Found {} A records", dns_id.len());

    Ok(dns_id.first().map(|x| x.clone()))
}

pub async fn get_zone_id(client: &Client, root_domain: &str) -> Result<Option<String>> {
    let zones: ApiSuccess<Vec<Zone>> = client
        .request_handle(&ListZones {
            params: ListZonesParams {
                name: Some(root_domain.to_string()),
                ..Default::default()
            },
        })
        .await?;

    // CloudFlare zone ID
    let zone_id = zones.result.into_iter().map(|x| x.id).collect::<Vec<_>>();

    Ok(zone_id.first().map(|x| x.clone()))
}

pub fn get_domain_clients(config: &Config) -> Result<HashMap<String, Client>> {
    Ok(config
        .zones
        .iter()
        .map(|zone| {
            let domains = zone
                .domains
                .iter()
                .map(|domain| {
                    let client = Client::new(
                        Credentials::UserAuthToken {
                            token: zone.credentials.key.clone(),
                        },
                        HttpApiClientConfig::default(),
                        Environment::Production,
                    )
                    .map_err(|e| Error::msg(format!("Failed to build CloudFlare client: {e:?}")))?;

                    Ok((domain.clone(), client))
                })
                .collect::<Result<HashMap<_, _>>>()?;

            Ok(domains)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<HashMap<_, _>>())
}
