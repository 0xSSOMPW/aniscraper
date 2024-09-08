use crate::{
    env::EnvVar,
    error::AniRustError,
    proxy::{get_random_proxy, Proxy},
};
use brotli::Decompressor;
use core::fmt;
use flate2::read::{GzDecoder, ZlibDecoder};
use reqwest::header;
use std::error::Error;
use std::io::Read;
use std::time::Duration;

/// Fetches data from the specified URL.
///
/// Returns the HTML content of the page as a string.
pub async fn get_curl(url: &str, proxies: &[Proxy]) -> Result<String, Box<dyn Error>> {
    let max_attempts = parse_usize(&EnvVar::MAX_RETRIES_ATTEMPTS.get_config()).unwrap_or(50);
    let timeout_duration = Duration::from_secs(5);

    let mut attempt = 0;
    while attempt < max_attempts {
        let client = if proxies.is_empty() {
            reqwest::Client::builder()
                .timeout(timeout_duration)
                .build()?
        } else {
            let proxy = get_random_proxy(proxies).unwrap();
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::http(&proxy.address)?)
                .timeout(timeout_duration)
                .build()?
        };

        let response = match client
            .get(url)
            .header(header::USER_AGENT, &EnvVar::USER_AGENT_HEADER.get_config())
            .header(
                header::ACCEPT_ENCODING,
                &EnvVar::ACCEPT_ENCODING_HEADER.get_config(),
            )
            .header(header::ACCEPT, &EnvVar::ACCEPT_HEADER.get_config())
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_) if attempt < max_attempts - 1 => {
                attempt += 1;
                continue;
            }
            Err(e) => return Err(Box::new(e)),
        };

        let res_headers = response.headers().to_owned();
        let content_encoding = res_headers
            .get(header::CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let res_bytes = response.bytes().await?;

        let body = match content_encoding {
            "gzip" => {
                let mut decoded = Vec::new();
                let mut gz = GzDecoder::new(res_bytes.as_ref());
                gz.read_to_end(&mut decoded)?;
                String::from_utf8(decoded)?
            }
            "deflate" => {
                let mut zf = ZlibDecoder::new(res_bytes.as_ref());
                let mut decoded = Vec::new();
                zf.read_to_end(&mut decoded)?;
                String::from_utf8(decoded)?
            }
            "br" => {
                let mut decompressor = Decompressor::new(res_bytes.as_ref(), 4096);
                let mut decoded = Vec::new();
                decompressor.read_to_end(&mut decoded)?;
                String::from_utf8(decoded)?
            }
            _ => String::from_utf8(res_bytes.to_vec())?,
        };

        return Ok(body);
    }

    Err(Box::new(AniRustError::FailedToFetchAfterRetries))
}

pub fn parse_usize(s: &str) -> Result<usize, AniRustError> {
    s.to_string()
        .parse::<usize>()
        .map_err(AniRustError::ParseIntError)
}

pub fn stringify<T: fmt::Display>(input: T) -> String {
    format!("{}", input)
}

pub fn opt_box_error_vec_to_string(error_vec: Vec<Option<Box<dyn Error>>>) -> String {
    error_vec
        .iter()
        .filter_map(|opt_error| opt_error.as_ref().map(|e| e.to_string()))
        .collect::<Vec<String>>()
        .join(", ")
}
