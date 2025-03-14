# CLIraoke

My first Rust app. Learned a lot. Swore a lot.

CLIraoke is karaoke for rustaceans too busy shaving ms to leave their command line. Lyrics so memory-safe you'll never forget them

## Usage

`cp .env-sample .env`

Get a [YouTube API](https://developers.google.com/youtube/v3/getting-started) key, because you can't have mine.

Add it to the `.env`

To build the binary, run 
```bash
cargo build --release
```

To run the binary, run
```bash
cargo run --release
```

When greeted with my character-filled prompts, simply enter a query. This is a YouTube search query that I add the string `"karaoke version"` at the end of.

You will then receive a list of synced lyrics from the awesome open project [LRCLib](https://lrclib.net/). Experiment with combinations, as the audio and lyrics may not be perfectly synced at start time.

## Troubleshooting

You may need to install `yt-dlp` and `ffmpeg` for this thing.

## Improvements

To make this better:

- [ ] Handle yt-dlp and ffmpeg commands in Rust wrappers, or pipe audio to a sink to play back
- [ ] Better controls: kill a song without killing the app
- [ ] Queueing
- [ ] Back up LRCLib dump
- [ ] Add DB so that users can indicate when a song/lyric combo is good. The DB saves the two IDs and returns search hits from there first.
- [ ] Ability to scrub audio forward with arrow keys, so syncing lines up
- [ ] ASCII stock photos, like one of those cheesy old karaoke display