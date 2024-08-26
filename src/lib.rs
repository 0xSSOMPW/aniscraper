// src/lib.rs

mod env;
mod error;
mod hianime;
mod model;
mod proxy;
mod utils;

use crate::hianime::HiAnimeRust;

#[derive(Debug)]
pub struct AniRust {
    pub HiAnime: HiAnimeRust,
}

impl AniRust {
    pub async fn new() -> Self {
        AniRust {
            HiAnime: HiAnimeRust::new().await,
        }
    }
}