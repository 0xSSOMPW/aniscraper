use chrono::{DateTime, Utc};
use diesel;
use reqwest::Client;
use serde_json::json;
use std::error::Error as StdError;
use std::fmt;
use tokio;

use crate::env::EnvVar;

/// Custom error enum to handle different types of errors
#[derive(Debug)]
pub enum AniRustError {
    /// Database error
    DatabaseError(diesel::result::Error),
    /// Reqwest error
    ReqwestError(reqwest::Error),
}

// Implement Display for AniRustError
impl fmt::Display for AniRustError {
    /// Formats the error message
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AniRustError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AniRustError::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
        }
    }
}

// Implement From trait to convert reqwest::Error to AniRustError
impl From<reqwest::Error> for AniRustError {
    /// Converts reqwest::Error to AniRustError
    fn from(err: reqwest::Error) -> Self {
        AniRustError::ReqwestError(err)
    }
}

// Implement From trait to convert diesel::result::Error to AniRustError
impl From<diesel::result::Error> for AniRustError {
    /// Converts diesel::result::Error to AniRustError
    fn from(err: diesel::result::Error) -> Self {
        AniRustError::DatabaseError(err)
    }
}

// Implement the Error trait for AniRustError
impl StdError for AniRustError {
    /// Returns the source of the error
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AniRustError::DatabaseError(err) => Some(err),
            AniRustError::ReqwestError(err) => Some(err),
        }
    }
}

impl AniRustError {
    /// Sends error details to a webhook
    async fn send_error_to_webhook(
        webhook_url: String,
        error_message: String,
    ) -> Result<(), reqwest::Error> {
        // Skip sending if the webhook_url is empty
        if webhook_url.is_empty() {
            return Ok(());
        }

        let client = Client::new();

        // Get the current date and time
        let now: DateTime<Utc> = Utc::now();
        let timestamp = now.format("%a %b %e %T %Y").to_string();

        // Add the timestamp to the error message
        let payload = json!({
            "error": error_message,
            "timestamp": timestamp
        });

        client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map(|_| ())
    }

    /// Chooses the appropriate webhook URL based on the error type
    fn webhook_url(&self) -> String {
        match self {
            AniRustError::DatabaseError(_) => EnvVar::DATABASE_ERROR_WEBHOOK.get_config(),
            AniRustError::ReqwestError(_) => EnvVar::REQWEST_ERROR_WEBHOOK.get_config(),
        }
    }
}

impl Drop for AniRustError {
    /// Sends error details to a webhook when the error is dropped
    fn drop(&mut self) {
        // Clone error message for use in async task
        let error_message = self.to_string();
        let webhook_url = self.webhook_url();

        // Spawn an async task to send the error details to the webhook
        let future = AniRustError::send_error_to_webhook(webhook_url, error_message);

        // Use an async runtime to handle the async task
        let _ = tokio::spawn(async {
            if let Err(e) = future.await {
                eprintln!("Failed to send error to webhook: {}", e);
            }
        });
    }
}
