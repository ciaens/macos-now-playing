use core::ffi::c_void;
use core::ptr::{self, NonNull};

use crate::ffi;

/// Cover art decoded once and reused across [`NowPlayingInfo`] updates.
#[derive(Debug)]
pub struct Artwork {
    raw: NonNull<c_void>,
}

impl Artwork {
    /// Decodes PNG, JPEG or any other image data `NSImage` reads.
    #[must_use]
    pub fn from_image_data(bytes: &[u8]) -> Option<Self> {
        NonNull::new(unsafe { ffi::mnp_artwork_new(bytes.as_ptr(), bytes.len()) })
            .map(|raw| Self { raw })
    }

    fn as_ptr(&self) -> *const c_void {
        self.raw.as_ptr().cast_const()
    }
}

impl Drop for Artwork {
    fn drop(&mut self) {
        unsafe { ffi::mnp_artwork_free(self.raw.as_ptr()) }
    }
}

// The handle is a retained MPMediaItemArtwork that never changes after creation; Rust only hands it to the framework or releases it.
unsafe impl Send for Artwork {}
// Shared references only copy the pointer value, so concurrent use cannot race.
unsafe impl Sync for Artwork {}

/// Playback state shown next to the now playing entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

impl PlaybackState {
    const fn raw(self) -> i32 {
        match self {
            Self::Playing => 1,
            Self::Paused => 2,
            Self::Stopped => 3,
        }
    }
}

/// Metadata of the current track. Absent optional fields are left out of the entry.
#[derive(Clone, Copy, Debug, Default)]
pub struct NowPlayingInfo<'a> {
    pub title: &'a str,
    pub artist: Option<&'a str>,
    pub album_title: Option<&'a str>,
    pub playback_duration: f64,
    pub elapsed_playback_time: f64,
    pub playback_rate: f64,
    pub playback_queue_index: u64,
    pub playback_queue_count: u64,
    pub artwork: Option<&'a Artwork>,
}

/// Publishes `info` as the system's now playing entry.
pub fn set_now_playing_info(info: &NowPlayingInfo<'_>) {
    let (artist, artist_length) = parts(info.artist);
    let (album_title, album_title_length) = parts(info.album_title);

    let raw = ffi::NowPlayingInfoRaw {
        title: info.title.as_ptr(),
        title_length: info.title.len(),
        artist,
        artist_length,
        album_title,
        album_title_length,
        playback_duration: info.playback_duration,
        elapsed_playback_time: info.elapsed_playback_time,
        playback_rate: info.playback_rate,
        playback_queue_index: info.playback_queue_index,
        playback_queue_count: info.playback_queue_count,
        artwork: info.artwork.map_or(ptr::null(), Artwork::as_ptr),
    };

    unsafe { ffi::mnp_now_playing_set(&raw const raw) }
}

/// Removes the now playing entry.
pub fn clear_now_playing_info() {
    ffi::mnp_now_playing_clear();
}

/// Sets the playback state shown by the system.
pub fn set_playback_state(state: PlaybackState) {
    ffi::mnp_playback_state_set(state.raw());
}

fn parts(value: Option<&str>) -> (*const u8, usize) {
    value.map_or((ptr::null(), 0), |value| (value.as_ptr(), value.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = include_bytes!("../examples/artwork.png");

    #[test]
    fn artwork_rejects_invalid_data() {
        assert!(Artwork::from_image_data(&[1, 2, 3]).is_none());
    }

    #[test]
    fn artwork_accepts_png() {
        assert!(Artwork::from_image_data(PNG).is_some());
    }

    #[test]
    fn info_round_trips_through_the_framework() {
        let artwork = Artwork::from_image_data(PNG);

        set_now_playing_info(&NowPlayingInfo {
            title: "Title",
            artist: Some("Artist"),
            artwork: artwork.as_ref(),
            ..NowPlayingInfo::default()
        });
        set_playback_state(PlaybackState::Playing);
        set_now_playing_info(&NowPlayingInfo::default());
        clear_now_playing_info();
        set_playback_state(PlaybackState::Stopped);
    }
}
