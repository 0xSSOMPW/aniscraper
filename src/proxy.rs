use rand::seq::SliceRandom;
use reqwest::Client;

use crate::{env::EnvVar, error::AniRustError};

// Define a struct to hold proxy data
#[derive(Debug, Clone)]
pub struct Proxy {
    pub address: String,
}

// Function to get a random proxy from the list
pub fn get_random_proxy(proxies: &[Proxy]) -> Option<Proxy> {
    proxies.choose(&mut rand::thread_rng()).cloned()
}

// Fetch proxy list from URL
pub async fn fetch_proxy_list(url: &str) -> Result<Vec<Proxy>, AniRustError> {
    let client = Client::new();
    let response = client.get(url).send().await?.text().await?;
    let proxies = response
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.is_empty() {
                Some(Proxy {
                    address: line.to_string(),
                })
            } else {
                None
            }
        })
        .collect();
    Ok(proxies)
}

// Load proxies from multiple sources
pub async fn load_proxies() -> Result<Vec<Proxy>, AniRustError> {
    let sock5_url = EnvVar::SOCK5_URL.get_config();
    let sock4_url = EnvVar::SOCK4_URL.get_config();
    let http_url = EnvVar::HTTP_URL.get_config();

    let sock5_proxies = fetch_proxy_list(&sock5_url).await?;
    let sock4_proxies = fetch_proxy_list(&sock4_url).await?;
    let http_proxies = fetch_proxy_list(&http_url).await?;

    let mut all_proxies = Vec::new();
    all_proxies.extend(sock5_proxies);
    all_proxies.extend(sock4_proxies);
    all_proxies.extend(http_proxies);

    Ok(all_proxies)
}
