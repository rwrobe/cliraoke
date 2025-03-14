use reqwest::Client;
use reqwest::blocking;
use reqwest::blocking::get;
use rodio::{Decoder, OutputStream, Sink};
use serde_json::Value;
use std::error::Error;
use std::io::BufReader;
use std::process::{Command, exit};
use std::{io::Cursor, thread, time::{Duration, Instant}};
use tokio::runtime::Runtime;
use yt_dlp::Youtube;
use std::collections::BTreeMap;
use regex::Regex;
use std::sync::{Arc, Mutex};

pub(crate) async fn run(api_key: &str, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    match fetch_videos(api_key, query).await {
        Ok(videos) => {
            println!("Fetched {} videos", videos.len());

            if videos.is_empty() {
                println!("No videos found");
                exit(1)
            }

            // Present options is a blocking operation, use spawn_blocking
            let video_id = tokio::task::spawn_blocking(move || present_yt_options(videos)).await?;

            if let Some(video_id) = video_id {
                // Get YouTube audio URL is also blocking
                let video_id_owned = video_id.clone(); // Clone to create an owned value
                let audio_url =
                    tokio::task::spawn_blocking(move || get_youtube_audio_url(&video_id_owned))
                        .await?;

                if let Some(audio_url) = audio_url {
                    // Fetch lyrics in a separate thread
                    let lyrics_handle = tokio::task::spawn_blocking(move || {
                        fetch_lyrics()
                    });

                    // Start playing audio
                    let play_handle = std::thread::spawn(move || {
                        play_audio(&audio_url);
                    });

                    // Wait for lyrics to be retrieved
                    if let Ok(Some(lyrics_map)) = lyrics_handle.await {
                        // Display lyrics in sync with playback
                        display_synced_lyrics(&lyrics_map);
                    } else {
                        println!("No lyrics available for this track.");
                    }

                    // Wait for audio playback to finish
                    if let Err(e) = tokio::signal::ctrl_c().await {
                        println!("Error waiting for Ctrl+C: {}", e);
                    }

                    // Audio playback has stopped
                    println!("Playback ended");
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

fn present_yt_options(videos: Vec<Value>) -> Option<String> {
    println!("Pick your song (check the URL if you're not sure):");

    for (index, video) in videos.iter().enumerate() {
        let snippet = &video["snippet"];

        println!(
            "{}. Title: {}; URL: https://www.youtube.com/watch?v={}",
            index + 1,
            snippet["title"],
            video["id"]["videoId"].as_str().unwrap()
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

// Get the direct audio stream URL using yt-dlp
fn get_youtube_audio_url(video_id: &str) -> Option<String> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    println!("Getting audio URL for YouTube video: {}", url);

    let output = Command::new("yt-dlp")
        .args([
            "-f",
            "bestaudio",
            "--get-url",
            "--extract-audio",
            "--audio-format",
            "mp3",
            &url,
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
            let audio_url = String::from_utf8_lossy(&retry_output.stdout)
                .trim()
                .to_string();
            println!("Retry successful. Got audio URL: {}", audio_url);
            Some(audio_url)
        } else {
            eprintln!(
                "Retry also failed: {}",
                String::from_utf8_lossy(&retry_output.stderr)
            );
            None
        }
    }
}

fn play_audio(url: &str) {
    println!("Playing audio from {}...", url);
    println!("Lyrics will be displayed as the song plays.");

    // Create a Command to run ffplay with silenced output
    let mut cmd = Command::new("ffplay");

    // Add arguments
    cmd.args(["-nodisp", "-autoexit", "-loglevel", "quiet", url]);

    // Redirect stdout and stderr to /dev/null (on Unix) or NUL (on Windows)
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::process::CommandExt;
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
    }

    #[cfg(target_family = "windows")]
    {
        use std::process::Stdio;
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null());
    }

    // Run the command
    match cmd.status() {
        Ok(status) => {
            if status.success() {
                println!("Audio playback completed via ffplay");
            } else {
                println!("ffplay failed with status: {}", status);
            }
        },
        Err(e) => {
            println!("Failed to run ffplay: {}", e);
        }
    }
}

// Fetch lyrics from Musixmatch API
fn fetch_lyrics() -> Option<BTreeMap<u64, String>> {
    let url = format!("https://lrclib.net/api/get/{}", 17533182);
    println!("Fetching lyrics from {}", url);

    let response = blocking::get(&url).ok()?;

    if !response.status().is_success() {
        println!("Failed to fetch lyrics: {}", response.status());
        return None;
    }

    let json_response = response.json::<Value>().ok()?;

    // Check if we have synced lyrics
    let synced_raw = match json_response.get("syncedLyrics") {
        Some(lyrics) => lyrics.as_str()?,
        None => {
            println!("No synced lyrics found in response");
            // Check if lyrics are nested in message/body
            match json_response.get("message") {
                Some(message) => match message.get("body") {
                    Some(body) => match body.get("syncedLyrics") {
                        Some(lyrics) => lyrics.as_str()?,
                        None => {
                            println!("No synced lyrics found in message.body");
                            return None;
                        }
                    },
                    None => {
                        println!("No body found in message");
                        return None;
                    }
                },
                None => {
                    println!("No message found in response");
                    return None;
                }
            }
        }
    };

    println!("Processing synced lyrics");

    // Create regex to extract timestamp and text
    let re = Regex::new(r"^\[(\d+):(\d+)\.(\d+)\]\s*(.*)$").ok()?;

    // Create a map to store timestamp -> lyric pairs
    let mut time_to_lyric = BTreeMap::new();

    // Process each line
    for line in synced_raw.lines() {
        if let Some(captures) = re.captures(line) {
            // Convert timestamp parts to milliseconds as u64
            let minutes: u64 = captures[1].parse().ok()?;
            let seconds: u64 = captures[2].parse().ok()?;
            let milliseconds: u64 = captures[3].parse().ok()?;

            // Calculate total milliseconds
            let timestamp_ms = minutes * 60_000 + seconds * 1000 + milliseconds;

            // Get the lyric text
            let lyric_text = captures[4].to_string();

            // Store in map
            time_to_lyric.insert(timestamp_ms, lyric_text);
        }
    }

    println!("Processed {} lyric lines", time_to_lyric.len());
    Some(time_to_lyric)
}

// Display lyrics in sync with music playback
fn display_synced_lyrics(lyrics_map: &BTreeMap<u64, String>) {
    println!("Starting synchronized lyrics display");

    // Start a timer to track playback time
    let start_time = Instant::now();

    // Create a clone of the lyrics map that we can iterate through
    let timestamps: Vec<u64> = lyrics_map.keys().cloned().collect();

    // Track which lyrics we've already displayed
    let mut displayed_up_to_index = 0;

    // Set up a clean display area for lyrics
    println!("\n\n\n");  // Add some space before lyrics start
    println!("----- LYRICS -----");

    // Continue until we've displayed all lyrics
    while displayed_up_to_index < timestamps.len() {
        // Get current playback time in milliseconds
        let current_time_ms = start_time.elapsed().as_millis() as u64;

        // Check if we need to display new lyrics
        while displayed_up_to_index < timestamps.len() && current_time_ms >= timestamps[displayed_up_to_index] {
            let timestamp = timestamps[displayed_up_to_index];
            if let Some(lyric) = lyrics_map.get(&timestamp) {
                // Calculate minutes and seconds for display
                let minutes = timestamp / 60000;
                let seconds = (timestamp % 60000) / 1000;
                let millis = timestamp % 1000;

                // Print timestamp and lyric
                println!("[{:02}:{:02}.{:03}] {}", minutes, seconds, millis, lyric);
            }
            displayed_up_to_index += 1;
        }

        // Sleep briefly to avoid consuming too much CPU
        thread::sleep(Duration::from_millis(10));
    }

    println!("----- END OF LYRICS -----");
}