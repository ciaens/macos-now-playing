use core::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use crate::ffi;

type MainFunction = Box<dyn FnOnce() + Send>;

/// Runs the calling thread's run loop in the default mode until `timeout` elapses or the loop is stopped.
pub fn run_current_run_loop(timeout: Duration) {
    ffi::mnp_run_loop_run(timeout.as_secs_f64());
}

/// Stops the main thread's run loop, from any thread.
pub fn stop_main_run_loop() {
    ffi::mnp_run_loop_stop_main();
}

/// Runs `function` on the main thread once the main run loop services its queue.
pub fn run_on_main(function: impl FnOnce() + Send + 'static) {
    let context: Box<MainFunction> = Box::new(Box::new(function));

    unsafe { ffi::mnp_run_on_main(call_main_function, Box::into_raw(context).cast()) }
}

extern "C" fn call_main_function(context: *mut c_void) {
    let function = unsafe { Box::from_raw(context.cast::<MainFunction>()) };

    let _ = catch_unwind(AssertUnwindSafe(function));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_returns() {
        run_current_run_loop(Duration::from_millis(10));
    }

    #[test]
    fn run_on_main_queues_the_function() {
        run_on_main(|| {});
    }
}
