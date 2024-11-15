#![expect(unsafe_code)]

use std::{ops::Deref, path::Path};

use libloading::{Error, Library};
use nexus_utils::api::{tracing, Meta, Plugin};
use tracing::{info, warn};

pub struct PluginInstance {
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

            let new = lib.get::<unsafe extern "Rust" fn(
                //    Arc<dyn Subscriber + Send + Sync>,
                ) -> Box<dyn Plugin>>(b"_new_rust_impl")?;
            let plugin = new(); //LOGGER.get().unwrap().clone()

            Ok(Self { meta, plugin, lib })
        }
    }
}

struct LibWrapper(Option<Library>);
impl LibWrapper {
    unsafe fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let lib = unsafe { Library::new(path.as_ref()) }.map(Some);
        lib.map(Self)
    }
}
impl Drop for LibWrapper {
    fn drop(&mut self) {
        // Necessary because Rust will change on ver. 2024
        #[allow(clippy::single_match)]
        match self.0.take().unwrap().close() {
            Err(e) => {
                warn!("Failed to close library: {}", e);
            }
            _ => {}
        }
    }
}
impl Deref for LibWrapper {
    type Target = Library;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}
