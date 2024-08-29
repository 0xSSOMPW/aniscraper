use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimalAnime {
    pub id: String,
    pub title: String,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Top10Anime {
    pub id: String,
    pub title: String,
    pub subs: u32,
    pub dubs: u32,
    pub eps: u32,
    pub rank: u32,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeaturedAnime {
    pub top_airing_animes: Vec<MinimalAnime>,
    pub most_popular_animes: Vec<MinimalAnime>,
    pub most_favorite_animes: Vec<MinimalAnime>,
    pub latest_completed_animes: Vec<MinimalAnime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Top10PeriodRankedAnime {
    pub day: Vec<Top10Anime>,
    pub week: Vec<Top10Anime>,
    pub month: Vec<Top10Anime>,
}
