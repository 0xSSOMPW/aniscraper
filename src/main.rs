use std::fmt::Display;

use error::AniRustError;
use utils::parse_usize;

mod env;
mod error;
mod macros;
mod model;
mod proxy;
mod schema;
mod utils;

#[tokio::main]
async fn main() {
    println!("{:?}", parse_usize("1s"));
    // if let Err(e) = parse_usize("ss") {
    //     let error_mess = format!("{}", e);
    //     AniRustError::send_error_to_webhook(&e.webhook_url(), &error_mess).await;
    // }
}
