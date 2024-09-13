// env.rs

use std::env;
use std::fmt;
use std::sync::Mutex;

use dotenvy::dotenv;
use lazy_static::lazy_static;

use crate::error::AniRustError;

lazy_static! {
    pub static ref SECRET: Mutex<Option<SecretConfig>> = Mutex::new(None);
}

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

#[derive(Debug, Clone)]
pub struct SecretConfig {
    pub max_retries_attempts: String,
    pub reqwest_error_webhook: String,
    pub no_proxies_available_error_webhook: String,
    pub failed_to_fetch_after_retries_error_webhook: String,
    pub utils_error_webhook: String,
    pub unknown_error_webhook: String,
    pub http_url: String,
    pub sock4_url: String,
    pub sock5_url: String,
    pub hianime_domains: String,
    pub user_agent_header: String,
    pub accept_encoding_header: String,
    pub accept_header: String,
}

impl SecretConfig {
    pub fn new(
        max_retries_attempts: String,
        reqwest_error_webhook: String,
        no_proxies_available_error_webhook: String,
        failed_to_fetch_after_retries_error_webhook: String,
        utils_error_webhook: String,
        unknown_error_webhook: String,
        http_url: String,
        sock4_url: String,
        sock5_url: String,
        hianime_domains: String,
        user_agent_header: String,
        accept_encoding_header: String,
        accept_header: String,
    ) -> Self {
        Self {
            max_retries_attempts,
            reqwest_error_webhook,
            no_proxies_available_error_webhook,
            failed_to_fetch_after_retries_error_webhook,
            utils_error_webhook,
            unknown_error_webhook,
            http_url,
            sock4_url,
            sock5_url,
            hianime_domains,
            user_agent_header,
            accept_encoding_header,
            accept_header,
        }
    }

    pub fn webhook_url(&self, error: AniRustError) -> String {
        match error {
            AniRustError::ReqwestError(_) => self.reqwest_error_webhook.clone(),
            AniRustError::HyperError(_) => self.reqwest_error_webhook.clone(),
            AniRustError::NoProxiesAvailable => self.no_proxies_available_error_webhook.clone(),
            AniRustError::FailedToFetchAfterRetries => {
                self.failed_to_fetch_after_retries_error_webhook.clone()
            }
            AniRustError::ParseIntError(_) => self.utils_error_webhook.clone(),
            AniRustError::NoDomainExists(_) => String::new(),
            AniRustError::UnknownError(_) => self.unknown_error_webhook.clone(),
        }
    }
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
        let key = self.as_str();

        // Attempt to get the value from the SECRET
        let secret = SECRET.lock().unwrap();

        if let Some(s) = &*secret {
            match self {
                EnvVar::MAX_RETRIES_ATTEMPTS => return s.max_retries_attempts.clone(),
                EnvVar::REQWEST_ERROR_WEBHOOK => return s.reqwest_error_webhook.clone(),
                EnvVar::NO_PROXIES_AVAILABLE_ERROR_WEBHOOK => {
                    return s.no_proxies_available_error_webhook.clone()
                }
                EnvVar::FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK => {
                    return s.failed_to_fetch_after_retries_error_webhook.clone()
                }
                EnvVar::UTILS_ERROR_WEBHOOK => return s.utils_error_webhook.clone(),
                EnvVar::HTTP_URL => return s.http_url.clone(),
                EnvVar::SOCK4_URL => return s.sock4_url.clone(),
                EnvVar::SOCK5_URL => return s.sock5_url.clone(),
                EnvVar::HIANIME_DOMAINS => return s.hianime_domains.clone(),
                EnvVar::USER_AGENT_HEADER => return s.user_agent_header.clone(),
                EnvVar::ACCEPT_ENCODING_HEADER => return s.accept_encoding_header.clone(),
                EnvVar::ACCEPT_HEADER => return s.accept_header.clone(),
                EnvVar::UNKNOWN_ERROR_WEBHOOK => return s.unknown_error_webhook.clone(),
            }
        }
        // Release the lock.
        drop(secret);

        // Load environment variables from a .env file
        dotenv().ok();

        // Fallback to environment variable if secret is not set
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
