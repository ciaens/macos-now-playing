use core::ffi::c_void;
use core::ptr::NonNull;
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::ffi;

/// Commands the system sends from media keys, headphones or Control Center.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Play,
    Pause,
    TogglePlayPause,
    Stop,
    NextTrack,
    PreviousTrack,
}

impl Command {
    const fn raw(self) -> i32 {
        match self {
            Self::Play => 0,
            Self::Pause => 1,
            Self::TogglePlayPause => 2,
            Self::Stop => 3,
            Self::NextTrack => 4,
            Self::PreviousTrack => 5,
        }
    }
}

const CHANGE_PLAYBACK_POSITION: i32 = 6;

/// Outcome a handler reports back to the system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlerStatus {
    Success,
    CommandFailed,
}

impl HandlerStatus {
    const fn raw(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::CommandFailed => 200,
        }
    }
}

type Handler = Box<dyn FnMut(f64) -> HandlerStatus + Send>;

/// Keeps a handler registered. Dropping it removes the handler on the main queue.
#[must_use = "dropping the token removes the handler"]
pub struct HandlerToken {
    command: i32,
    target: Option<NonNull<c_void>>,
    context: *mut Handler,
}

impl Drop for HandlerToken {
    fn drop(&mut self) {
        if let Some(target) = self.target {
            unsafe {
                ffi::mnp_command_remove(
                    self.command,
                    target.as_ptr(),
                    free_handler,
                    self.context.cast(),
                );
            }
        }
    }
}

// The closure is Send and only the main thread calls it; the target is a retained framework object that is only handed back to the framework.
unsafe impl Send for HandlerToken {}

/// Registers `handler` for `command`. Registration takes effect only on the main thread; use [`run_on_main`](crate::run_on_main) from elsewhere.
pub fn add_command_handler(
    command: Command,
    mut handler: impl FnMut() -> HandlerStatus + Send + 'static,
) -> HandlerToken {
    register(command.raw(), Box::new(move |_| handler()))
}

/// Registers `handler` for playback position changes. It receives the requested position in seconds.
pub fn add_playback_position_handler(
    handler: impl FnMut(f64) -> HandlerStatus + Send + 'static,
) -> HandlerToken {
    register(CHANGE_PLAYBACK_POSITION, Box::new(handler))
}

fn register(command: i32, handler: Handler) -> HandlerToken {
    let context = Box::into_raw(Box::new(handler));
    let target = NonNull::new(unsafe { ffi::mnp_command_add(command, trampoline, context.cast()) });

    if target.is_none() {
        free_handler(context.cast());
    }

    HandlerToken {
        command,
        target,
        context,
    }
}

extern "C" fn trampoline(context: *mut c_void, position: f64) -> i32 {
    let handler = unsafe { &mut *context.cast::<Handler>() };

    catch_unwind(AssertUnwindSafe(|| handler(position)))
        .unwrap_or(HandlerStatus::CommandFailed)
        .raw()
}

extern "C" fn free_handler(context: *mut c_void) {
    drop(unsafe { Box::from_raw(context.cast::<Handler>()) });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    #[test]
    fn trampoline_forwards_position_and_status() {
        let seen = Arc::new(AtomicU64::new(0));
        let handler: Handler = {
            let seen = seen.clone();
            Box::new(move |position| {
                seen.store(position.to_bits(), Ordering::SeqCst);
                HandlerStatus::CommandFailed
            })
        };
        let context = Box::into_raw(Box::new(handler));

        assert_eq!(trampoline(context.cast(), 12.5), 200);
        assert_eq!(seen.load(Ordering::SeqCst), 12.5_f64.to_bits());

        free_handler(context.cast());
    }

    #[test]
    fn token_registers_and_drops_off_the_main_thread() {
        let token = add_command_handler(Command::Play, || HandlerStatus::Success);
        drop(token);
    }
}
