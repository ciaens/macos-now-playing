use core::ffi::c_void;

#[repr(C)]
pub struct NowPlayingInfoRaw {
    pub title: *const u8,
    pub title_length: usize,
    pub artist: *const u8,
    pub artist_length: usize,
    pub album_title: *const u8,
    pub album_title_length: usize,
    pub playback_duration: f64,
    pub elapsed_playback_time: f64,
    pub playback_rate: f64,
    pub playback_queue_index: u64,
    pub playback_queue_count: u64,
    pub artwork: *const c_void,
}

pub type CommandHandler = extern "C" fn(context: *mut c_void, position: f64) -> i32;
pub type ContextFunction = extern "C" fn(context: *mut c_void);

unsafe extern "C" {
    pub unsafe fn mnp_now_playing_set(info: *const NowPlayingInfoRaw);
    pub safe fn mnp_now_playing_clear();
    pub safe fn mnp_playback_state_set(state: i32);
    pub unsafe fn mnp_artwork_new(bytes: *const u8, length: usize) -> *mut c_void;
    pub unsafe fn mnp_artwork_free(artwork: *mut c_void);
    pub unsafe fn mnp_command_add(
        command: i32,
        handler: CommandHandler,
        context: *mut c_void,
    ) -> *mut c_void;
    pub unsafe fn mnp_command_remove(
        command: i32,
        target: *mut c_void,
        free_context: ContextFunction,
        context: *mut c_void,
    );
    pub unsafe fn mnp_run_on_main(function: ContextFunction, context: *mut c_void);
    pub safe fn mnp_run_loop_run(seconds: f64);
    pub safe fn mnp_run_loop_stop_main();
}
