use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{anime, anime_id, anime_staff, episodes, staff};

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = anime)]
pub struct Anime {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub mal_id: i32,
    pub al_id: i32,
    pub japanese_title: Option<String>,
    pub synonyms: Option<String>,
    pub image: String,
    pub category: String,
    pub rating: String,
    pub quality: String,
    pub duration: String,
    pub premiered: String,
    pub aired: String,
    pub status: String,
    pub mal_score: String,
    pub studios: String,
    pub producers: String,
    pub genres: String,
    pub sub_episodes: i32,
    pub dub_episodes: i32,
    pub total_episodes: i32,
    pub sub_or_dub: String,
}

#[derive(Queryable, Insertable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = anime_id)]
pub struct AnimeID {
    pub anime_name: String,
}

#[derive(Queryable, Insertable, Selectable, Debug, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = episodes)]
pub struct Episode {
    pub id: String,
    pub title: String,
    pub is_filler: bool,
    pub episode_no: i32,
    pub anime_id: i32,
}

#[derive(Queryable, Insertable, Selectable, Debug, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = staff)]
pub struct Staff {
    pub mal_id: i32,
    pub name: String,
    pub mal_url: String,
    pub image: String,
    pub positions: Vec<Option<String>>,
}

#[derive(Queryable, Insertable, Selectable, Debug, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = anime_staff)]
pub struct AnimeStaff {
    pub anime_id: i32,
    pub staff_id: i32,
    pub positions: Vec<Option<String>>,
}
