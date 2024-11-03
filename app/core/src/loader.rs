#![allow(unsafe_code)]

use std::{ops::Deref, path::Path, sync::Arc};

use libloading::{Error, Library};
use nexus_api::plugin::{Meta, Plugin};
use nexus_utils::LOGGER;
use tracing::{info, warn, Subscriber};

pub(crate) struct PluginInstance {
    pub(crate) meta: &'static Meta,
    pub(crate) plugin: Box<dyn Plugin>,
    lib: LibWrapper,
}

impl PluginInstance {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        unsafe {
            info!(
                "Loading `{}`...",
                path.file_name().unwrap().to_string_lossy()
            );

            let lib = LibWrapper::new(path)?;
            let meta = *lib.get(b"META")?;

            let new = lib
                .get::<unsafe extern "Rust" fn(
                    Arc<dyn Subscriber + Send + Sync>,
                ) -> Box<dyn Plugin>>(b"new")?;
            let plugin = new(LOGGER.get().unwrap().clone());

            Ok(Self { meta, plugin, lib })
        }
    }
}

struct LibWrapper(Option<Library>);
impl LibWrapper {
    unsafe fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self(Some(unsafe { Library::new(path.as_ref())? })))
    }
}
impl Drop for LibWrapper {
    fn drop(&mut self) {
        if let Err(e) = self.0.take().unwrap().close() {
            warn!("Failed to close library: {}", e);
        }
    }
}
impl Deref for LibWrapper {
    type Target = Library;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}
