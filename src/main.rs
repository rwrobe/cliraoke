mod audio;
mod cli;
mod lyrics;

use dotenv::dotenv;
use std::env;
use std::process::exit;
use crate::cli::cli::CLIOption;

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var(ENV_API_KEY).expect("YOUTUBE_API_KEY must be set");

    // Get user query.
    println!("Welcome to CLIraoke.");
    println!("Please type a song or artist name: ");
    let query_base = cli::cli::get_user_input();

    // TODO: Run lyric and video fetching concurrently.
    // Show options for the query:
    let videos = audio::audio::fetch_videos(api_key.as_str(), query_base.as_str()).await;

    if videos.is_err() {
        println!("Error fetching videos: {}", videos.err().unwrap());
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

    let video_opt = tokio::task::spawn_blocking(move || {
        cli::cli::present_options(vid_opts)
    }).await?;

    // Get the lyrics for the query:
    let lyrs = lyrics::lyrics::search_lyrics(query_base.as_str()).await;
    if lyrs.is_err() {
        println!("We found your song, but had a problem finding the lyrics: {}", lyrs.err().unwrap());
        exit(1);
    }

    println!("What a banger. OK, now select the lyrics to use: ");

    // Show options for the lyrics:
    let mut lyr_opts: Vec<CLIOption> = Vec::new();
    for lyr in lyrs? {
        let opt = CLIOption {
            artist: Some(lyr.artist),
            id: lyr.id,
            title: lyr.title,
        };
        lyr_opts.push(opt);
    }

    let lyric_opt = tokio::task::spawn_blocking(move || {
        cli::cli::present_options(lyr_opts)
    }).await?;

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
        audio_url = tokio::task::spawn_blocking(move || audio::audio::get_youtube_audio_url(video_opt.id.as_str())).await?;
    }

    if audio_url.is_none() {
        println!("Error getting audio URL");
        exit(1);
    }

    // Wait for the lyrics to be ready before playing.
    match lyrics_thread.await {
        Ok(Some(lyrics_map)) => {
            let play_thread = std::thread::spawn(move || {
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