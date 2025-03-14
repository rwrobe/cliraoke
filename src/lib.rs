use reqwest::Client;
use reqwest::blocking;
use reqwest::blocking::get;
use rodio::{Decoder, OutputStream, Sink};
use serde_json::Value;
use std::error::Error;
use std::io::BufReader;
use std::process::{Command, exit};
use std::{io::Cursor, thread, time::Duration};
use tokio::runtime::Runtime;
use yt_dlp::Youtube;

pub(crate) async fn run(api_key: &str, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    match fetch_videos(api_key, query).await {
        Ok(videos) => {
            println!("Fetched {} videos", videos.len());

            if videos.is_empty() {
                println!("No videos found");
                exit(1)
            }

            // Present options is a blocking operation, use spawn_blocking
            let video_id = tokio::task::spawn_blocking(move || {
                present_options(videos)
            }).await?;

            if let Some(video_id) = video_id {
                // Get YouTube audio URL is also blocking
                let video_id_owned = video_id.clone(); // Clone to create an owned value
                let audio_url = tokio::task::spawn_blocking(move || {
                    get_youtube_audio_url(&video_id_owned)
                }).await?;

                if let Some(audio_url) = audio_url {
                    play_audio(&audio_url);

                    // Wait for a bit to allow the audio to start playing
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    // Keep the main thread alive until user interrupts
                    println!("Press Ctrl+C to stop playback.");
                    tokio::signal::ctrl_c().await?;
                }
            }

            Ok(())
        }
        Err(e) => {
            println!("Error fetching videos: {}", e);
            if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                if let Some(status) = reqwest_err.status() {
                    println!("HTTP Status {}", status);
                }
            }

            Err("Error fetching videos".into())
        }
    }
}

fn present_options(videos: Vec<Value>) -> Option<String> {
    println!("Pick your song (check the URL if you're not sure):");

    for (index, video) in videos.iter().enumerate() {
        let snippet = &video["snippet"];

        println!(
            "{}. Title: {}; URL: https://www.youtube.com/watch?v={}",
            index + 1,
            snippet["title"],
            video["id"]["videoId"]
        );
    }

    let mut input = String::new();

    std::io::stdin().read_line(&mut input).unwrap();

    if let Ok(index) = input.trim().parse::<usize>() {
        let video = &videos[index - 1];
        let video_id = video["id"]["videoId"].as_str().unwrap().to_string();
        let video_url = format!("https://www.youtube.com/watch?v={}", video_id);

        println!(
            "Rock on. Playing: {} ({})",
            video["snippet"]["title"], video_url
        );
        return Some(video_id);
    }

    None
}

async fn fetch_videos(
    api_key: &str,
    query: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = Client::new(); // Create a new HTTP client
    let mut videos = Vec::new(); // Initialize a vector to store videos
    let mut page_token = String::new(); // Token to handle pagination
    let max_results = 5; // Maximum number of results per page

    // Build the API request URL
    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?key={}&q={}&part=snippet,id&order=relevance&maxResults={}&type=video&pageToken={}",
        api_key, query, max_results, page_token
    );

    let response = client
        .get(&url)
        .header("Referer", "oke-doke") // Add referer header
        .send()
        .await?; // Send the HTTP GET request

    // Check if the response was successful
    if !response.status().is_success() {
        println!("API request failed with status: {}", response.status());
        println!("Response body: {}", response.text().await?);
        return Err("API request failed".into());
    }

    let json: Value = response.json().await?; // Parse the response body as JSON

    // Check for API errors
    if let Some(error) = json.get("error") {
        print!("API returned an error: {:?}", error);
        return Err("API returned an error".into());
    }

    // Extract video items and add to the videos vector
    if let Some(items) = json["items"].as_array() {
        videos.extend(items.clone());
    }

    // Handle pagination by checking for the nextPageToken
    if let Some(next_page_token) = json["nextPageToken"].as_str() {
        page_token = next_page_token.to_string();
    }

    Ok(videos) // Return the list of videos
}

// Get the direct audio stream URL using yt-dlp
fn get_youtube_audio_url(video_id: &str) -> Option<String> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    println!("Getting audio URL for YouTube video: {}", url);

    let output = Command::new("yt-dlp")
        .args([
            "-f", "bestaudio",
            "--get-url",
            "--extract-audio",
            "--audio-format", "mp3",
            &url
        ])
        .output()
        .expect("Failed to execute yt-dlp");

    if output.status.success() {
        let audio_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("Successfully got audio URL: {}", audio_url);
        Some(audio_url)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("yt-dlp failed: {}", error);

        // Try a different approach - maybe without audio format specification
        println!("Retrying with simplified parameters...");
        let retry_output = Command::new("yt-dlp")
            .args(["-f", "bestaudio", "--get-url", &url])
            .output()
            .expect("Failed to execute yt-dlp");

        if retry_output.status.success() {
            let audio_url = String::from_utf8_lossy(&retry_output.stdout).trim().to_string();
            println!("Retry successful. Got audio URL: {}", audio_url);
            Some(audio_url)
        } else {
            eprintln!("Retry also failed: {}", String::from_utf8_lossy(&retry_output.stderr));
            None
        }
    }
}

fn play_audio(url: &str) {
    println!("Playing audio from {}...", url);

    let status = Command::new("ffplay")
        .args(["-nodisp", "-autoexit", url])
        .status();

    if let Ok(status) = status {
        if status.success() {
            println!("Audio playback completed via ffplay");
            return;
        } else {
            println!("ffplay failed with status: {}", status);
        }
    }
}

// Fetch lyrics from Musixmatch API
fn fetch_lyrics(api_key: &str) -> Option<String> {
    let url = format!(
        "https://api.musixmatch.com/ws/1.1/track.search?q_track=Thieves%20in%20the%20Night&q_artist=Black%20Star&apikey={}",
        api_key
    );

    let response = blocking::get(&url).ok()?.json::<Value>().ok()?;
    let track_list = response["message"]["body"]["track_list"].as_array()?;

    if !track_list.is_empty() {
        let track_id = track_list[0]["track"]["track_id"].as_i64()?;
        let lyrics_url = format!(
            "https://api.musixmatch.com/ws/1.1/track.lyrics.get?track_id={}&apikey={}",
            track_id, api_key
        );

        let lyrics_response = blocking::get(&lyrics_url).ok()?.json::<Value>().ok()?;
        return lyrics_response["message"]["body"]["lyrics"]["lyrics_body"]
            .as_str()
            .map(|s| s.to_string());
    }
    None
}

// Display lyrics in sync
fn display_lyrics(lyrics: &str) {
    let lines: Vec<&str> = lyrics.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        thread::sleep(Duration::from_secs(i as u64 * 5)); // Adjust timing
        println!("{}", line);
    }
}
