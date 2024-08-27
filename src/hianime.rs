use lazy_static::lazy_static;
use scraper::{selectable::Selectable, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    env::EnvVar,
    error::AniRustError,
    model::{Anime, MinimalAnime},
    proxy::{load_proxies, Proxy},
    utils::{get_curl, opt_box_error_vec_to_string},
};

lazy_static! {
    static ref TRENDING_SELECTOR: Selector =
        Selector::parse("#anime-trending #trending-home .swiper-wrapper .swiper-slide").unwrap();
    static ref LATEST_EPISODES_SELECTOR: Selector = Selector::parse(
        "#main-content .block_area_home:nth-of-type(1) .tab-content .film_list-wrap .flw-item"
    )
    .unwrap();
    static ref TOP_UPCOMING_SELECTOR: Selector = Selector::parse(
        "#main-content .block_area_home:nth-of-type(3) .tab-content .film_list-wrap .flw-item"
    )
    .unwrap();
    static ref GENRES_SELECTOR: Selector = Selector::parse(
        "#main-sidebar .block_area.block_area_sidebar.block_area-genres .sb-genre-list li"
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
    pub top_upcoming_animes: Vec<Anime>,
    pub genres: Vec<String>,
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
        let mut error_vec = vec![];
        let mut curl = String::new();

        for domain in &self.domains {
            let url = format!("{}/home", domain);

            match get_curl(&url, &self.proxies).await {
                Ok(curl_string) => {
                    curl = curl_string;
                    break;
                }
                Err(e) => {
                    error_vec.push(Some(e));
                }
            }
        }

        if curl.is_empty() {
            let error_string: String = opt_box_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);

        let trending = extract_minimal_anime(&document, &TRENDING_SELECTOR);
        let latest_episodes = extract_anime_data(&document, &LATEST_EPISODES_SELECTOR);
        let top_upcoming_animes = extract_anime_data(&document, &TOP_UPCOMING_SELECTOR);
        let genres = extract_genres(&document, &GENRES_SELECTOR);

        Ok(HomeInfo {
            trending,
            latest_episodes,
            top_upcoming_animes,
            genres,
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

fn extract_minimal_anime(document: &Html, selector: &Selector) -> Vec<MinimalAnime> {
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

    trending
}

fn extract_genres(document: &Html, selector: &Selector) -> Vec<String> {
    let mut genres = vec![];

    for element in document.select(selector) {
        let text = element.text().collect::<String>().trim().to_string();
        if text.is_empty() {
            genres.push(String::new())
        } else {
            genres.push(text);
        }
    }

    genres
}
