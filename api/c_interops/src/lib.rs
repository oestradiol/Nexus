#![expect(unsafe_code)]
use std::{
    ffi::{c_char, CStr},
    sync::Arc,
};

use tracing::{debug, error, info, trace, warn, Subscriber};

#[unsafe(no_mangle)]
pub extern "C" fn log(log: *const c_char, level: u8) -> bool {
    let log = unsafe { CStr::from_ptr(log) }.to_string_lossy();
    match level {
        0 => error!("{}", log),
        1 => warn!("{}", log),
        2 => info!("{}", log),
        3 => debug!("{}", log),
        4 => trace!("{}", log),
        _ => return false,
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn init_logger(
    logger: *mut Arc<dyn Subscriber + Send + Sync>,
) -> bool {
    let logger = unsafe { *Box::from_raw(logger) };
    tracing::subscriber::set_global_default(logger).is_ok()
}
