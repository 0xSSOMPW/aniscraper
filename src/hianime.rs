use lazy_static::lazy_static;
use regex::Regex;
use scraper::{selectable::Selectable, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    env::{self, EnvVar, SecretConfig},
    error::AniRustError,
    proxy::{load_proxies, Proxy},
    servers::{AnimeServer, EpisodeType, MegaCloudServer, ServerExtractedInfo, StreamTapeServer},
    utils::{anirust_error_vec_to_string, get_ajax_curl, get_curl},
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
    static ref NAVIGATION_SELECTOR: Selector = Selector::parse("div.pre-pagination.mt-5.mb-5 > nav > ul > li:last-child a").unwrap();
    static ref ABOUT_ANIME_SELECTOR: Selector = Selector::parse("#ani_detail .ani_detail-stage .container .anis-content").unwrap();
    static ref MOST_POPULAR_ANIME_SELECTOR: Selector = Selector::parse("#main-sidebar .block_area.block_area_sidebar.block_area-realtime:nth-of-type(2) .anif-block-ul ul li").unwrap();
    static ref RELATED_ANIME_SELECTOR: Selector = Selector::parse("#main-sidebar .block_area.block_area_sidebar.block_area-realtime:nth-of-type(1) .anif-block-ul ul li").unwrap();
    static ref RECOMMENDED_ANIME_SELECTOR: Selector = Selector::parse("#main-content .block_area.block_area_category .tab-content .flw-item").unwrap();
    static ref SEASONS_SELECTOR: Selector = Selector::parse(".os-list a.os-item").unwrap();
    static ref EPISODE_SELECTOR: Selector = Selector::parse(".detail-infor-content .ss-list a").unwrap();
    static ref CATEGORY_SELECTOR: Selector = Selector::parse("#main-content .tab-content .film_list-wrap .flw-item").unwrap();
    static ref SEARCH_SELECTOR: Selector = Selector::parse("#main-content .tab-content .film_list-wrap .flw-item").unwrap();
    static ref EPISODE_NO_SELECTOR: Selector = Selector::parse(".server-notice strong").unwrap();
    static ref EPISODE_SUB_SELECTOR: Selector = Selector::parse(".ps_-block.ps_-block-sub.servers-sub .ps__-list .server-item").unwrap();
    static ref EPISODE_DUB_SELECTOR: Selector = Selector::parse(".ps_-block.ps_-block-sub.servers-dub .ps__-list .server-item").unwrap();
    static ref EPISODE_RAW_SELECTOR: Selector = Selector::parse(".ps_-block.ps_-block-sub.servers-raw .ps__-list .server-item").unwrap();
}

