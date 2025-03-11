mod lib;

use dotenv::dotenv;
use std::env;
use lib::{fetch_videos, write_to_csv};

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from .env file

    let api_key = env::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");
    let channel_id = "UCBRxDSTfr2aJVODDh4WG_7g"; // Change to your desired channel ID

    // Fetch videos using the YouTube API
    match fetch_videos(&api_key, channel_id).await {
        Ok(videos) => {
            println!("Fetched {} videos", videos.len());

            if videos.is_empty() {
                println!("No videos found");
            } else {
                // Call the function to write videos to a CSV file
                write_to_csv(videos)?;
                println!("Videos written to CSV");
            }
        }
        Err(e) => {
            println!("Error fetching videos: {}", e);
            if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                if let Some(status) = reqwest_err.status() {
                    println!("HTTP Status {}", status);
                }
            }
        }
    }

    Ok(())
}