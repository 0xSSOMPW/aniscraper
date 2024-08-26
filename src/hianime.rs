use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    env::EnvVar,
    error::AniRustError,
    model::{Anime, MinimalAnime},
    proxy::{load_proxies, Proxy},
    utils::get_curl,
};

lazy_static! {
    static ref TRENDING_SELECTOR: Selector =
        Selector::parse("#anime-trending #trending-home .swiper-wrapper .swiper-slide").unwrap();
    static ref LATEST_EPISODES_SELECTOR: Selector = Selector::parse(
        "#main-content .block_area_home:nth-of-type(1) .tab-content .film_list-wrap .flw-item"
    )
    .unwrap();
}

#[derive(Debug)]
pub struct HiAnimeRust {
    domains: Vec<String>,
    proxies: Vec<Proxy>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HomeInfo {
    pub trending: Vec<MinimalAnime>,
    pub latest_episodes: Vec<Anime>,
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

    pub async fn scrape_home(&self) -> Result<HomeInfo, AniRustError> {
        let url = format!("{}/home", self.domains[0]);
        let curl = get_curl(&url, &self.proxies).await?;
        let document = Html::parse_document(&curl);

        let mut trending = vec![];

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

            trending.push(MinimalAnime { id, title, image });
        }

        let latest_episodes = extract_anime_data(&document, &LATEST_EPISODES_SELECTOR);

        Ok(HomeInfo {
            trending,
            latest_episodes,
        })
    }
}

fn extract_anime_data(document: &Html, selector: &Selector) -> Vec<Anime> {
    let mut res = vec![];

    for element in document.select(selector) {
        let id = element
            .select(&Selector::parse(".film-name .dynamic-name").unwrap())
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|s| s.trim_start_matches('/').to_string())
            .unwrap_or_default();

        let title = element
            .select(&Selector::parse(".film-name .dynamic-name").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let subs = element
            .select(&Selector::parse(".film-poster .tick-sub").unwrap())
            .next()
            .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
            .unwrap_or_default();

        let dubs = element
            .select(&Selector::parse(".film-poster .tick-dub").unwrap())
            .next()
            .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
            .unwrap_or_default();

        let eps = element
            .select(&Selector::parse(".film-poster .tick-eps").unwrap())
            .next()
            .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
            .unwrap_or_default();

        let duration = element
            .select(&Selector::parse(".fd-infor .fdi-duration").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let rating = element
            .select(&Selector::parse(".film-poster .tick-rate").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let image = element
            .select(&Selector::parse(".film-poster .film-poster-img").unwrap())
            .next()
            .and_then(|e| e.value().attr("data-src").map(|s| s.to_string()))
            .unwrap_or_default();

        res.push(Anime {
            id,
            title,
            subs,
            dubs,
            eps,
            duration,
            rating,
            image,
        });
    }
    res
}
