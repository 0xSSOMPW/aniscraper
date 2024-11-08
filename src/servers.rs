use regex::Regex;
use scraper::Html;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::AniRustError,
    proxy::Proxy,
    utils::{anirust_error_vec_to_string, bytes_to_hex, decrypt_aes_256_cbc, get_curl},
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub file: String,
    pub kind: String,
    pub label: Option<String>,
    pub default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntroOutro {
    pub start: u32,
    pub end: u32,
}

impl Default for IntroOutro {
    fn default() -> Self {
        IntroOutro { start: 0, end: 0 }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MegaCloudUnencryptedSrc {
    pub file: String,
    pub src_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MegaCloudExtractedData {
    pub intro: IntroOutro,
    pub outro: IntroOutro,
    pub tracks: Vec<Track>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamTapeExtractedData {
    pub url: String,
    pub is_m3u8: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    #[serde(rename = "file")]
    pub url: String,
    #[serde(rename = "type")]
    pub src_type: String,
}

struct MegaCloud {
    pub script: &'static str,
    pub sources: &'static str,
}

const MEGACLOUD: MegaCloud = MegaCloud {
    script: "https://megacloud.tv/js/player/a/prod/e1-player.min.js?v=",
    sources: "https://megacloud.tv/embed-2/ajax/e-1/getSources?id=",
};

struct StreamSb {
    pub host1: &'static str,
    pub host2: &'static str,
}

const STREAMSB: StreamSb = StreamSb {
    host1: "https://watchsb.com/sources50",
    host2: "https://streamsss.net/sources16",
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
    Raw,
}

impl EpisodeType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "sub" => EpisodeType::Sub,
            "dub" => EpisodeType::Dub,
            "raw" => EpisodeType::Raw,
            _ => EpisodeType::Sub,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EpisodeType::Sub => "sub",
            EpisodeType::Dub => "dub",
            EpisodeType::Raw => "raw",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerExtractedInfo {
    MegaCloud(MegaCloudExtractedData),
    StreamTape(StreamTapeExtractedData),
}

pub struct MegaCloudServer;

impl MegaCloudServer {
    pub async fn extract(
        video_url: &str,
        proxies: &[Proxy],
    ) -> Result<ServerExtractedInfo, AniRustError> {
        let video_id = extract_video_id(video_url);
        let url = format!("{}{}", MEGACLOUD.sources, video_id);
        let json_data = fetch_initial_data(&url, proxies).await?;

        let is_encrypted = json_data["encrypted"].as_bool().unwrap_or(false);
        let intro: IntroOutro = parse_json_field(&json_data, "intro").unwrap_or_default();
        let outro: IntroOutro = parse_json_field(&json_data, "outro").unwrap_or_default();
        let tracks: Vec<Track> = parse_json_field(&json_data, "tracks")?;

        let sources = if is_encrypted {
            let encrypted_string = extract_encrypted_string(&json_data);
            let decrypted_sources = decrypt_sources(&encrypted_string, proxies).await?;
            parse_sources(&decrypted_sources)?
        } else {
            parse_json_field(&json_data, "sources")?
        };

        Ok(ServerExtractedInfo::MegaCloud(MegaCloudExtractedData {
            intro,
            outro,
            tracks,
            sources,
        }))
    }
}

pub struct StreamTapeServer;

impl StreamTapeServer {
    pub async fn extract(
        video_url: &str,
        proxies: &[Proxy],
    ) -> Result<ServerExtractedInfo, AniRustError> {
        let mut error_vec = vec![];
        let mut curl = String::new();

        match get_curl(video_url, proxies).await {
            Ok(curl_string) => {
                curl = curl_string;
            }
            Err(e) => {
                error_vec.push(Some(e));
            }
        }

        if curl.is_empty() {
            let error_string = anirust_error_vec_to_string(error_vec);
            return Err(AniRustError::UnknownError(error_string));
        }

        let document = Html::parse_document(&curl);

        let re = Regex::new(r"robotlink'\).innerHTML = (.*)'").unwrap();
        let html = document.root_element().html();

        if let Some(captures) = re.captures(&html) {
            if let Some(matched) = captures.get(1) {
                let parts: Vec<&str> = matched.as_str().split("+ ('").collect();
                if parts.len() == 2 {
                    let fh = parts[0].replace('\'', "");
                    let mut sh = parts[1].to_string();
                    sh = sh[3..].to_string();

                    let url = format!("https:{}{}", fh, sh);
                    return Ok(ServerExtractedInfo::StreamTape(StreamTapeExtractedData {
                        url: url.clone(),
                        is_m3u8: url.contains(".m3u8"),
                    }));
                }
            }
        }

        Err(AniRustError::FailedToFetchAfterRetries)
    }
}

// BUG: this server has been shut down
// pub struct StreamSBServer;
//
// impl StreamSBServer {
//     pub async fn extract(
//         video_url: &str,
//         is_alt: Option<bool>,
//         proxies: &[Proxy],
//     ) -> Result<(), AniRustError> {
//         let encoded_id = get_encoded_video_id(video_url);
//         let hexed_id = bytes_to_hex(&encoded_id);
//         let res = process_streamsb_url(is_alt, &hexed_id, proxies).await?;
//         Ok(())
//     }
// }

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

fn decrypt(
    encrypted: &str,
    key_or_secret: &str,
    maybe_iv: Option<Vec<u8>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let (key, nonce, contents) = if let Some(iv) = maybe_iv {
        (
            key_or_secret.as_bytes().to_vec(),
            iv,
            base64::decode(encrypted).unwrap_or_default(),
        )
    } else {
        let cypher = base64::decode(encrypted).unwrap_or_default();
        let salt = &cypher[8..16];
        let password = [key_or_secret.as_bytes(), salt].concat();

        let mut md5_hashes = Vec::new();
        let mut digest = password.clone();
        for _ in 0..3 {
            let hash = md5::compute(&digest);
            md5_hashes.push(hash.0.to_vec());
            digest = [hash.0.to_vec(), password.clone()].concat();
        }

        let key = [&md5_hashes[0][..], &md5_hashes[1][..]].concat();
        let nonce = md5_hashes[2][..].to_vec();
        let contents = cypher[16..].to_vec();

        (key, nonce, contents)
    };

    let decrypted = decrypt_aes_256_cbc(&nonce, &key, &contents);

    Ok(String::from_utf8(decrypted)?)
}

fn extract_video_id(video_url: &str) -> String {
    video_url
        .split('/')
        .last()
        .and_then(|s| s.split('?').next())
        .unwrap_or_default()
        .to_string()
}

async fn fetch_initial_data(url: &str, proxies: &[Proxy]) -> Result<Value, AniRustError> {
    let response = get_curl(url, proxies).await?;
    serde_json::from_str(&response).map_err(|e| AniRustError::UnknownError(e.to_string()))
}

fn parse_json_field<T: serde::de::DeserializeOwned>(
    json: &Value,
    field: &str,
) -> Result<T, AniRustError> {
    serde_json::from_value(json[field].clone())
        .map_err(|e| AniRustError::UnknownError(format!("Failed to parse {}: {}", field, e)))
}

fn extract_encrypted_string(json: &Value) -> String {
    if let Some(data) = json.get("sources") {
        serde_json::from_str::<String>(data.to_string().as_str()).unwrap_or_default()
    } else {
        String::new()
    }
}

async fn decrypt_sources(
    encrypted_string: &str,
    proxies: &[Proxy],
) -> Result<String, AniRustError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AniRustError::UnknownError(e.to_string()))?
        .as_millis();

    let full_url = format!(
        "https://megacloud.tv/js/player/a/prod/e1-player.min.js?v={}",
        now
    );
    let script = get_curl(&full_url, proxies).await?;

    let variables = extract_variables(&script)?;
    if variables.is_empty() {
        return Err(AniRustError::UnknownError(
            "Can't find variables. Perhaps the extractor is outdated.".to_string(),
        ));
    }

    let (secret, encrypted_source) = get_secret(encrypted_string, &variables);
    let decrypted = decrypt(&encrypted_source, &secret, None)?;
    Ok(decrypted)
}

fn parse_sources(decrypted: &str) -> Result<Vec<Source>, AniRustError> {
    serde_json::from_str(decrypted)
        .map_err(|e| AniRustError::UnknownError(format!("Failed to parse sources: {}", e)))
}

fn get_payload(hex: &str) -> String {
    // `5363587530696d33443675687c7c{hex}7c7c433569475830474c497a65767c7c73747265616d7362`;
    let payload = format!("566d337678566f743674494a7c7c{}7c7c346b6767586d6934774855537c7c73747265616d7362/6565417268755339773461447c7c346133383438333436313335376136323337373433383634376337633465366534393338373136643732373736343735373237613763376334363733353737303533366236333463353333363534366137633763373337343732363536313664373336327c7c6b586c3163614468645a47617c7c73747265616d7362", hex);

    payload
}

fn get_encoded_video_id(video_url: &str) -> Vec<u8> {
    let mut id = video_url
        .split("/e/")
        .last()
        .unwrap_or_default()
        .to_string();

    if id.contains("html") {
        id = id.split(".html").next().unwrap_or_default().to_string();
    }

    id.as_bytes().to_vec()
}

async fn process_streamsb_url(
    is_alt: Option<bool>,
    hexed_id: &str,
    proxies: &[Proxy],
) -> Result<String, AniRustError> {
    let host = if matches!(is_alt, Some(true)) {
        &STREAMSB.host2
    } else {
        &STREAMSB.host1
    };

    let url = format!("{}/{}", host, hexed_id);
    let res = get_curl(&url, proxies).await?;

    Ok(res)
}
