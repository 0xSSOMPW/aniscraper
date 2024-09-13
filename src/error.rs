use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde_json::json;
use std::error::Error as StdError;
use std::fmt;

use crate::env::EnvVar;

/// Custom error enum to handle different types of errors
#[derive(Debug)]
pub enum AniRustError {
    /// Reqwest error
    ReqwestError(reqwest::Error),
    HyperError(hyper::Error),
    /// No Proxies available error
    NoProxiesAvailable,
    /// Failed to fetch even after multiple tries error
    FailedToFetchAfterRetries,
    /// Parsing int error
    ParseIntError(std::num::ParseIntError),
    /// No domain exists
    NoDomainExists(String),
    /// all rest errors
    UnknownError(String),
}

// Implement Display for AniRustError
impl fmt::Display for AniRustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AniRustError::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
            AniRustError::HyperError(err) => write!(f, "Reqwest error: {}", err),
            AniRustError::NoProxiesAvailable => write!(f, "No proxies available"),
            AniRustError::FailedToFetchAfterRetries => write!(f, "Failed to fetch after retries"),
            AniRustError::ParseIntError(err) => write!(f, "Failed to parse int error: {}", err),
            AniRustError::NoDomainExists(site) => write!(f, "No domain added for: {}", site),
            AniRustError::UnknownError(err) => write!(f, "Std error occured: {}", err),
        }
    }
}

// Implement From trait to convert reqwest::Error to AniRustError
impl From<reqwest::Error> for AniRustError {
    fn from(err: reqwest::Error) -> Self {
        AniRustError::ReqwestError(err)
    }
}

impl From<hyper::Error> for AniRustError {
    fn from(err: hyper::Error) -> Self {
        AniRustError::HyperError(err)
    }
}

// Implement `From<Box<dyn StdError>>` for `CustomError`
impl From<Box<dyn StdError>> for AniRustError {
    fn from(err: Box<dyn StdError>) -> Self {
        AniRustError::UnknownError(err.to_string())
    }
}

// Implement the Error trait for AniRustError
impl StdError for AniRustError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AniRustError::ReqwestError(err) => Some(err),
            AniRustError::HyperError(err) => Some(err),
            AniRustError::NoProxiesAvailable => None,
            AniRustError::FailedToFetchAfterRetries => None,
            AniRustError::ParseIntError(err) => Some(err),
            AniRustError::NoDomainExists(_) => None,
            AniRustError::UnknownError(_) => None,
        }
    }
}

impl AniRustError {
    pub fn webhook_url(&self) -> String {
        match self {
            AniRustError::ReqwestError(_) => EnvVar::REQWEST_ERROR_WEBHOOK.get_config(),
            AniRustError::HyperError(_) => EnvVar::REQWEST_ERROR_WEBHOOK.get_config(),
            AniRustError::NoProxiesAvailable => {
                EnvVar::NO_PROXIES_AVAILABLE_ERROR_WEBHOOK.get_config()
            }
            AniRustError::FailedToFetchAfterRetries => {
                EnvVar::FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK.get_config()
            }
            AniRustError::ParseIntError(_) => EnvVar::UTILS_ERROR_WEBHOOK.get_config(),
            AniRustError::NoDomainExists(_) => String::new(),
            AniRustError::UnknownError(_) => EnvVar::UNKNOWN_ERROR_WEBHOOK.get_config(),
        }
    }
}

impl Drop for AniRustError {
    fn drop(&mut self) {
        let webhook_url = self.webhook_url();
        let error_message = format!("{}", self);
        send_error_to_webhook(&webhook_url, &error_message);
    }
}

fn send_error_to_webhook(webhook_url: &str, error_message: &str) {
    if webhook_url.is_empty() {
        return;
    }

    let webhook_url = webhook_url.to_string();
    let error_message = error_message.to_string();

    tokio::task::spawn_blocking(move || {
        // Ensure the webhook URL is not empty
        if webhook_url.is_empty() {
            return;
        }

        let client = Client::new();
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let content = format!(
            r#"{{"Timestamp": "{}", "Error": "{}"}}"#,
            timestamp, error_message
        );

        let payload = json!({
            "content": content,
        });

        // Perform the blocking HTTP request
        let _res = client.post(&webhook_url).json(&payload).send();
    });
}
