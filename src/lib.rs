use reqwest::Client;
use serde_json::Value;

pub(crate) async fn fetch_videos(
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

use csv::Writer;

pub(crate) fn write_to_csv(videos: Vec<Value>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new CSV writer and specify the output file name (change if desired)
    let mut wtr = Writer::from_path("youtube_videos.csv")?;

    // Write the header row
    wtr.write_record(&["Video ID", "Title", "Description", "Published At"])?;

    for video in videos {
        let snippet = &video["snippet"];

        // Write each video's data to the CSV file
        wtr.write_record(&[
            video["id"]["videoId"].as_str().unwrap_or(""),
            snippet["title"].as_str().unwrap_or(""),
            snippet["description"].as_str().unwrap_or(""),
            snippet["publishedAt"].as_str().unwrap_or(""),
        ])?;
    }

    wtr.flush()?; // Ensure all data is written to the file
    Ok(())
}
