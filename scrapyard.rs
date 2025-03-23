fn thing() {
    // Get user query.
    println!("Welcome to CLIraoke.");
    print!("Please type a song or artist name: ");
    let query_base = cli::cli::get_user_input();

    // TODO: Run lyric and video fetching concurrently.
    // Show options for the query:
    let videos = audio::audio::fetch_videos(yt_api_key, query_base.as_str()).await;

    if videos.is_err() {
        println!(
            "Error fetching videos: {}",
            videos.err().and_then(|e| Some(e.to_string())).unwrap()
        );
        exit(1);
    }

    let videos = videos?; // Something about borrow checker? who knows
    if videos.is_empty() {
        println!("No videos found");
        exit(1);
    }

    // Get the video id from the user:
    let mut vid_opts: Vec<CLIOption> = Vec::new();
    for video in videos {
        let opt = CLIOption {
            artist: Some(video.artist),
            id: video.id,
            title: video.title,
        };
        vid_opts.push(opt);
    }

    let video_opt =
        tokio::task::spawn_blocking(move || cli::cli::present_options(vid_opts)).await?;

    // Get the lyrics for the query:
    let lyrs = lyrics::lyrics::search_lyrics(query_base.as_str()).await?;
    // if lyrs.is_err() {
    //     println!("We found your song, but had a problem finding the lyrics: {}", lyrs.err().unwrap());
    //     exit(1);
    // }

    if lyrs.is_empty() {
        println!("No lyrics found for this song.");
        exit(1);
    }

    println!("What a banger. OK, now select the lyrics to use: ");

    // Show options for the lyrics:
    let mut lyr_opts: Vec<CLIOption> = Vec::new();
    for lyr in lyrs {
        let opt = CLIOption {
            artist: Some(lyr.artist),
            id: lyr.id,
            title: lyr.title,
        };
        lyr_opts.push(opt);
    }

    let lyric_opt =
        tokio::task::spawn_blocking(move || cli::cli::present_options(lyr_opts)).await?;

    let lyrics_thread = tokio::task::spawn_blocking(move || {
        if let Some(opt) = lyric_opt {
            lyrics::lyrics::fetch_lyrics(opt.id.as_str())
        } else {
            None
        }
    });

    // Get the audio URL for the video:
    let mut audio_url: Option<String> = None;
    if let Some(video_opt) = video_opt {
        audio_url = tokio::task::spawn_blocking(move || {
            audio::audio::get_youtube_audio_url(video_opt.id.as_str())
        })
            .await?;
    }

    if audio_url.is_none() {
        println!("Error getting audio URL");
        exit(1);
    }

    // Wait for the lyrics to be ready before playing.
    match lyrics_thread.await {
        Ok(Some(lyrics_map)) => {
            let _play_thread = std::thread::spawn(move || {
                audio::audio::play_audio(audio_url.unwrap().as_str());
            });

            // Display lyrics in sync with playback
            lyrics::lyrics::display_synced_lyrics(&lyrics_map);

            // Wait for audio playback to finish
            if let Err(e) = tokio::signal::ctrl_c().await {
                println!("Error waiting for Ctrl+C: {}", e);
            }

            // Audio playback has stopped
            println!("Playback ended");
        }
        _ => {
            println!("No lyrics available for this track.");
        }
    }

    Ok(())
}

use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Deserializer};
use serde_json::{Number, Value};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use std::{thread, u64};

pub(crate) mod error;

fn deserialize_u64<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Number::deserialize(deserializer)?;
    Ok(s.to_string())
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricResponse {
    #[serde(deserialize_with = "deserialize_u64")]
    pub(crate) id: String,
    pub(crate) track_name: String,
    pub(crate) artist_name: String,
    _album_name: String,
    _duration: f64,
    _instrumental: bool,
    _plain_lyrics: Option<String>,
    pub(crate) synced_lyrics: Option<String>,
    // TODO: Is this even possible?
    pub(crate) message: Option<Message>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub(crate) synced_lyrics: Option<String>,
}

// This is for print purposes only, perhaps the abstraction should not be here but
// in the future print to std layer?
#[derive(Debug)]
pub struct OptionResponse {
    pub(crate) id: String,
    pub(crate) artist: String,
    pub(crate) title: String,
    pub(crate) synced_lyrics: String,
}

#[derive(Debug, Clone)]
pub struct DisplayLyric {
    pub timestamp: u64,
    pub text: String,
}

