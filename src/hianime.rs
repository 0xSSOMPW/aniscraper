use lazy_static::lazy_static;
use scraper::{selectable::Selectable, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    env::EnvVar,
    error::AniRustError,
    model::{
        Anime, FeaturedAnime, MinimalAnime, SpotlightAnime, Top10Anime, Top10PeriodRankedAnime,
    },
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
    static ref SPOTLIGHT_SELECTOR: Selector =
        Selector::parse("#slider .swiper-wrapper .swiper-slide").unwrap();
    static ref FEATURED_SELECTOR: Selector =
        Selector::parse("#anime-featured .row div:nth-of-type(1) .anif-block-ul ul li").unwrap();
    static ref TOP_10_SELECTOR: Selector =
        Selector::parse("#main-sidebar .block_area-realtime [id^=\"top-viewed-\"]").unwrap();
    static ref A_TO_Z_SELECTOR: Selector = Selector::parse("#main-wrapper div div.page-az-wrap section div.tab-content div div.film_list-wrap .flw-item").unwrap();
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
    pub spotlight_animes: Vec<SpotlightAnime>,
    pub featured: FeaturedAnime,
    pub top_10_animes: Top10PeriodRankedAnime,
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
        let spotlight_animes = extract_spotlight_anime_data(&document, &SPOTLIGHT_SELECTOR);
        let genres = extract_genres(&document, &GENRES_SELECTOR);
        let top_10_animes = extract_top_10(&document, &TOP_10_SELECTOR);

        let (top_airing_animes, most_popular_animes, most_favorite_animes, latest_completed_animes) =
            extract_featured_anime(&document, &FEATURED_SELECTOR);
        let featured = FeaturedAnime {
            top_airing_animes,
            most_popular_animes,
            most_favorite_animes,
            latest_completed_animes,
        };

        Ok(HomeInfo {
            trending,
            latest_episodes,
            top_upcoming_animes,
            spotlight_animes,
            featured,
            top_10_animes,
            genres,
        })
    }

    pub async fn scrape_atoz(&self, page_no: u32) -> Result<Vec<Anime>, AniRustError> {
        let mut error_vec = vec![];
        let mut curl = String::new();

        for domain in &self.domains {
            let url = format!("{}/az-list?page={}", domain, page_no);

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

        let animes = extract_anime_data(&document, &A_TO_Z_SELECTOR);

        Ok(animes)
    }
}

fn extract_anime_data(document: &Html, selector: &Selector) -> Vec<Anime> {
    document
        .select(selector)
        .map(|element| {
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

            Anime {
                id,
                title,
                subs,
                dubs,
                eps,
                duration,
                rating,
                image,
            }
        })
        .collect()
}

fn extract_spotlight_anime_data(document: &Html, selector: &Selector) -> Vec<SpotlightAnime> {
    document
        .select(selector)
        .map(|element| {
            let id = element
                .select(&Selector::parse(".deslide-item-content .desi-buttons a").unwrap())
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|s| s.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let title = element
                .select(
                    &Selector::parse(".deslide-item-content .desi-head-title.dynamic-name")
                        .unwrap(),
                )
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let rank = element
                .select(&Selector::parse(".deslide-item-content .desi-sub-text").unwrap())
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default()
                .split_whitespace()
                .next()
                .and_then(|s| s.trim_start_matches('#').parse::<u32>().ok())
                .unwrap_or_default();

            let image = element
                .select(
                    &Selector::parse(".deslide-cover .deslide-cover-img .film-poster-img").unwrap(),
                )
                .next()
                .and_then(|e| e.value().attr("data-src").map(|s| s.to_string()))
                .unwrap_or_default();

            let description = element
                .select(&Selector::parse(".deslide-item-content .desi-description").unwrap())
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let extra_info: Vec<String> = element
                .select(&Selector::parse(".deslide-item-content .sc-detail .scd-item").unwrap())
                .map(|e| e.text().collect::<String>().trim().to_string())
                .collect();

            let eps = extra_info
                .get(4)
                .and_then(|s| {
                    s.split_whitespace()
                        .map(|s| s.parse().ok())
                        .collect::<Vec<_>>()
                        .get(2)
                        .copied()
                })
                .flatten()
                .unwrap_or_default();

            let subs = extra_info
                .get(4)
                .and_then(|s| {
                    s.split_whitespace()
                        .map(|s| s.parse().ok())
                        .collect::<Vec<_>>()
                        .first()
                        .copied()
                })
                .flatten()
                .unwrap_or_default();

            let dubs = extra_info
                .get(4)
                .and_then(|s| {
                    s.split_whitespace()
                        .map(|s| s.parse().ok())
                        .collect::<Vec<_>>()
                        .get(1)
                        .copied()
                })
                .flatten()
                .unwrap_or_default();

            SpotlightAnime {
                id,
                title,
                rank,
                image,
                description,
                subs,
                dubs,
                eps,
                duration: extra_info.get(1).cloned().unwrap_or_default(),
                quality: extra_info.get(3).cloned().unwrap_or_default(),
                category: extra_info.get(0).cloned().unwrap_or_default(),
                released_day: extra_info.get(2).cloned().unwrap_or_default(),
            }
        })
        .collect()
}

