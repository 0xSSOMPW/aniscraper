use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use std::error::Error as StdError;
use std::fmt;

use crate::env::EnvVar;

/// Custom error enum to handle different types of errors
#[derive(Debug)]
pub enum AniRustError {
    /// Database error
    DatabaseError(diesel::result::Error),
    /// Reqwest error
    ReqwestError(reqwest::Error),
    /// No Proxies available error
    NoProxiesAvailable,
    /// Failed to fetch even after multiple tries error
    FailedToFetchAfterRetries,
    /// Parsing int error
    ParseIntError(std::num::ParseIntError),
}

// Implement Display for AniRustError
impl fmt::Display for AniRustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AniRustError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AniRustError::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
            AniRustError::NoProxiesAvailable => write!(f, "No proxies available"),
            AniRustError::FailedToFetchAfterRetries => write!(f, "Failed to fetch after retries"),
            AniRustError::ParseIntError(err) => write!(f, "Failed to parse int error: {}", err),
        }
    }
}

// Implement From trait to convert reqwest::Error to AniRustError
impl From<reqwest::Error> for AniRustError {
    fn from(err: reqwest::Error) -> Self {
        AniRustError::ReqwestError(err)
    }
}

// Implement From trait to convert diesel::result::Error to AniRustError
impl From<diesel::result::Error> for AniRustError {
    fn from(err: diesel::result::Error) -> Self {
        AniRustError::DatabaseError(err)
    }
}

// Implement the Error trait for AniRustError
impl StdError for AniRustError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AniRustError::DatabaseError(err) => Some(err),
            AniRustError::ReqwestError(err) => Some(err),
            AniRustError::NoProxiesAvailable => None,
            AniRustError::FailedToFetchAfterRetries => None,
            AniRustError::ParseIntError(err) => Some(err),
        }
    }
}

impl AniRustError {
    pub fn webhook_url(&self) -> String {
        match self {
            AniRustError::DatabaseError(_) => EnvVar::DATABASE_ERROR_WEBHOOK.get_config(),
            AniRustError::ReqwestError(_) => EnvVar::REQWEST_ERROR_WEBHOOK.get_config(),
            AniRustError::NoProxiesAvailable => {
                EnvVar::NO_PROXIES_AVAILABLE_ERROR_WEBHOOK.get_config()
            }
            AniRustError::FailedToFetchAfterRetries => {
                EnvVar::FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK.get_config()
            }
            AniRustError::ParseIntError(_) => EnvVar::UTILS_ERROR_WEBHOOK.get_config(),
        }
    }

    pub async fn send_error_to_webhook(webhook_url: &str, error_message: &str) {
        println!("0");
        if webhook_url.is_empty() {
            return;
        }

        let client = Client::new();
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.format("%a %b %e %T %Y").to_string();

        let content = format!(
            "{{\"timestamp\": \"{}\", \"error\": \"{}\"}}",
            timestamp, error_message
        );

        let payload = json!({
            "content": content,
        });

        // Perform the async HTTP request
        let response = client.post(webhook_url).json(&payload).send().await;
        println!("{:?}", response);
    }
}
