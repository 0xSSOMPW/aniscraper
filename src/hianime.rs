use lazy_static::lazy_static;
use scraper::{Html, Selector};

use crate::{
    env::EnvVar,
    error::AniRustError,
    model::MinimalAnime,
    proxy::{load_proxies, Proxy},
    utils::get_curl,
};

lazy_static! {
    static ref TRENDING_SELECTOR: Selector =
        Selector::parse("#anime-trending #trending-home .swiper-wrapper .swiper-slide").unwrap();
}

#[derive(Debug)]
pub struct HiAnimeRust {
    domains: Vec<String>,
    proxies: Vec<Proxy>,
}

impl HiAnimeRust {
    pub async fn new() -> Self {
        let domain_list = EnvVar::HIANIME_DOMAINS.get_config();

        let domains: Vec<String> = if domain_list.is_empty() {
            vec!["https://aniwatchtv.to".to_string()]
        } else {
            domain_list
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };

        let proxies = match load_proxies().await {
            Ok(p) => p,
            Err(_) => Vec::new(),
        };

        HiAnimeRust { domains, proxies }
    }

    pub async fn scrape_home(&self) -> Result<Vec<MinimalAnime>, AniRustError> {
        let url = format!("{}/home", self.domains[0]);
        let curl = get_curl(&url, &self.proxies).await?;
        let document = Html::parse_document(&curl);

        let mut res = vec![];

        for element in document.select(&TRENDING_SELECTOR) {
            let id = element
                .select(&Selector::parse(".item .film-poster").unwrap())
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|href| href.trim_start_matches('/'))
                .map(|s| s.to_string())
                .unwrap_or_default();

            let title = element
                .select(&Selector::parse(".item .number .film-title.dynamic-name").unwrap())
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let image = element
                .select(&Selector::parse(".item .film-poster .film-poster-img").unwrap())
                .next()
                .and_then(|e| e.value().attr("data-src"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            res.push(MinimalAnime { id, title, image });
        }

        Ok(res)
    }
}