fn extract_minimal_anime(document: &Html, selector: &Selector) -> Vec<MinimalAnime> {
    document
        .select(selector)
        .map(|element| {
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

            MinimalAnime { id, title, image }
        })
        .collect()
}

fn extract_featured_anime(
    document: &Html,
    selector: &Selector,
) -> (
    Vec<MinimalAnime>,
    Vec<MinimalAnime>,
    Vec<MinimalAnime>,
    Vec<MinimalAnime>,
) {
    let id_selector = Selector::parse(".film-detail .film-name .dynamic-name").unwrap();
    let image_selector = Selector::parse(".film-poster a .film-poster-img").unwrap();

    let res: Vec<MinimalAnime> = document
        .select(selector)
        .map(|element| {
            let id = element
                .select(&id_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|href| href.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let title = element
                .select(&id_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let image = element
                .select(&image_selector)
                .next()
                .and_then(|e| e.value().attr("data-src"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            MinimalAnime { id, title, image }
        })
        .collect();

    let top_airing_animes = res[0..5].to_vec();
    let most_popular_animes = res[5..10].to_vec();
    let most_favorite_animes = res[10..15].to_vec();
    let latest_completed_animes = res[15..20].to_vec();

    (
        top_airing_animes,
        most_popular_animes,
        most_favorite_animes,
        latest_completed_animes,
    )
}

fn extract_top_10(document: &Html, selector: &Selector) -> Top10PeriodRankedAnime {
    let (day, week, month) = document
        .select(selector)
        .filter_map(|element| element.value().attr("id"))
        .map(|id| id.split('-').last().unwrap_or("").trim().to_string())
        .fold(
            (vec![], vec![], vec![]),
            |(mut day, mut week, mut month), period_type| {
                match period_type.as_str() {
                    "week" => week.extend(extract_top_10_by_period_type(document, "week")),
                    "month" => month.extend(extract_top_10_by_period_type(document, "month")),
                    _ => day.extend(extract_top_10_by_period_type(document, "day")),
                }
                (day, week, month)
            },
        );

    Top10PeriodRankedAnime { day, week, month }
}

fn extract_top_10_by_period_type(document: &Html, period_type: &str) -> Vec<Top10Anime> {
    let selector_format = format!("#top-viewed-{} ul li", period_type);
    let selector = Selector::parse(&selector_format).unwrap();

    document
        .select(&selector)
        .map(|element| {
            let id = element
                .select(&Selector::parse(".film-detail .film-name .dynamic-name").unwrap())
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|s| s.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let title = element
                .select(&Selector::parse(".film-detail .film-name .dynamic-name").unwrap())
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let rank = element
                .select(&Selector::parse(".film-number span").unwrap())
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .and_then(|e| e.parse::<u32>().ok())
                .unwrap_or_default();

            let subs = element
                .select(&Selector::parse(".film-detail .fd-infor .tick-item.tick-sub").unwrap())
                .next()
                .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
                .unwrap_or_default();

            let dubs = element
                .select(&Selector::parse(".film-detail .fd-infor .tick-item.tick-dub").unwrap())
                .next()
                .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
                .unwrap_or_default();

            let eps = element
                .select(&Selector::parse(".film-detail .fd-infor .tick-item.tick-eps").unwrap())
                .next()
                .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
                .unwrap_or_default();

            let image = element
                .select(&Selector::parse(".film-poster .film-poster-img").unwrap())
                .next()
                .and_then(|e| e.value().attr("data-src").map(|s| s.to_string()))
                .unwrap_or_default();

            Top10Anime {
                id,
                title,
                image,
                subs,
                dubs,
                eps,
                rank,
            }
        })
        .collect()
}

fn extract_genres(document: &Html, selector: &Selector) -> Vec<String> {
    document
        .select(selector)
        .map(|element| {
            let text = element.text().collect::<String>().trim().to_string();
            if text.is_empty() {
                String::new()
            } else {
                text
            }
        })
        .collect()
}
