use crate::{
    env::EnvVar,
    error::AniRustError,
    proxy::{get_random_proxy, Proxy},
};
use brotli::Decompressor;
use core::fmt;
use flate2::read::{GzDecoder, ZlibDecoder};
use hex::FromHexError;
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper_tls::HttpsConnector;
use hyper_util::rt::TokioExecutor;
use openssl::symm::{Cipher, Crypter, Mode};
use reqwest::header;
use serde_json::Value;
use std::io::Read;
use std::time::Duration;

/// Fetches data from the specified URL.
///
/// Returns the HTML content of the page as a string.
/// TODO: find a way to do it using hyper , to reduce no of dependencies
pub async fn get_curl(url: &str, proxies: &[Proxy]) -> Result<String, AniRustError> {
    let max_attempts = parse_usize(&EnvVar::MAX_RETRIES_ATTEMPTS.get_config()).unwrap_or(50);
    let timeout_duration = Duration::from_secs(5);

    let mut attempt = 0;
    while attempt < max_attempts {
        let client = if proxies.is_empty() {
            reqwest::Client::builder()
                .timeout(timeout_duration)
                .build()?
        } else {
            let proxy = get_random_proxy(proxies).unwrap();
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::http(&proxy.address)?)
                .timeout(timeout_duration)
                .build()?
        };

        let response = match client
            .get(url)
            .header(header::USER_AGENT, &EnvVar::USER_AGENT_HEADER.get_config())
            .header(
                header::ACCEPT_ENCODING,
                &EnvVar::ACCEPT_ENCODING_HEADER.get_config(),
            )
            .header(header::ACCEPT, &EnvVar::ACCEPT_HEADER.get_config())
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_) if attempt < max_attempts - 1 => {
                attempt += 1;
                continue;
            }
            Err(e) => return Err(AniRustError::ReqwestError(e)),
        };

        let res_headers = response.headers().to_owned();
        let content_encoding = res_headers
            .get(header::CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let res_bytes = response.bytes().await?;

        let body = match content_encoding {
            "gzip" => {
                let mut decoded = Vec::new();
                let mut gz = GzDecoder::new(res_bytes.as_ref());
                gz.read_to_end(&mut decoded).unwrap_or_default();
                String::from_utf8(decoded).unwrap_or_default()
            }
            "deflate" => {
                let mut zf = ZlibDecoder::new(res_bytes.as_ref());
                let mut decoded = Vec::new();
                zf.read_to_end(&mut decoded).unwrap_or_default();
                String::from_utf8(decoded).unwrap_or_default()
            }
            "br" => {
                let mut decompressor = Decompressor::new(res_bytes.as_ref(), 4096);
                let mut decoded = Vec::new();
                decompressor.read_to_end(&mut decoded).unwrap_or_default();
                String::from_utf8(decoded).unwrap_or_default()
            }
            _ => String::from_utf8(res_bytes.to_vec()).unwrap_or_default(),
        };

        return Ok(body);
    }

    Err(AniRustError::FailedToFetchAfterRetries)
}

pub fn parse_usize(s: &str) -> Result<usize, AniRustError> {
    s.to_string()
        .parse::<usize>()
        .map_err(AniRustError::ParseIntError)
}

pub fn stringify<T: fmt::Display>(input: T) -> String {
    format!("{}", input)
}

pub fn anirust_error_vec_to_string(error_vec: Vec<Option<AniRustError>>) -> String {
    error_vec
        .iter()
        .filter_map(|opt_error| opt_error.as_ref().map(|e| e.to_string()))
        .collect::<Vec<String>>()
        .join(", ")
}

// TODO: find a way to impl proxies
pub async fn get_ajax_curl(url: &str, field: &str) -> Result<String, AniRustError> {
    // Create an HTTPS connector
    let https = HttpsConnector::new();
    // Create an HTTP client
    let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
        .build::<_, Empty<Bytes>>(https);

    // Define the URL
    let url = url.parse().unwrap_or_default();

    // Make the GET request
    let res = client
        .get(url)
        .await
        .unwrap()
        .map_err(|_| AniRustError::UnknownError("hyper client failed".to_string()));

    // Collect the response body
    let body = res.collect().await?;
    let body_bytes = body.to_bytes();

    // Convert body bytes to a UTF-8 string
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

    // Parse the string as JSON
    let json_value = serde_json::from_str::<Value>(&body_string).unwrap_or_default();

    match json_value.get(field) {
        Some(data) => {
            Ok(serde_json::from_str::<String>(data.to_string().as_str()).unwrap_or_default())
        }
        None => Ok(String::new()),
    }
}

