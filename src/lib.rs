#![cfg(target_os = "macos")]
//! Now playing information and remote commands on macOS through `MediaPlayer.framework`.

mod ffi;
mod now_playing;
mod remote_command;
mod run_loop;

pub use now_playing::{
    Artwork, NowPlayingInfo, PlaybackState, clear_now_playing_info, set_now_playing_info,
    set_playback_state,
};
pub use remote_command::{
    Command, HandlerStatus, HandlerToken, add_command_handler, add_playback_position_handler,
};
pub use run_loop::{run_current_run_loop, run_on_main, stop_main_run_loop};
