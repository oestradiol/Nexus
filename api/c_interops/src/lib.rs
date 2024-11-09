#![expect(unsafe_code)]
use std::{
    collections::HashMap,
    ffi::{c_char, CStr},
    mem::MaybeUninit,
    ops::Deref,
    sync::{Arc, LazyLock, RwLock},
};

use nexus_api::tracing;
use tracing::{
    callsite::DefaultCallsite, field::FieldSet, level_filters, log, Callsite,
    Event, Level, Metadata, Subscriber,
};

static CALLSITES: LazyLock<
    RwLock<HashMap<Level, &'static CallsiteWrapper<'static>>>,
> = std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
fn get_or_init_callsite(level: Level) -> &'static DefaultCallsite {
    let callsites = CALLSITES.read().unwrap();
    // Necessary because Rust will change on ver. 2024
    #[allow(clippy::single_match_else)]
    // Necessary for early drop of `callsites`
    #[allow(clippy::option_if_let_else)]
    match callsites.get(&level) {
        Some(callsite) => callsite,
        None => {
            drop(callsites);
            let callsite = CallsiteWrapper::new(level);
            let callsite = Box::leak(Box::new(callsite));
            CALLSITES.write().unwrap().insert(level, callsite);
            callsite
        }
    }
}

struct CallsiteWrapper<'a> {
    metadata: Box<Metadata<'a>>,
    callsite: Box<DefaultCallsite>,
}
impl Deref for CallsiteWrapper<'_> {
    type Target = DefaultCallsite;
    fn deref(&self) -> &Self::Target {
        &self.callsite
    }
}
impl CallsiteWrapper<'static> {
    fn new(level: Level) -> Self {
        let mut field_id_uninit = MaybeUninit::zeroed();
        let r#ref = field_id_uninit.as_mut_ptr();
        let meta = Box::into_raw(Box::from(Metadata::new(
            "dynamic_event",
            "nexus_api_c_interops::log",
            level,
            Some(file!()),
            Some(line!()),
            Some("nexus_api_c_interops"),
            FieldSet::new(&["message"], unsafe {
                field_id_uninit.assume_init()
            }),
            tracing::metadata::Kind::EVENT,
        )));
        let callsite =
            Box::into_raw(Box::new(DefaultCallsite::new(unsafe { &*meta })));
        unsafe {
            let _ = std::mem::replace(&mut (*r#ref).0, &*callsite);
            Self {
                metadata: Box::from_raw(meta),
                callsite: Box::from_raw(callsite),
            }
        }
    }
}

const fn u8_to_level(level: u8) -> Level {
    match level {
        0 => Level::ERROR,
        1 => Level::WARN,
        3 => Level::DEBUG,
        4 => Level::TRACE,
        _ => Level::INFO, // Default to INFO if level is out of range
    }
}
/// # Safety
/// All the same as `CStr::from_ptr` function.
#[unsafe(no_mangle)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::missing_panics_doc)]
pub unsafe extern "C" fn log(log: *const c_char, level: u8) {
    let log = unsafe { CStr::from_ptr(log) }.to_string_lossy();
    let log: &str = Box::leak(Box::from(format!("{log}")));
    let level = u8_to_level(level);

    let callsite = get_or_init_callsite(level);
    let enabled = level <= level_filters::STATIC_MAX_LEVEL
        && level <= level_filters::LevelFilter::current()
        && {
            let interest = callsite.interest();
            !interest.is_never()
                && tracing::__macro_support::__is_enabled(
                    callsite.metadata(),
                    interest,
                )
        };
    let level = match level {
        Level::ERROR => log::Level::Error,
        Level::WARN => log::Level::Warn,
        Level::INFO => log::Level::Info,
        Level::DEBUG => log::Level::Debug,
        _ => log::Level::Trace,
    };
    let value_set = {
        let mut iter = callsite.metadata().fields().iter();
        [(
            &iter.next().expect("FieldSet corrupted (this is a bug)"),
            Some(&log as &dyn tracing::field::Value),
        )]
    };
    let value_set = callsite.metadata().fields().value_set(&value_set);
    if enabled {
        let meta = callsite.metadata();
        Event::dispatch(meta, &value_set);
    }
    if level <= log::STATIC_MAX_LEVEL && tracing::dispatcher::has_been_set() {
        use tracing::log;
        if level <= log::max_level() {
            let meta = callsite.metadata();
            let log_meta = log::Metadata::builder()
                .level(level)
                .target(meta.target())
                .build();
            let logger = log::logger();
            if logger.enabled(&log_meta) {
                tracing::__macro_support::__tracing_log(
                    meta, logger, log_meta, &value_set,
                );
            }
        }
    }
}

/// # Safety
/// All the same as `Box::from_raw` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_logger(
    logger: *mut Arc<dyn Subscriber + Send + Sync>,
) -> bool {
    let logger = unsafe { *Box::from_raw(logger) };
    tracing::subscriber::set_global_default(logger).is_ok()
}
