use crate::app::Song;
use anyhow::anyhow;
use reqwest::Client;
use serde_json::Value;

const SEARCH_SUFFIX: &str = "karaoke version";

// fetch_videos from the YouTube API.
pub async fn search(
    api_key: &str,
    query: &str,
) -> anyhow::Result<Vec<Song>> {
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
        items: Vec<YoutubeItem>,
    }

    let json_response = serde_json::from_value::<YoutubeResponse>(json)?;
    let songs: Vec<_> = json_response.items.iter().map(|item| {
        Song {
            video_id: item.id.video_id.to_owned(),
            // Filled from lyrics search
            lyric_id: String::new(),
            track_name: String::new(),
            artist_name: String::new(),
            synced_lyrics: String::new(),
            lyric_map: None,
            message: (),
        }
    }).collect();

    Ok(songs) // Return the list of songs
}