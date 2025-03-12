mod lib;

use dotenv::dotenv;
use std::env;
use std::process::exit;
use crate::lib::run;

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";
const SEARCH_SUFFIX: &str = "karaoke";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from .env file

    let api_key = env::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");
    let query = format!("thieves in the night {}", SEARCH_SUFFIX);

    match run(&api_key, &query).await {
        Ok(result) => {
            println!("Success!");
            Ok(())
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
