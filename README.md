# macos-now-playing

Publishes now playing information to macOS (Control Center, the media key overlay, headphones and other remote controls) and receives remote commands, through `MediaPlayer.framework`.

The framework calls live in one Objective-C file compiled with `cc`.

## Requirements

- macOS 12 or later
- Xcode Command Line Tools

## Usage

```rust
use std::time::Duration;

use macos_now_playing::{
    Artwork, Command, HandlerStatus, NowPlayingInfo, PlaybackState, add_command_handler,
    run_current_run_loop, set_now_playing_info, set_playback_state,
};

let artwork = std::fs::read("cover.jpg")
    .ok()
    .and_then(|bytes| Artwork::from_image_data(&bytes));

set_now_playing_info(&NowPlayingInfo {
    title: "Sunburn",
    artist: Some("Muse"),
    album_title: Some("Showbiz"),
    playback_duration: 234.0,
    playback_rate: 1.0,
    artwork: artwork.as_ref(),
    ..NowPlayingInfo::default()
});
set_playback_state(PlaybackState::Playing);

let _play = add_command_handler(Command::Play, || {
    println!("play");
    HandlerStatus::Success
});

loop {
    run_current_run_loop(Duration::from_secs(1));
}
```

## Threads

- Register handlers on the main thread. From another thread, wrap the registration in `run_on_main`.
- Handlers run on the main thread while its run loop is pumped with `run_current_run_loop`.
- Dropping a `HandlerToken` schedules the removal on the main queue, so it is safe from any thread. Until the main run loop services the queue, the handler may still fire.
- `set_now_playing_info`, `clear_now_playing_info`, `set_playback_state` and `Artwork` work from any thread.
- `stop_main_run_loop` stops the main thread's loop from any thread.

## Example

`cargo run --example smoke -- cover.jpg` fills Control Center for thirty seconds and prints every command it receives.

## License

MIT
