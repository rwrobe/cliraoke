mod lib;

use yt_dlp::Youtube;
use std::path::PathBuf;
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
    // TODO: Add search query input.
    let query = format!("thieves in the night {}", SEARCH_SUFFIX);

    let executables_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");

    let fetcher = Youtube::with_new_binaries(executables_dir, output_dir).await?;

    match run(&api_key, &query, &fetcher).await {
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