#[derive(Debug)]
pub struct HiAnimeRust {
    domains: Vec<String>,
    proxies: Vec<Proxy>,
    secret: Option<SecretConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HomeInfo {
    pub trending: Vec<MinimalAnime>,
    pub latest_episodes: Vec<Anime>,
    pub top_upcoming_animes: Vec<Anime>,
    pub spotlight_animes: Vec<SpotlightAnime>,
    pub featured: FeaturedAnime,
    pub top_10_animes: Top10PeriodRankedAnime,
    pub genres: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryInfo {
    pub total_pages: u32,
    pub current_page: u32,
    pub has_next_page: bool,
    pub animes: Vec<Anime>,
    pub top_10_animes: Top10PeriodRankedAnime,
    pub genres: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchInfo {
    pub total_pages: u32,
    pub current_page: u32,
    pub has_next_page: bool,
    pub animes: Vec<Anime>,
    pub most_popular_animes: Vec<SideBarAnimes>,
    pub genres: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpisodesInfo {
    pub total_episodes: u32,
    pub episodes: Vec<AnimeEpisode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtoZ {
    pub has_next_page: bool,
    pub current_page: u32,
    pub total_pages: u32,
    pub animes: Vec<Anime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinimalAnime {
    pub id: String,
    pub title: String,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Anime {
    pub id: String,
    pub title: String,
    pub subs: u32,
    pub dubs: u32,
    pub eps: u32,
    pub duration: String,
    pub rating: String,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpotlightAnime {
    pub id: String,
    pub title: String,
    pub subs: u32,
    pub dubs: u32,
    pub eps: u32,
    pub duration: String,
    pub rank: u32,
    pub image: String,
    pub description: String,
    pub category: String,
    pub released_day: String,
    pub quality: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Top10Anime {
    pub id: String,
    pub title: String,
    pub subs: u32,
    pub dubs: u32,
    pub eps: u32,
    pub rank: u32,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SideBarAnimes {
    pub id: String,
    pub title: String,
    pub subs: u32,
    pub dubs: u32,
    pub eps: u32,
    pub category: String,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeSeason {
    pub id: String,
    pub title: String,
    pub anime_title: String,
    pub image: String,
    pub is_current: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeEpisode {
    pub id: String,
    pub episode_no: u32,
    pub title: String,
    pub is_filler: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeaturedAnime {
    pub top_airing_animes: Vec<MinimalAnime>,
    pub most_popular_animes: Vec<MinimalAnime>,
    pub most_favorite_animes: Vec<MinimalAnime>,
    pub latest_completed_animes: Vec<MinimalAnime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Top10PeriodRankedAnime {
    pub day: Vec<Top10Anime>,
    pub week: Vec<Top10Anime>,
    pub month: Vec<Top10Anime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AboutAnime {
    pub id: String,
    pub mal_id: u32,
    pub al_id: u32,
    pub anime_id: u32,
    pub title: String,
    pub description: String,
    pub image: String,
    pub rating: String,
    pub category: String,
    pub duration: String,
    pub quality: String,
    pub subs: u32,
    pub dubs: u32,
    pub eps: u32,
    pub japanese: String,
    pub synonyms: String,
    pub aired: String,
    pub premiered: String,
    pub status: String,
    pub mal_score: String,
    pub studios: Vec<String>,
    pub producers: Vec<String>,
    pub genres: Vec<String>,
    pub most_popular_animes: Vec<SideBarAnimes>,
    pub related_animes: Vec<SideBarAnimes>,
    pub recommended_animes: Vec<Anime>,
    pub seasons: Vec<AnimeSeason>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    pub server_name: String,
    pub server_id: u32,
    pub data_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub episode_no: u32,
    pub sub: Vec<Server>,
    pub dub: Vec<Server>,
    pub raw: Vec<Server>,
}

trait HasClass {
    fn has_class(&self, class_name: &str) -> bool;
}

impl HasClass for scraper::ElementRef<'_> {
    fn has_class(&self, class_name: &str) -> bool {
        self.value()
            .attr("class")
            .map(|class| class.split_whitespace().any(|c| c == class_name))
            .unwrap_or(false)
    }
}

impl HiAnimeRust {
    pub async fn new(secret: Option<SecretConfig>) -> Self {
        let secret_clone = initialize_secret(secret);
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
            Err(e) => {
                eprintln!("Failed to load proxies: {:?}", e);
                Vec::new()
            }
        };

        HiAnimeRust {
            domains,
            proxies,
            secret: secret_clone,
        }
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
            let error_string: String = anirust_error_vec_to_string(error_vec);
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

    pub async fn scrape_atoz(&self, page_no: u32) -> Result<AtoZ, AniRustError> {
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
            let error_string: String = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);

        let animes = extract_anime_data(&document, &A_TO_Z_SELECTOR);

        let total_pages = get_last_page_no(&document);
        let current_page = page_no;
        let has_next_page = page_no != total_pages;

        Ok(AtoZ {
            has_next_page,
            current_page,
            total_pages,
            animes,
        })
    }

    pub async fn scrape_about_anime(&self, id: &str) -> Result<AboutAnime, AniRustError> {
        let mut error_vec = vec![];
        let mut curl = String::new();

        for domain in &self.domains {
            let url = format!("{}/{}", domain, id);

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
            let error_string: String = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);
        let about = extract_anime_about_info(&document, &ABOUT_ANIME_SELECTOR);
        Ok(about)
    }

    pub async fn scrape_category(
        &self,
        category: &str,
        page_no: u32,
    ) -> Result<CategoryInfo, AniRustError> {
        let mut error_vec = vec![];
        let mut curl = String::new();

        for domain in &self.domains {
            let url = format!("{}/{}?page={}", domain, category, page_no);

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
            let error_string: String = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);
        let animes = extract_anime_data(&document, &CATEGORY_SELECTOR);
        let top_10_animes = extract_top_10(&document, &TOP_10_SELECTOR);
        let genres = extract_genres(&document, &GENRES_SELECTOR);
        let total_pages = get_last_page_no(&document);
        let current_page = page_no;
        let has_next_page = page_no != total_pages;

        Ok(CategoryInfo {
            total_pages,
            current_page,
            has_next_page,
            animes,
            top_10_animes,
            genres,
        })
    }

    pub async fn scrape_search(
        &self,
        query: &str,
        page_no: u32,
    ) -> Result<SearchInfo, AniRustError> {
        let mut error_vec = vec![];
        let mut curl = String::new();

        for domain in &self.domains {
            let url = format!("{}/search?keyword={}&page={}", domain, query, page_no);

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
            let error_string: String = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);
        let most_popular_selector = Selector::parse(
            "#main-sidebar .block_area.block_area_sidebar.block_area-realtime .anif-block-ul ul li",
        )
        .unwrap();

        let animes = extract_anime_data(&document, &SEARCH_SELECTOR);
        let most_popular_animes = extract_side_bar_animes(&document, &most_popular_selector);
        let total_pages = get_last_page_no(&document);
        let current_page = page_no;
        let has_next_page = page_no != total_pages;
        let genres = extract_genres(&document, &GENRES_SELECTOR);

        Ok(SearchInfo {
            total_pages,
            current_page,
            has_next_page,
            animes,
            most_popular_animes,
            genres,
        })
    }

    pub async fn scrape_episodes(&self, id: &str) -> Result<EpisodesInfo, AniRustError> {
        let mut error_vec = vec![];
        let mut curl = String::new();
        let anime_id = id.split('-').last().unwrap();

        for domain in &self.domains {
            let url = format!("{}/ajax/v2/episode/list/{}", domain, anime_id);

            match get_ajax_curl(&url, "html").await {
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
            let error_string: String = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);

        let episodes = extract_anime_episode(&document, &EPISODE_SELECTOR);
        let total_episodes = episodes.len() as u32;

        Ok(EpisodesInfo {
            total_episodes,
            episodes,
        })
    }

    pub async fn scrape_servers(&self, id: &str) -> Result<ServerInfo, AniRustError> {
        let mut error_vec = vec![];
        let mut curl = String::new();
        let episode_id = id.split("ep=").last().unwrap_or_default();

        for domain in &self.domains {
            let url = format!(
                "{}/ajax/v2/episode/servers?episodeId={}",
                domain, episode_id
            );

            match get_ajax_curl(&url, "html").await {
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
            let error_string: String = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);

        let episode_str = document
            .select(&EPISODE_NO_SELECTOR)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let last_part = episode_str.split_whitespace().last().unwrap_or_default();

        let episode_no = last_part.parse::<u32>().unwrap_or_default();
        let sub = extract_episode_servers(&document, &EPISODE_SUB_SELECTOR);
        let dub = extract_episode_servers(&document, &EPISODE_DUB_SELECTOR);
        let raw = extract_episode_servers(&document, &EPISODE_RAW_SELECTOR);

        Ok(ServerInfo {
            episode_no,
            sub,
            dub,
            raw,
        })
    }

    pub async fn scrape_episode_server_source(
        &self,
        id: &str,
        episode_type: EpisodeType,
        anime_server: Option<AnimeServer>,
    ) -> Result<ServerExtractedInfo, AniRustError> {
        let server_list = self.scrape_servers(id).await?;
        let mut error_vec = vec![];
        let mut link = String::new();

        let mut server_id: u32 = 0;
        let mut data_id: u32 = 0;

        match episode_type {
            EpisodeType::Dub => {
                update_server_id(&mut server_id, &mut data_id, server_list.dub, anime_server)
            }
            EpisodeType::Raw => {
                update_server_id(&mut server_id, &mut data_id, server_list.raw, anime_server)
            }
            _ => update_server_id(&mut server_id, &mut data_id, server_list.sub, anime_server),
        }

        for domain in &self.domains {
            let url = format!("{}/ajax/v2/episode/sources?id={}", domain, data_id);
            println!("{:?}", url);

            match get_ajax_curl(&url, "link").await {
                Ok(curl_string) => {
                    link = curl_string;
                    break;
                }
                Err(e) => {
                    error_vec.push(Some(e));
                }
            }
        }

        if link.is_empty() {
            let error_string: String = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let server_info = match server_id {
            3 => StreamTapeServer::extract(&link, &self.proxies).await?,
            4 => MegaCloudServer::extract(&link, &self.proxies).await?,
            5 => MegaCloudServer::extract(&link, &self.proxies).await?,
            1 => MegaCloudServer::extract(&link, &self.proxies).await?,
            _ => MegaCloudServer::extract(&link, &self.proxies).await?,
        };

        Ok(server_info)
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
                .unwrap_or_default()
                .split("/")
                .last()
                .unwrap_or_default()
                .to_string();

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
                category: extra_info.first().cloned().unwrap_or_default(),
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

fn extract_anime_about_info(document: &Html, selector: &Selector) -> AboutAnime {
    let play_button_selector = Selector::parse(".anisc-detail .film-buttons a.btn-play").unwrap();
    let name_selector = Selector::parse(".anisc-detail .film-name.dynamic-name").unwrap();
    let rating_selector = Selector::parse(".film-stats .tick .tick-pg").unwrap();
    let quality_selector = Selector::parse(".film-stats .tick .tick-quality").unwrap();
    let subs_selector = Selector::parse(".film-stats .tick .tick-sub").unwrap();
    let dubs_selector = Selector::parse(".film-stats .tick .tick-dub").unwrap();
    let eps_selector = Selector::parse(".film-stats .tick .tick-eps").unwrap();
    let image_selector = Selector::parse(".anisc-poster .film-poster .film-poster-img").unwrap();
    let description_selector = Selector::parse(".anisc-detail .film-description .text").unwrap();
    let tick_selector = Selector::parse(".film-stats .tick").unwrap();
    let json_script_selector = Selector::parse("#syncData").unwrap();
    let more_info_selector = Selector::parse(
        "#ani_detail .ani_detail-stage .container .anis-content .anisc-info .item-title",
    )
    .unwrap();
    let genres_selector = Selector::parse(
        "#ani_detail .ani_detail-stage .container .anis-content .anisc-info .item-list",
    )
    .unwrap();

    let mut about_anime = AboutAnime {
        id: String::new(),
        mal_id: 0,
        anime_id: 0,
        al_id: 0,
        title: String::new(),
        description: String::new(),
        image: String::new(),
        category: String::new(),
        rating: String::new(),
        quality: String::new(),
        duration: String::new(),
        subs: 0,
        dubs: 0,
        eps: 0,
        japanese: String::new(),
        synonyms: String::new(),
        aired: String::new(),
        premiered: String::new(),
        status: String::new(),
        mal_score: String::new(),
        studios: vec![],
        producers: vec![],
        genres: vec![],
        most_popular_animes: vec![],
        related_animes: vec![],
        recommended_animes: vec![],
        seasons: vec![],
    };

    document.select(selector).for_each(|element| {
        about_anime.id = element
            .select(&play_button_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|s| s.split('/').last().unwrap_or("").to_string())
            .unwrap_or_default();

        about_anime.title = element
            .select(&name_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        about_anime.rating = element
            .select(&rating_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        about_anime.quality = element
            .select(&quality_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        about_anime.subs = element
            .select(&subs_selector)
            .next()
            .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
            .unwrap_or_default();

        about_anime.dubs = element
            .select(&dubs_selector)
            .next()
            .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
            .unwrap_or_default();

        about_anime.eps = element
            .select(&eps_selector)
            .next()
            .and_then(|e| e.text().collect::<String>().parse::<u32>().ok())
            .unwrap_or_default();

        about_anime.image = element
            .select(&image_selector)
            .next()
            .and_then(|e| e.value().attr("src").map(|s| s.to_string()))
            .unwrap_or_default();

        about_anime.description = element
            .select(&description_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if let Some(tick) = element.select(&tick_selector).next() {
            let text = tick
                .text()
                .collect::<String>()
                .replace('\n', " ")
                .trim()
                .to_string();

            let mut parts = text.split_whitespace().rev();
            about_anime.category = parts.nth(1).unwrap_or("").to_string();
            about_anime.duration = parts.next().unwrap_or("").to_string();
        }

        let json_text = document
            .select(&json_script_selector)
            .next()
            .map(|script| script.text().collect::<String>())
            .unwrap_or_default();

        if let Ok(json) = serde_json::from_str::<Value>(&json_text) {
            about_anime.anime_id = json
                .get("anime_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .parse::<u32>()
                .unwrap_or_default();

            about_anime.mal_id = json
                .get("mal_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .parse::<u32>()
                .unwrap_or_default();

            about_anime.al_id = json
                .get("anilist_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .parse::<u32>()
                .unwrap_or_default();
        }
    });

    document.select(&more_info_selector).for_each(|element| {
        let head = element
            .select(&Selector::parse(".item-head").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let key = element
            .select(&Selector::parse(".name").unwrap())
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        match head.as_str() {
            "Japanese:" => about_anime.japanese = key,
            "Synonyms:" => about_anime.synonyms = key,
            "Aired:" => about_anime.aired = key,
            "Premiered:" => about_anime.premiered = key,
            "Status:" => about_anime.status = key,
            "MAL Score:" => about_anime.mal_score = key,
            "Producers:" => {
                about_anime.producers.extend(
                    element
                        .select(&Selector::parse("a.name").unwrap())
                        .map(|e| e.text().collect::<String>().trim().to_string()),
                );
            }
            "Studios:" => {
                about_anime.studios.extend(
                    element
                        .select(&Selector::parse("a.name").unwrap())
                        .map(|e| e.text().collect::<String>().trim().to_string()),
                );
            }
            _ => {}
        }
    });

    document.select(&genres_selector).for_each(|element| {
        about_anime.genres.extend(
            element
                .select(&Selector::parse("a").unwrap())
                .map(|e| e.text().collect::<String>().trim().to_string()),
        );
    });

    about_anime
        .most_popular_animes
        .extend(extract_side_bar_animes(
            document,
            &MOST_POPULAR_ANIME_SELECTOR,
        ));
    about_anime
        .related_animes
        .extend(extract_side_bar_animes(document, &RELATED_ANIME_SELECTOR));
    about_anime
        .recommended_animes
        .extend(extract_anime_data(document, &RECOMMENDED_ANIME_SELECTOR));
    about_anime
        .seasons
        .extend(extract_anime_seasons(document, &SEASONS_SELECTOR));

    about_anime
}

fn extract_side_bar_animes(document: &Html, selector: &Selector) -> Vec<SideBarAnimes> {
    let dynamic_name_selector = Selector::parse(".film-detail .dynamic-name").unwrap();
    let tick_selector = Selector::parse(".fd-infor .tick").unwrap();
    let tick_item_sub_selector = Selector::parse(".fd-infor .tick .tick-item.tick-sub").unwrap();
    let tick_item_dub_selector = Selector::parse(".fd-infor .tick .tick-item.tick-dub").unwrap();
    let tick_item_eps_selector = Selector::parse(".fd-infor .tick .tick-item.tick-eps").unwrap();
    let film_poster_selector = Selector::parse(".film-poster .film-poster-img").unwrap();

    document
        .select(selector)
        .map(|element| {
            let id = element
                .select(&dynamic_name_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(|s| s.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let title = element
                .select(&dynamic_name_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let image = element
                .select(&film_poster_selector)
                .next()
                .and_then(|e| e.value().attr("data-src").map(|s| s.to_string()))
                .unwrap_or_default();

            let subs = element
                .select(&tick_item_sub_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().trim().parse::<u32>().ok())
                .unwrap_or_default();

            let dubs = element
                .select(&tick_item_dub_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().trim().parse::<u32>().ok())
                .unwrap_or_default();

            let eps = element
                .select(&tick_item_eps_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().trim().parse::<u32>().ok())
                .unwrap_or_default();

            let category = element
                .select(&tick_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .map(|s| s.replace('\n', " ").replace("  ", " ").trim().to_string())
                .map(|s| s.split_whitespace().last().unwrap_or_default().to_string())
                .unwrap_or_default();

            SideBarAnimes {
                id,
                title,
                image,
                subs,
                dubs,
                eps,
                category,
            }
        })
        .collect()
}

fn extract_anime_seasons(document: &Html, selector: &Selector) -> Vec<AnimeSeason> {
    document
        .select(selector)
        .map(|element| {
            let id = element
                .value()
                .attr("href")
                .map(|s| s.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let title = element
                .value()
                .attr("title")
                .map(|e| e.trim().to_string())
                .unwrap_or_default();

            let anime_title = element
                .select(&Selector::parse(".title").unwrap())
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let mut image = element
                .select(&Selector::parse(".season-poster").unwrap())
                .next()
                .and_then(|e| e.value().attr("style").map(|s| s.to_string()))
                .unwrap_or_default();

            let re = Regex::new(r"url\((?P<url>.*?)\)").unwrap();

            image = re
                .captures(&image)
                .and_then(|caps| caps.name("url"))
                .map(|m| m.as_str().trim_matches('"').to_string())
                .unwrap_or_default();

            let is_current = element.has_class("active");

            AnimeSeason {
                id,
                title,
                anime_title,
                image,
                is_current,
            }
        })
        .collect()
}

fn extract_anime_episode(document: &Html, selector: &Selector) -> Vec<AnimeEpisode> {
    document
        .select(selector)
        .filter_map(|element| {
            let id = element
                .value()
                .attr("href")
                .map(|s| s.trim_start_matches('/').to_string())
                .unwrap_or_default()
                .split('/')
                .last()
                .unwrap_or_default()
                .to_string();

            if id.is_empty() {
                return None;
            }

            let title = element
                .value()
                .attr("title")
                .map(|e| e.trim().to_string())
                .unwrap_or_default();

            let episode_no = element
                .value()
                .attr("data-number")
                .and_then(|e| e.trim().parse::<u32>().ok())
                .unwrap_or(0);

            let is_filler = element.has_class("ssl-item-filler");

            Some(AnimeEpisode {
                id,
                title,
                episode_no,
                is_filler,
            })
        })
        .collect()
}

fn extract_episode_servers(document: &Html, selector: &Selector) -> Vec<Server> {
    document
        .select(selector)
        .map(|element| {
            let server_name = element
                .select(&Selector::parse("a").unwrap())
                .next()
                .map(|e| {
                    e.text()
                        .collect::<String>()
                        .to_lowercase()
                        .trim()
                        .to_string()
                })
                .unwrap_or_default();

            let data_id = element
                .attr("data-id")
                .and_then(|id| id.trim().parse::<u32>().ok())
                .unwrap_or_default();

            let server_id = element
                .attr("data-server-id")
                .and_then(|id| id.trim().parse::<u32>().ok())
                .unwrap_or_default();

            Server {
                server_name,
                server_id,
                data_id,
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

// Function to extract the last page number from the response
fn get_last_page_no(document: &Html) -> u32 {
    document
        .select(&NAVIGATION_SELECTOR)
        .last()
        .and_then(|element| element.value().attr("href"))
        .and_then(|href| href.split('=').last())
        .and_then(|page_str| page_str.parse::<u32>().ok())
        .unwrap_or(1)
}

fn initialize_secret(secret: Option<SecretConfig>) -> Option<SecretConfig> {
    let mut secret_lock = env::SECRET.lock().unwrap();
    secret_lock.clone_from(&secret);
    let secret_clone = secret_lock.clone();
    // Release the lock.
    drop(secret_lock);
    secret_clone
}

fn update_server_id(
    server_id: &mut u32,
    data_id: &mut u32,
    servers: Vec<Server>,
    anime_server: Option<AnimeServer>,
) {
    let anime_server = anime_server.unwrap_or(AnimeServer::Vidstreaming);

    for server in servers {
        println!("{} - {}", server.server_name, anime_server.as_str());
        if server.server_name == anime_server.as_str() {
            *server_id = server.server_id;
            *data_id = server.data_id;
            return;
        }
    }
}
