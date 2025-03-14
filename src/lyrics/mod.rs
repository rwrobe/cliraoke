pub mod lyrics {
    use regex::Regex;
    use reqwest::{Client, blocking};
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::fmt::Display;
    use std::{thread, u64};
    use std::time::{Duration, Instant};
    use serde::{de, Deserialize, Deserializer};

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LyricResponse {
        pub(crate) id: u64,
        pub(crate) artist_name: String,
        pub(crate) track_name: String,
        pub(crate) synced_lyrics: String,
    }

    #[derive(Debug)]
    pub struct Lyric {
        pub(crate) id: String,
        pub(crate) artist: String,
        pub(crate) title: String,
        pub(crate) synced_lyrics: String,
    }


    pub async fn search_lyrics(query: &str) -> Result<Vec<Lyric>, Box<dyn std::error::Error>> {

        let client = Client::new(); // Create a new HTTP client
        // let mut lyrics = Vec::new(); // Initialize a vector to store videos
        let base_url = "https://lrclib.net/api/search";
        let url = format!("{}?q={}", base_url, query);

        let response = client
            .get(&url)
            .header("Referer", "https://lrclib.net") // Add referer header
            .send()
            .await?; // Send the HTTP GET request

        // Check if the response was successful
        if !response.status().is_success() {
            println!("API request failed with status: {}", response.status());
            println!("Response body: {}", response.text().await?);
            return Err("API request failed".into());
        }

        let json: Vec<Value> = response.json().await?; // Parse the response body as JSON array

        // Do this idiomatic idiomat.
        let lyrics = json
            .iter()
            .cloned()
            .filter_map(|v| {
                match serde_json::from_value::<LyricResponse>(v) {
                    Ok(lyric) => {
                        Some(lyric)
                    },
                    Err(e) => {
                        println!("Failed to parse lyric: {}", e);
                        None
                    }
                }
            })
            .map(|l| Lyric {
                id: l.id.to_string(),
                artist: l.artist_name,
                title: l.track_name,
                synced_lyrics: l.synced_lyrics,
            })
            .collect();

        Ok(lyrics)
    }

    pub fn fetch_lyrics(id: &str) -> Option<BTreeMap<u64, String>> {
        let url = format!("https://lrclib.net/api/get/{}", id);

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

        Some(time_to_lyric)
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
}
