use regex::Regex;
use serde_json::Value;

use crate::{error::AniRustError, proxy::Proxy, utils::get_curl};
use std::{
    env::vars,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Track {
    pub file: String,
    pub kind: String,
    pub label: Option<String>,
    pub default: Option<bool>,
}

pub struct IntroOutro {
    pub start: u32,
    pub end: u32,
}

pub struct UnencryptedSrc {
    pub file: String,
    pub src_type: String,
}

pub struct ExtractedSrc {
    pub sources: Vec<UnencryptedSrc>,
    pub tracks: Vec<Track>,
    pub encrypted: bool,
    pub intro: IntroOutro,
    pub outro: IntroOutro,
    pub server: u32,
}

pub struct ExtractedData {
    pub intro: IntroOutro,
    pub outro: IntroOutro,
    pub tracks: Vec<Track>,
    pub sources: Vec<Source>,
}

pub struct Source {
    pub url: String,
    pub src_type: String,
}

// Implement the MegaCloud struct
pub struct MegaCloud {
    pub script: &'static str,
    pub sources: &'static str,
}

pub const MEGACLOUD: MegaCloud = MegaCloud {
    script: "https://megacloud.tv/js/player/a/prod/e1-player.min.js?v=",
    sources: "https://megacloud.tv/embed-2/ajax/e-1/getSources?id=",
};

#[derive(Debug, PartialEq, Eq)]
pub enum AnimeServer {
    Vidstreaming,
    Megacloud,
    Streamsb,
    Streamtape,
    Vidcloud,
}

impl AnimeServer {
    pub fn from_str(s: &str) -> Self {
        match s {
            "vidsrc" => AnimeServer::Vidstreaming,
            "megacloud" => AnimeServer::Megacloud,
            "streamsb" => AnimeServer::Streamsb,
            "streamtape" => AnimeServer::Streamtape,
            "vidcloud" => AnimeServer::Vidcloud,
            _ => AnimeServer::Vidstreaming,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AnimeServer::Vidstreaming => "vidsrc",
            AnimeServer::Megacloud => "megacloud",
            AnimeServer::Streamsb => "streamsb",
            AnimeServer::Streamtape => "streamtape",
            AnimeServer::Vidcloud => "vidcloud",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EpisodeType {
    Sub,
    Dub,
}

impl EpisodeType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "sub" => EpisodeType::Sub,
            "dub" => EpisodeType::Dub,
            _ => EpisodeType::Sub,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EpisodeType::Sub => "sub",
            EpisodeType::Dub => "dub",
        }
    }
}

pub struct MegaCloudServer;
