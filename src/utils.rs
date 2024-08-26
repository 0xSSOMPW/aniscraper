use crate::{
    env::EnvVar,
    error::AniRustError,
    handle_error,
    proxy::{get_random_proxy, Proxy},
};
use core::fmt;
use reqwest::Client;
use std::time::Duration;

/// Fetches data from the specified URL.
///
/// Returns the HTML content of the page as a string.
pub async fn get_curl(url: &str, proxies: &[Proxy]) -> Result<String, AniRustError> {
    let max_attempts = parse_usize(&EnvVar::MAX_RETRIES_ATTEMPTS.get_config())?;
    let timeout_duration = Duration::from_secs(5);

    for _ in 0..max_attempts {
        if let Some(proxy) = get_random_proxy(proxies) {
            let client = Client::builder()
                .proxy(reqwest::Proxy::http(&proxy.address)?)
                .timeout(timeout_duration)
                .build()?;

            let response = client.get(url).send().await?;
            if response.status().is_success() {
                return Ok(response.text().await?);
            }
        } else {
            return Err(AniRustError::NoProxiesAvailable);
        }
    }

    Err(AniRustError::FailedToFetchAfterRetries)
}

pub fn parse_usize(s: &str) -> Result<usize, AniRustError> {
    handle_error!(s
        .to_string()
        .parse::<usize>()
        .map_err(AniRustError::ParseIntError))
}

pub fn stringify<T: fmt::Display>(input: T) -> String {
    format!("{}", input)
}
