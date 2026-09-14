#[cfg(target_os = "macos")]
fn main() {
    use std::time::Duration;

    use macos_now_playing::{
        Artwork, Command, HandlerStatus, NowPlayingInfo, PlaybackState, add_command_handler,
        add_playback_position_handler, clear_now_playing_info, run_current_run_loop,
        set_now_playing_info, set_playback_state,
    };

    let bytes = std::env::args()
        .nth(1)
        .and_then(|path| std::fs::read(path).ok());
    let artwork =
        Artwork::from_image_data(bytes.as_deref().unwrap_or(include_bytes!("artwork.png")));

    set_now_playing_info(&NowPlayingInfo {
        title: "Smoke test",
        artist: Some("macos-now-playing"),
        album_title: Some("Examples"),
        playback_duration: 240.0,
        playback_rate: 1.0,
        playback_queue_count: 1,
        artwork: artwork.as_ref(),
        ..NowPlayingInfo::default()
    });
    set_playback_state(PlaybackState::Playing);

    let commands = [
        Command::Play,
        Command::Pause,
        Command::TogglePlayPause,
        Command::Stop,
        Command::NextTrack,
        Command::PreviousTrack,
    ];
    let mut tokens: Vec<_> = commands
        .into_iter()
        .map(|command| {
            add_command_handler(command, move || {
                println!("{command:?}");
                HandlerStatus::Success
            })
        })
        .collect();
    tokens.push(add_playback_position_handler(|position| {
        println!("seek to {position:.1}s");
        HandlerStatus::Success
    }));

    println!(
        "Open Control Center, then use the media keys, headphone buttons or the position bar. Exits after 30 seconds."
    );
    for _ in 0..30 {
        run_current_run_loop(Duration::from_secs(1));
    }

    drop(tokens);
    run_current_run_loop(Duration::from_millis(100));
    clear_now_playing_info();
    set_playback_state(PlaybackState::Stopped);
}

#[cfg(not(target_os = "macos"))]
fn main() {}