pub fn substring_after(str: &str, to_find: &str) -> String {
    let index = str.find(to_find);
    match index {
        Some(i) => str[i + to_find.len()..].to_string(),
        None => String::new(),
    }
}

pub fn substring_before(str: &str, to_find: &str) -> String {
    let index = str.find(to_find);
    match index {
        Some(i) => str[..i].to_string(),
        None => String::new(),
    }
}

// Encrypts a plaintext message using AES-256 in CBC (Cipher Block Chaining) mode.
// //
// // # Arguments
// //
// // * `iv` - An initialization vector (IV) used to provide the starting state for encryption.
// Must be 16 bytes long.
// // * `key` - A secret key used to perform encryption. Must be 32 bytes long for AES-256.
// // * `text` - The plaintext message to be encrypted, represented as a byte slice.
// //
// // # Returns
// //
// // Returns a `Vec<u8>` containing the encrypted ciphertext.
// //
// // # Example
// //
// // ```rust
// // use aes::encrypt_aes_256_cbc;
// // let iv = b"1234567890123456";
// // let key = b"01234567890123456789012345678901";
// // let text = b"Hello, world!";
// // let ciphertext = encrypt_aes_256_cbc(iv, key, text);
// // assert_eq!(ciphertext, [197, 221, 61, 37, 184, 139, 38, 189, 182, 53, 144, 45, 170, 182, 220,
// 210, 64, 155, 36, 239, 138, 38, 33, 48, 25, 101, 160, 99, 8, 24, 111, 137]);
// // ```
// //
// // # Errors
// //
// // This function will panic if the initialization vector or key lengths are incorrect.
pub fn encrypt_aes_256_cbc(iv: &[u8], key: &[u8], text: &[u8]) -> Vec<u8> {
    let cipher = Cipher::aes_256_cbc();
    let mut encrypter = Crypter::new(cipher, Mode::Encrypt, key, Some(iv)).unwrap();

    let block_size = cipher.block_size();
    let mut encrypted_data = vec![0; text.len() + block_size];
    let count = encrypter
        .update(text, &mut encrypted_data)
        .unwrap_or_default();
    let rest = encrypter
        .finalize(&mut encrypted_data[count..])
        .unwrap_or_default();
    encrypted_data.truncate(count + rest);

    encrypted_data
}

// Decrypts an AES-256 encrypted message using CBC (Cipher Block Chaining) mode.
// //
// // # Arguments
// //
// // * `iv` - An initialization vector (IV) used during the encryption process. Must be 16 bytes
// long.
// // * `key` - A secret key used to perform decryption. Must be 32 bytes long for AES-256.
// // * `encrypted_data` - The ciphertext message to be decrypted, represented as a byte slice.
// //
// // # Returns
// //
// // Returns a `Vec<u8>`
// //
// // # Example
// //
// // ```rust
// // use aes::decrypt_aes_256_cbc;
// // use hex::FromHexError;
// // let iv = b"1234567890123456";
// // let key = b"01234567890123456789012345678901";
// // let ciphertext = [202, 234, 210, 161, 130, 10, 237, 118, 76, 66, 52, 178, 12, 8, 2, 241];
// // let plaintext = decrypt_aes_256_cbc(iv, key, &ciphertext).unwrap();
// // assert_eq!(plaintext, b"Hello, world!");
// // ```
// //
// // # Errors
// //
// // Returns an error if decryption fails, for example, due to incorrect key or initialization
// vector.
pub fn decrypt_aes_256_cbc(iv: &[u8], key: &[u8], encrypted_data: &[u8]) -> Vec<u8> {
    let cipher = Cipher::aes_256_cbc();
    let mut decrypter = Crypter::new(cipher, Mode::Decrypt, key, Some(iv)).unwrap();

    let block_size = cipher.block_size();
    let mut decrypted_data = vec![0; encrypted_data.len() + block_size];
    let count = decrypter
        .update(encrypted_data, &mut decrypted_data)
        .unwrap_or_default();
    let rest = decrypter
        .finalize(&mut decrypted_data[count..])
        .unwrap_or_default();
    decrypted_data.truncate(count + rest);

    decrypted_data
}
