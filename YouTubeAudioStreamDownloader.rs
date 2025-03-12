mod lib;

use yt_dlp::Youtube;
use std::path::PathBuf;
use dotenv::dotenv;
use std::env;
use std::process::exit;
use std::env::args;
use std::error::Error;

use stream_download::http::HttpStream;
use stream_download::http::reqwest::Client;
use stream_download::source::{DecodeError, SourceStream};
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tracing::info;
use tracing::metadata::LevelFilter;
use tokio::task;

const ENV_API_KEY: &str = "YOUTUBE_API_KEY";
const SEARCH_SUFFIX: &str = "karaoke";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = args().nth(1).unwrap_or_else(|| {
        "http://www.hyperion-records.co.uk/audiotest/14 Clementi Piano Sonata in D major, Op 25 No \
         6 - Movement 2 Un poco andante.MP3"
            .to_string()
    });

    let stream = HttpStream::<Client>::create(url.parse()?).await?;

    info!("content length={:?}", stream.content_length());
    info!("content type={:?}", stream.content_type());

    let reader =
        match StreamDownload::from_stream(stream, TempStorageProvider::new(), Settings::default())
            .await
        {
            Ok(reader) => reader,
            Err(e) => return Err(Box::new(e.decode_error().await)),
        };

    let handle = task::spawn_blocking(move || {
        // Rodio audio sink creation and playback
        let (_stream, handle) = rodio::OutputStream::try_default()?;
        let sink = rodio::Sink::try_new(&handle)?;
        sink.append(rodio::Decoder::new(reader)?);
        sink.sleep_until_end();

        Ok::<_, Box<dyn Error + Send + Sync>>(())
    });

    // Propagate the error explicitly to avoid compiler issues
    match handle.await {
        Ok(result) => result?, // Unwrap inner Result here
        Err(join_error) => Err(Box::new(join_error))?, // Handle task join errors
    }

    Ok(())
}