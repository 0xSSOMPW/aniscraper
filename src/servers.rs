use regex::Regex;
use serde_json::Value;

use crate::{error::AniRustError, proxy::Proxy, utils::get_curl};
use std::time::{SystemTime, UNIX_EPOCH};

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

impl MegaCloudServer {
    pub async fn extract(video_url: &str, proxies: &[Proxy]) -> Result<(), AniRustError> {
        let mut encrypted_string = String::new();
        let video_id = video_url
            .split('/')
            .last()
            .and_then(|s| s.split('?').next())
            .unwrap_or_default();

        let url = format!("{}{}", MEGACLOUD.sources, video_id);

        let curl = get_curl(&url, proxies).await?;
        // // Parse the string as JSON
        let json_value = serde_json::from_str::<Value>(&curl).unwrap_or_default();

        if let Some(data) = json_value.get("sources") {
            encrypted_string =
                serde_json::from_str::<String>(data.to_string().as_str()).unwrap_or_default();
        }

        // Get the current time in milliseconds since UNIX_EPOCH
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        // Format the URL by concatenating the base URL with the current timestamp
        let base_url = "https://megacloud.tv/js/player/a/prod/e1-player.min.js?v=";
        let full_url = format!("{}{}", base_url, now);

        let curl = get_curl(&full_url, proxies).await?;
        let variables = extract_variables(&curl)?;
        if variables.is_empty() {
            return Err(AniRustError::UnknownError(
                "Can't find variables. Perhaps the extractor is outdated.".to_string(),
            ));
        }
        println!("{:?}", get_secret(&encrypted_string, &variables));

        Ok(())
    }
}

fn extract_variables(text: &str) -> Result<Vec<(u32, u32)>, AniRustError> {
    let regex = Regex::new(r"case\s*0x[0-9a-f]+:\s*\w+\s*=\s*(\w+)\s*,\s*\w+\s*=\s*(\w+);")?;

    let vars: Vec<(u32, u32)> = regex
        .captures_iter(text)
        .filter_map(|cap| {
            if cap[0].contains("partKey") {
                return None;
            }

            let match_key1 = matching_key(&cap[1], text).ok()?;
            let match_key2 = matching_key(&cap[2], text).ok()?;

            match (
                u32::from_str_radix(&match_key1, 16),
                u32::from_str_radix(&match_key2, 16),
            ) {
                (Ok(key1), Ok(key2)) => Some((key1, key2)),
                _ => None,
            }
        })
        .collect();

    Ok(vars)
}

fn matching_key(value: &str, script: &str) -> Result<String, AniRustError> {
    let regex = Regex::new(&format!(r",{}=(((?:0x)?[0-9a-fA-F]+))", value))?;
    if let Some(captures) = regex.captures(script) {
        let match_str = captures
            .get(1)
            .ok_or_else(|| AniRustError::UnknownError("Failed to capture key".to_string()))?
            .as_str();
        Ok(match_str.trim_start_matches("0x").to_string())
    } else {
        Err(AniRustError::UnknownError(
            "Failed to match the key".to_string(),
        ))
    }
}

fn get_secret(encrypted_string: &str, values: &Vec<(u32, u32)>) -> (String, String) {
    let mut secret = String::new();
    let mut encrypted_source_array: Vec<char> = encrypted_string.chars().collect();
    let mut current_index: usize = 0;

    for &(start_offset, length) in values {
        let start = start_offset as usize + current_index;
        let end = start + length as usize;
        for i in start..end {
            if let Some(ch) = encrypted_string.chars().nth(i) {
                secret.push(ch);
                encrypted_source_array[i] = '\0';
            }
        }
        current_index += length as usize;
    }

    let encrypted_source: String = encrypted_source_array
        .into_iter()
        .filter(|&c| c != '\0')
        .collect();

    (secret, encrypted_source)
}
