use crate::audio::{AudioFetcher, AudioResult, AudioService};
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::process::Command;
use std::time::Duration;

const SEARCH_SUFFIX: &str = "karaoke version";

#[derive(Debug, Clone)]
pub struct YouTube {
    pub api_key: String,
    pub http_ct: Client,
}

impl YouTube {
    pub(crate) fn new(api_key: String) -> Self {
        YouTube {
            api_key,
            http_ct: Client::new(),
        }
    }

    fn get_url(&self, id: &str) -> Option<String> {
        let url = format!("https://www.youtube.com/watch?v={}", id);

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

    async fn get_duration(&self, id: String) -> anyhow::Result<Duration> {
        // Build the API request URL
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?key={}&id={}&part=contentDetails",
            self.api_key, id,
        );

        let res = self
            .http_ct
            .get(&url)
            .header("Referer", "cliraoke") // Add referer header
            .send()
            .await?;

        // Check if the response was successful
        if !res.status().is_success() {
            println!("API request failed with status: {}", res.status());
            println!("Response body: {}", res.text().await?);
            return Err(anyhow!("API request failed"));
        }

        let json: Value = res.json().await?;

        #[derive(Debug, serde::Deserialize)]
        struct YTVideoResponse {
            items: Vec<YTVideoItem>,
        }

        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct YTVideoItem {
            content_details: YTContentDetails,
        }

        #[derive(Debug, serde::Deserialize)]
        struct YTContentDetails {
            duration: String,
        }

        let video_response = serde_json::from_value::<YTVideoResponse>(json)?;
        let duration_str = &video_response.items[0].content_details.duration;

        // Parse as ISO 8601 duration: https://developers.google.com/youtube/v3/docs/videos/list
        let duration = duration_str
            .parse::<iso8601_duration::Duration>()
            .unwrap()
            .to_std();

        match duration {
            Some(duration) => Ok(duration),
            None => {
                println!("Failed to parse duration: {}", duration_str);
                Err(anyhow!("Failed to parse duration"))
            }
        }
    }
}

#[async_trait]
impl AudioFetcher for YouTube {
    async fn search(&self, query: &str) -> anyhow::Result<Vec<AudioResult>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let page_token = String::new(); // Token to handle pagination
        let max_results = 5; // Maximum number of results per page

        // Build the API request URL
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?key={}&q={}&part=snippet,id&order=relevance&maxResults={}&type=video&pageToken={}",
            self.api_key,
            format!("{} {}", query, SEARCH_SUFFIX),
            max_results,
            page_token
        );

        let response = self
            .http_ct
            .get(&url)
            .header("Referer", "cliraoke") // Add referer header
            .send()
            .await?; // Send the HTTP GET request

        // Check if the response was successful
        if !response.status().is_success() {
            println!("API request failed with status: {}", response.status());
            println!("Response body: {}", response.text().await?);
            return Err(anyhow!("API request failed"));
        }

        let json: Value = response.json().await?; // Parse the response body as JSON

        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct YtId {
            video_id: String,
        }
        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct YtSnippet {
            title: String,
            channel_title: String,
        }
        #[derive(Debug, serde::Deserialize)]
        struct YoutubeItem {
            id: YtId,
            snippet: YtSnippet,
        }
        #[derive(Debug, serde::Deserialize)]
        struct YoutubeResponse {
            items: Vec<YoutubeItem>,
        }

        let json_response = serde_json::from_value::<YoutubeResponse>(json)?;
        let futures: Vec<_> = json_response
            .items
            .iter()
            .map(|item| {
                let video_id = item.id.video_id.to_owned();
                let title = item.snippet.title.to_owned();
                let artist = item.snippet.channel_title.to_owned();

                async move {
                    let mut res = AudioResult {
                        id: video_id.clone(),
                        title,
                        artist,
                        duration: Duration::new(0, 0), // Placeholder for duration
                    };

                    let d = self.get_duration(video_id).await;
                    match d {
                        Ok(duration) => {
                            res.duration = duration;
                        }
                        Err(e) => {
                            println!("Failed to get duration: {}", e);
                            res.duration = Duration::new(0, 0); // Default value
                        }
                    }

                    res
                }
            })
            .collect();

        let audios = futures::future::join_all(futures).await;

        Ok(audios)
    }
}

impl AudioService for YouTube {
    fn play(&self, id: &str) {
        let url =  self.get_url(id).expect("Failed to get url");

        // Create a Command to run ffplay with silenced output
        let mut cmd = Command::new("ffplay");

        // Add arguments
        cmd.args(["-nodisp", "-autoexit", "-loglevel", "quiet", url.as_str()]);

        // Redirect stdout and stderr to /dev/null (on Unix) or NUL (on Windows)
        #[cfg(target_family = "unix")]
        {
            cmd.stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
        }

        #[cfg(target_family = "windows")]
        {
            use std::process::Stdio;
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }

        // Run the command
        match cmd.status() {
            Ok(status) => {
                if status.success() {
                    println!("Audio playback completed via ffplay");
                } else {
                    println!("ffplay failed with status: {}", status);
                }
            }
            Err(e) => {
                println!("Failed to run ffplay: {}", e);
            }
        }
    }

    fn pause(&self) {
        todo!()
    }
}