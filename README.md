<p align="center"><img src="https://github.com/user-attachments/assets/87ace4a6-a3ed-4c7c-903c-2b055628cc54" width="175"/></p>

<h1 align="center">aniscraper</h1>

<p align="center">
  A Rust library for scraping data with proxy support and error handling.
</p>

## Index

- [Installation](#installation)
- [Configuration](#configuration)
  - [Using `.env` file](#using-env-file)
  - [Manual Configuration](#manual-configuration)
- [Usage](#usage)
- [License](#license)
- [Contributing](#contributing)

##  <span id="installation">Installation</span>

To get started with `aniscraper`, first add it to your `Cargo.toml`:

```toml
[dependencies]
aniscraper = "0.1"  # Replace with the latest version
```

Or

```bash
cargo add aniscraper
```

Then, run `cargo build` to fetch and compile the library.

##  <span id="configuration">Configuration</span>

Before using `aniscraper`, you need to configure it with the necessary parameters. You can do this in two ways:

##  <span id="using-env-file">Using `.env` File</span>

Create a `.env` file in the root of your project directory and add the following environment variables:

```env
# Required
USER_AGENT_HEADER=xxx
ACCEPT_ENCODING_HEADER=xxx
ACCEPT_HEADER=xxx

# Optional
MAX_RETRIES_ATTEMPTS=100
SOCK5_URL=https://xxx/socks5.txt
SOCK4_URL=https://xxx/socks4.txt
HTTP_URL=https://xxx/http.txt

UNKNOWN_ERROR_WEBHOOK=xxx
REQWEST_ERROR_WEBHOOK=xxx
NO_PROXIES_AVAILABLE_ERROR_WEBHOOK=xxx
FAILED_TO_FETCH_AFTER_RETRIES_ERROR_WEBHOOK=xxx
UTILS_ERROR_WEBHOOK=xxx

HIANIME_DOMAINS=xxx
```

>[!NOTE]
>Required variables are `USER_AGENT_HEADER`, `ACCEPT_ENCODING_HEADER`, and `ACCEPT_HEADER`. All other variables are optional and can be left empty if not needed.

##  <span id="manual-configuration">Manual Configuration</span>

Alternatively, you can configure `aniscraper` manually using a struct. First, ensure you have `aniscraper` included in your project:

```rust
use aniscraper::env::SecretConfig;
```

Then, define and initialize the `SecretConfig` struct:

```rust
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
```

Then pass `SecretConfig` as an argument to the `new` method:

```rust
#[tokio::main]
async fn main() {
    let secret_config = SecretConfig {
        max_retries_attempts: "100".to_string(),
        reqwest_error_webhook: "".to_string(),
        no_proxies_available_error_webhook: "".to_string(),
        failed_to_fetch_after_retries_error_webhook: "".to_string(),
        utils_error_webhook: "".to_string(),
        unknown_error_webhook: "".to_string(),
        http_url: "".to_string(),
        sock4_url: "".to_string(),
        sock5_url: "".to_string(),
        hianime_domains: "".to_string(),
        user_agent_header: "your-user-agent".to_string(),
        accept_encoding_header: "your-accept-encoding".to_string(),
        accept_header: "your-accept-header".to_string(),
    };
    let hianime = HiAnimeRust::new(Some(secret_config)).await;
}
```

##  <span id="usage">Usage</span>

Once configured, you can use `aniscraper` in your project to start scraping with proxy support and error handling. Here's an example of how to use the library:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hianime = HiAnimeRust::new(None).await;
    
    let hianime_home_info = hianime.scrape_home().await?;
    let hianime_about_anime_info = hianime.scrape_about_anime("one-piece-100").await?;
    let hianime_atoz_list_info = hianime.scrape_atoz(2).await?;
    let hianime_episodes_info = hianime.scrape_episodes("one-piece-100").await?;
    let hianime_search_query_info = hianime.scrape_search("jojo", 1).await?;
    let hianime_category_info = hianime.scrape_category("ova", 3).await?;
    let hianime_episode_sources_info = hianime.scrape_servers("death-note-60?ep=1464").await?;
    let hianime_episode_streaming_links = hianime
                                              .scrape_episode_server_source(
                                                  "death-note-60?ep=1464",
                                                  aniscraper::servers::EpisodeType::Sub,
                                                  Some(AnimeServer::Streamtape)
                                              ).await?;
}
```

##  <span id="license">License</span>

`aniscraper` is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

##  <span id="contributing">Contributing</span>

Contributions to `aniscraper` are welcome! If you have suggestions, improvements, or bug fixes, please submit a pull request or open an issue on our [GitHub repository](https://github.com/your-username/aniscraper). Your feedback and contributions are greatly appreciated.

---
