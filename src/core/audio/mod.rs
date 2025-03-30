pub(crate) mod audio {
    use anyhow::anyhow;
    use reqwest::Client;
    use serde_json::Value;
    use std::process::Command;

    const SEARCH_SUFFIX: &str = "karaoke version";

    pub struct Video {
        pub id: String,
        pub title: String,
        pub artist: String,
    }

    pub async fn fetch_videos(
        api_key: &str,
        query: &str,
    ) -> anyhow::Result<Vec<Video>> {
        let client = Client::new(); // Create a new HTTP client
        let page_token = String::new(); // Token to handle pagination
        let max_results = 5; // Maximum number of results per page

        // Build the API request URL
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?key={}&q={}&part=snippet,id&order=relevance&maxResults={}&type=video&pageToken={}",
            api_key,
            format!("{} {}", query, SEARCH_SUFFIX),
            max_results,
            page_token
        );

        let response = client
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
            items: Vec<YoutubeItem>
        }

        let json_response = serde_json::from_value::<YoutubeResponse>(json)?;
        let videos: Vec<_> = json_response.items.iter().map(|item| {
            Video {
                id: item.id.video_id.to_owned(),
                title: item.snippet.title.to_owned(),
                artist: item.snippet.channel_title.to_owned(),
            }
        }).collect();

        Ok(videos) // Return the list of videos
    }

    // Get the direct audio stream URL using yt-dlp
    pub fn get_youtube_audio_url(video_id: &str) -> Option<String> {
        let url = format!("https://www.youtube.com/watch?v={}", video_id);

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

    pub fn play_audio(url: &str) {
        // Create a Command to run ffplay with silenced output
        let mut cmd = Command::new("ffplay");

        // Add arguments
        cmd.args(["-nodisp", "-autoexit", "-loglevel", "quiet", url]);

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
}
