mod lib;

use dotenv::dotenv;
use lib::{fetch_videos, present_options};
use std::env;
use std::process::exit;

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";
const SEARCH_SUFFIX: &str = "karaoke";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from .env file

    let api_key = env::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");
    let query = format!("thieves in the night {}", SEARCH_SUFFIX);

    // Fetch videos using the YouTube API
    match fetch_videos(&api_key, query.as_str()).await {
        Ok(videos) => {
            println!("Fetched {} videos", videos.len());

            if videos.is_empty() {
                println!("No videos found");
                exit(1)
            }

            // Give song options.
            let video = present_options(videos)?;

            // Use the result of present_options to get the audio URL and stream it to ffmpeg.
            if let Some(video) = video {
                let video_id = video["id"]["videoId"].as_str().unwrap();
                let audio_url = lib::get_audio_url(video_id);

                if let Some(audio_url) = audio_url {
                    println!("Streaming audio from: {}", audio_url);
                    lib::stream_audio(audio_url.as_str());
                } else {
                    println!("Failed to get audio URL");
                }
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
