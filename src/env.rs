// env.rs

use std::env;
use std::fmt;

use dotenvy::dotenv;

#[derive(Debug)]
pub enum EnvVar {
    MAX_RETRIES_ATTEMPTS,
    REQWEST_ERROR_WEBHOOK,
    NO_PROXIES_AVAILABLE_ERROR_WEBHOOK,
    FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK,
    UTILS_ERROR_WEBHOOK,
    UNKNOWN_ERROR_WEBHOOK,
    HTTP_URL,
    SOCK4_URL,
    SOCK5_URL,
    HIANIME_DOMAINS,
    USER_AGENT_HEADER,
    ACCEPT_ENCODING_HEADER,
    ACCEPT_HEADER,
}

impl EnvVar {
    // Convert EnvVar to the corresponding environment variable key
    fn as_str(&self) -> &'static str {
        match self {
            EnvVar::MAX_RETRIES_ATTEMPTS => "MAX_RETRIES_ATTEMPTS",
            EnvVar::REQWEST_ERROR_WEBHOOK => "REQWEST_ERROR_WEBHOOK",
            EnvVar::NO_PROXIES_AVAILABLE_ERROR_WEBHOOK => "NO_PROXIES_AVAILABLE_ERROR_WEBHOOK",
            EnvVar::FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK => {
                "FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK"
            }
            EnvVar::UTILS_ERROR_WEBHOOK => "UTILS_ERROR_WEBHOOK",
            EnvVar::HTTP_URL => "HTTP_URL",
            EnvVar::SOCK4_URL => "SOCK4_URL",
            EnvVar::SOCK5_URL => "SOCK5_URL",
            EnvVar::HIANIME_DOMAINS => "HIANIME_DOMAINS",
            EnvVar::USER_AGENT_HEADER => "USER_AGENT_HEADER",
            EnvVar::ACCEPT_ENCODING_HEADER => "ACCEPT_ENCODING_HEADER",
            EnvVar::ACCEPT_HEADER => "ACCEPT_HEADER",
            EnvVar::UNKNOWN_ERROR_WEBHOOK => "UNKNOWN_ERROR_WEBHOOK",
        }
    }

    // Fetch the environment variable value and return it as a String
    pub fn get_config(&self) -> String {
        dotenv().ok();
        let key = self.as_str();

        match env::var(key) {
            Ok(val) => val,
            Err(_) => String::new(),
        }
    }
}

#[derive(Debug)]
pub enum AppConfig {
    MaxRetriesAttempts(usize),
    ReqwestErrorWebhook(Option<String>),
    UtilsErrorWebhook(Option<String>),
    UnknownErrorWebhhok(Option<String>),
    NoProxiesAvailable(Option<String>),
    FailedToFetchAfterRetries(Option<String>),
    HttpUrl(String),
    Sock4Url(String),
    Sock5Url(String),
    HianimeDomains(String),
    UserAgentHeader(String),
    AcceptEncodingHeader(String),
    AccpetHeader(String),
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppConfig::MaxRetriesAttempts(no) => write!(f, "Max Retries Attempts: {}", no),
            AppConfig::ReqwestErrorWebhook(webhook) => {
                write!(f, "Reqwest Error Webhook: {:?}", webhook)
            }
            AppConfig::UtilsErrorWebhook(webhook) => {
                write!(f, "Utils Error Webhook: {:?}", webhook)
            }
            AppConfig::UnknownErrorWebhhok(webhook) => {
                write!(f, "Unknown Error Webhook: {:?}", webhook)
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
            AppConfig::HianimeDomains(domains) => {
                write!(f, "HiAnime domains: {}", domains)
            }
            AppConfig::UserAgentHeader(header) => {
                write!(f, "User Agent Header: {}", header)
            }
            AppConfig::AcceptEncodingHeader(header) => {
                write!(f, "Accept encoding Header: {}", header)
            }
            AppConfig::AccpetHeader(header) => {
                write!(f, "Accept Header: {}", header)
            }
        }
    }
}
