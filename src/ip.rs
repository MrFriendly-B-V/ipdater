use crate::config::Config;
use color_eyre::eyre::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;

pub async fn get_current_ip(config: &Config) -> Result<String> {
    let client = get_http_client()?;
    let ip = client
        .get(&config.ipserver)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(ip.replace('\n', ""))
}

fn get_http_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "User-Agent",
        HeaderValue::from_str(&format!("ipdater v{}", env!("CARGO_PKG_VERSION")))?,
    );

    let http_client = Client::builder().default_headers(headers).build()?;

    Ok(http_client)
}
