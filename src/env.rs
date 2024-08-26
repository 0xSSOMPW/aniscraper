// env.rs

use std::env;
use std::fmt;

use dotenvy::dotenv;

#[derive(Debug)]
pub enum EnvVar {
    DATABASE_URL,
    MAX_RETRIES_ATTEMPTS,
    DATABASE_ERROR_WEBHOOK,
    REQWEST_ERROR_WEBHOOK,
    NO_PROXIES_AVAILABLE_ERROR_WEBHOOK,
    FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK,
    UTILS_ERROR_WEBHOOK,
    HTTP_URL,
    SOCK4_URL,
    SOCK5_URL,
}

impl EnvVar {
    // Convert EnvVar to the corresponding environment variable key
    fn as_str(&self) -> &'static str {
        match self {
            EnvVar::DATABASE_URL => "DATABASE_URL",
            EnvVar::MAX_RETRIES_ATTEMPTS => "MAX_RETRIES_ATTEMPTS",
            EnvVar::DATABASE_ERROR_WEBHOOK => "DATABASE_ERROR_WEBHOOK",
            EnvVar::REQWEST_ERROR_WEBHOOK => "REQWEST_ERROR_WEBHOOK",
            EnvVar::NO_PROXIES_AVAILABLE_ERROR_WEBHOOK => "NO_PROXIES_AVAILABLE_ERROR_WEBHOOK",
            EnvVar::FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK => {
                "FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK"
            }
            EnvVar::UTILS_ERROR_WEBHOOK => "UTILS_ERROR_WEBHOOK",
            EnvVar::HTTP_URL => "HTTP_URL",
            EnvVar::SOCK4_URL => "SOCK4_URL",
            EnvVar::SOCK5_URL => "SOCK5_URL",
        }
    }

    // Fetch the environment variable value and return it as a String
    pub fn get_config(&self) -> String {
        dotenv().ok();
        let key = self.as_str();

        match env::var(key) {
            Ok(val) => val,
            Err(_) => match self {
                EnvVar::DATABASE_URL => panic!("DATABASE_URL is not set"),
                _ => String::new(),
            },
        }
    }
}

#[derive(Debug)]
pub enum AppConfig {
    DatabaseUrl(String),
    MaxRetriesAttempts(usize),
    DatabaseErrorWebhook(Option<String>),
    ReqwestErrorWebhook(Option<String>),
    UtilsErrorWebhook(Option<String>),
    NoProxiesAvailable(Option<String>),
    FailedToFetchAfterRetries(Option<String>),
    HttpUrl(String),
    Sock4Url(String),
    Sock5Url(String),
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppConfig::DatabaseUrl(url) => write!(f, "Database URL: {}", url),
            AppConfig::MaxRetriesAttempts(no) => write!(f, "Max Retries Attempts: {}", no),
            AppConfig::DatabaseErrorWebhook(webhook) => {
                write!(f, "Database Error Webhook: {:?}", webhook)
            }
            AppConfig::ReqwestErrorWebhook(webhook) => {
                write!(f, "Reqwest Error Webhook: {:?}", webhook)
            }
            AppConfig::UtilsErrorWebhook(webhook) => {
                write!(f, "Utils Error Webhook: {:?}", webhook)
            }
            AppConfig::NoProxiesAvailable(webhook) => {
                write!(f, "No proxies available Error Webhook: {:?}", webhook)
            }
            AppConfig::FailedToFetchAfterRetries(webhook) => {
                write!(
                    f,
                    "Failed to fetch after retries Error Webhook: {:?}",
                    webhook
                )
            }
            AppConfig::HttpUrl(url) => {
                write!(f, "Http proxy Url: {}", url)
            }
            AppConfig::Sock4Url(url) => {
                write!(f, "Sock4 proxy Url: {}", url)
            }
            AppConfig::Sock5Url(url) => {
                write!(f, "Sock5 proxy Url: {}", url)
            }
        }
    }
}
