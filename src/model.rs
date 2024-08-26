use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimalAnime {
    pub id: String,
    pub title: String,
    pub image: String,
}
