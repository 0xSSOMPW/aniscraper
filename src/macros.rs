#[macro_export]
macro_rules! handle_error {
    ($result:expr) => {{
        use chrono::{DateTime, Utc};
        use reqwest::blocking::Client;
        use serde_json::json;

        // Capture the result and its error, if any
        let result = $result;

        // Spawn an async task to handle errors
        if let Err(e) = &result {
            let error_message = format!("{}", e);
            let webhook_url = e.webhook_url();

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
                let _res = client
                    .post(&webhook_url)
                    .json(&payload)
                    .send();
            });
        }

        result
    }};
}