pub async fn search_lyrics(query: &str) -> Result<Vec<OptionResponse>, anyhow::Error> {
    let client = Client::new(); // Create a new HTTP client
    // let mut lyrics = Vec::new(); // Initialize a vector to store videos
    let base_url = "https://lrclib.net/api/search";
    let url = format!("{}?q={}", base_url, query);

    let response = client
        .get(&url)
        .header("Referer", "https://lrclib.net") // Add referer header
        .send()
        .await
        .expect("should get lrclib response"); // Send the HTTP GET request

    // TODO: We can do this with better error handling
    // if !response.status().is_success() {
    //     println!("API request failed with status: {}", response.status());
    //     println!("Response body: {}", response.text().await?);
    //     return Err(anyhow::anyhow!("API request failed"));
    // }

    let json: Vec<Value> = response
        .json()
        .await
        .expect("should parse the value as json"); // Parse the response body as JSON array

    // Do this idiomatic idiomat.
    let lyrics = json
        .iter()
        .cloned()
        .filter_map(|v| match serde_json::from_value::<LyricResponse>(v) {
            Ok(lyric) => Some(lyric),
            Err(e) => {
                println!("Failed to parse lyric: {}", e);
                None
            }
        })
        .map(|l| OptionResponse {
            id: l.id.to_string(),
            artist: l.artist_name,
            title: l.track_name,
            // Again, we should handle this case better elsewhere, this is placeholder
            // and is never read by the application using this data.
            synced_lyrics: l.synced_lyrics.unwrap_or("".to_owned()),
        })
        .collect();

    Ok(lyrics)
}

pub async fn fetch_lyrics(id: &str) -> anyhow::Result<Option<LyricResponse>> {
    let url = format!("https://lrclib.net/api/get/{}", id);

    let response = reqwest::get(&url).await?.json::<LyricResponse>().await?;

    // Check if we have synced lyrics
    if let Some(synced_lyrics) = response.synced_lyrics {
        let lyrics_map = raw_to_lyrics_map(&synced_lyrics)?;
        return Ok(Some(lyrics_map));
    }

    if let Some(synced_lyrics) = response.message.and_then(|m| m.synced_lyrics) {
        let lyrics_map = raw_to_lyrics_map(&synced_lyrics)?;
        return Ok(Some(lyrics_map));
    }

    // Print error where that is handled...
    Err(anyhow::anyhow!("No synced lyrics found in response"))
}

pub fn display_synced_lyrics(lyrics_map: &BTreeMap<u64, String>) {
    println!("Starting synchronized lyrics display");

    let start_time = Instant::now();

    let timestamps: Vec<u64> = lyrics_map.keys().cloned().collect();

    let mut displayed_up_to_index = 0;

    println!("\n\n\n");
    println!("----- LFG -----");

    // Continue until we've displayed all lyrics
    while displayed_up_to_index < timestamps.len() {
        // Get current playback time in milliseconds
        let current_time_ms = start_time.elapsed().as_millis() as u64;

        // Check if we need to display new lyrics
        // TODO this feels gross. Can we do it another way?
        while displayed_up_to_index < timestamps.len()
            && current_time_ms >= timestamps[displayed_up_to_index]
        {
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

    println!("----- BRAVO -----");
}

struct LyricsMap();

impl LyricsMap {
    fn new() -> Self {
        LyricsMap()
    }
}

// TODO: Might have a better way to do this based on response... Deal with later.
fn raw_to_lyrics_map(synced_lyric_str: &str) -> anyhow::Result<LyricResponse> {
    // Create regex to extract timestamp and text
    let re = Regex::new(r"^\[(\d+):(\d+)\.(\d+)\]\s*(.*)$")?;

    // Create a map to store timestamp -> lyric pairs
    let mut time_to_lyric = LyricsMap::new();

    // Process each line
    for line in synced_lyric_str.lines() {
        if let Some(captures) = re.captures(line) {
            // Convert timestamp parts to milliseconds as u64
            let minutes: u64 = captures[1].parse()?;
            let seconds: u64 = captures[2].parse()?;
            let milliseconds: u64 = captures[3].parse()?;

            // Calculate total milliseconds
            let timestamp_ms = minutes * 60_000 + seconds * 1000 + milliseconds;

            // Get the lyric text
            let lyric_text = captures[4].to_string();

            // Store in map
            time_to_lyric.insert(timestamp_ms, lyric_text);
        }
    }
    Ok(time_to_lyric)
}
