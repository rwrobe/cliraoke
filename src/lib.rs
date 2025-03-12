use reqwest::Client;
use reqwest::blocking;
use rodio::{Decoder, OutputStream, Sink};
use serde_json::Value;
use std::error::Error;
use std::process::{Command, exit};
use std::{io::Cursor, thread, time::Duration};
use tokio::runtime::Runtime;

pub(crate) async fn run(api_key: &str, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch videos using the YouTube API
    match fetch_videos(&api_key, query).await {
        Ok(videos) => {
            println!("Fetched {} videos", videos.len());

            if videos.is_empty() {
                println!("No videos found");
                exit(1)
            }

            // Give song options.
            let video_id = present_options(videos);

            // Use the result of present_options to get the audio URL and stream it to ffmpeg.
            let audio_url = get_audio_url(video_id.unwrap().unwrap().trim());

            stream_audio(audio_url.as_str());

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

fn present_options(videos: Vec<Value>) -> Result<Option<String>, Box<dyn std::error::Error>> {
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
        return Ok(Some(video_id));
    }

    Ok(None)
}

// Get the direct audio stream URL using yt-dlp
fn get_audio_url(video_id: &str) -> String {
    let output = Command::new("yt-dlp")
        // output to stdout
        .args(&[
            video_id,
            "-o",
            "-",
            "-f",
            "bestaudio",
            "--merge-output-format",
            "mkv",
        ])
        .output()
        .expect("Failed to execute yt-dlp");

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    url
}

// Stream audio from the URL
fn stream_audio(url: &str) {
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build runtime");

            runtime.block_on(async {
                play_audio(url).await;
            });
        });
    } else {
        let rt = Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            play_audio(url).await;
        });
    }
}

async fn play_audio(url: &str) {
    println!("Playing audio from {}...", url);
    exit(0);
    let response = reqwest::get(url)
        .await
        .expect("Failed to fetch audio stream");
    let bytes = response.bytes().await.expect("Failed to read bytes");

    let cursor = Cursor::new(bytes);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let source = Decoder::new(cursor).unwrap();

    sink.append(source);
    sink.sleep_until_end();
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
