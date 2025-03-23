use crate::app::{LyricsMap, Song};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use std::{thread, u64};

pub(crate) mod error;




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
    pub(crate) synced_lyrics: LyricsMap,
}

#[derive(Debug, Clone)]
pub struct DisplayLyric {
    pub timestamp: u64,
    pub text: String,
}

pub async fn search(query: &str) -> Result<Vec<Song>, anyhow::Error> {
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

    let json: Vec<Value> = response
        .json()
        .await
        .expect("should parse the value as json"); // Parse the response body as JSON array

    // Do this idiomatic idiomat.
    let lyrics = json
        .iter()
        .cloned()
        .filter_map(|v| match serde_json::from_value::<Song>(v) {
            Ok(song) => Some(song),
            Err(e) => {
                println!("Failed to parse lyric: {}", e);
                None
            }
        })
        .map(|l| {
            let lyrics_map = raw_to_lyrics_map(&l.synced_lyrics).unwrap();

            return Song {
                lyric_map: Some(lyrics_map),
                ..l
            }
        })
        .collect();

    Ok(lyrics)
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

// TODO: Might have a better way to do this based on response... Deal with later.
fn raw_to_lyrics_map(synced_lyric_str: &str) -> anyhow::Result<LyricsMap> {
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
