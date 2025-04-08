use crate::lyrics::{LyricResponse, LyricsFetcher, LyricsResult, LyricsService};
use crate::models::song::LyricsMap;
use async_trait::async_trait;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use std::thread;

#[derive(Clone)]
pub struct LRCLib;

impl LRCLib {
    pub fn new() -> Self {
        LRCLib
    }
}

#[async_trait]
impl LyricsFetcher for LRCLib {
    async fn search(&self, query: &str) -> anyhow::Result<Vec<LyricsResult>> {
        if query.is_empty() {
            return Ok(Vec::new())
        }

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
            .map(|l| {
                let synced = l.synced_lyrics.clone();
                LyricsResult {
                    id: l.id.to_string(),
                    artist: l.artist_name,
                    title: l.track_name,
                    // Again, we should handle this case better elsewhere, this is placeholder
                    // and is never read by the application using this data.
                    synced_lyrics: l.synced_lyrics.unwrap_or("".to_owned()),
                    lyric_map: None,
                }
            })
            .collect();

        Ok(lyrics)
    }

    async fn parse(&self, synced: String) -> anyhow::Result<Option<LyricsMap>> {
        // Create regex to extract timestamp and text
        let re = Regex::new(r"^\[(\d+):(\d+)\.(\d+)\]\s*(.*)$")?;

        // Create a map to store timestamp -> lyric pairs
        let mut time_to_lyric = LyricsMap::new();

        // Process each line
        for line in synced.lines() {
            if let Some(captures) = re.captures(line) {
                // Convert timestamp parts to milliseconds as u64
                let minutes: u64 = captures[1].parse()?;
                let seconds: u64 = captures[2].parse()?;
                let milliseconds: u64 = captures[3].parse()?;

                // Calculate total milliseconds
                let timestamp_ms = minutes * 60_000 + seconds * 1000 + milliseconds;
                // Round to the nearest 200ms.
                let timestamp_ms = (timestamp_ms / 200) * 200;

                // Get the lyric text
                let lyric_text = captures[4].to_string();

                // Store in map
                time_to_lyric.insert(timestamp_ms, lyric_text);
            }
        }

        Ok(Some(time_to_lyric))
    }
}

impl LyricsService for LRCLib {
    fn play(&self, elapsed_time_ms: u64, lyrics_map: LyricsMap) -> anyhow::Result<String> {
        if let Some(lyric) = lyrics_map.get(&elapsed_time_ms) {
            // Calculate minutes and seconds for display
            let minutes = elapsed_time_ms / 60000;
            let seconds = (elapsed_time_ms % 60000) / 1000;
            let millis = elapsed_time_ms % 1000;

            // Return timestamp and lyric
            return Ok(lyric.into());
        }

        // If no lyric found for the timestamp, return an empty string
        Ok("".to_string())
    }
}
