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
