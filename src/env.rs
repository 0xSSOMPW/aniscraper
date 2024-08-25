// env.rs

use std::env;
use std::fmt;

use dotenvy::dotenv;

#[derive(Debug)]
pub enum EnvVar {
    DATABASE_URL,
    DATABASE_ERROR_WEBHOOK,
    REQWEST_ERROR_WEBHOOK,
}

impl EnvVar {
    // Convert EnvVar to the corresponding environment variable key
    fn as_str(&self) -> &'static str {
        match self {
            EnvVar::DATABASE_URL => "DATABASE_URL",
            EnvVar::DATABASE_ERROR_WEBHOOK => "DATABASE_ERROR_WEBHOOK",
            EnvVar::REQWEST_ERROR_WEBHOOK => "REQWEST_ERROR_WEBHOOK",
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
    DatabaseErrorWebhook(Option<String>),
    ReqwestErrorWebhook(Option<String>),
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppConfig::DatabaseUrl(url) => write!(f, "Database URL: {}", url),
            AppConfig::DatabaseErrorWebhook(webhook) => {
                write!(f, "Database Error Webhook: {:?}", webhook)
            }
            AppConfig::ReqwestErrorWebhook(webhook) => {
                write!(f, "Reqwest Error Webhook: {:?}", webhook)
            }
        }
    }
}
