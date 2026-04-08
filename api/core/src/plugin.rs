use std::ffi::CStr;

use crate::runtime::RuntimeRef;

pub use async_trait::async_trait;

#[repr(C)]
pub struct Meta {
    pub name: &'static CStr,
    pub authors: &'static CStr,
    pub version: &'static CStr,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    /// Initialize the plugin with a runtime handle.
    /// Called once before main() by the loader.
    fn init(&mut self, runtime: RuntimeRef);

    /// Main plugin entry point.
    async fn main(&self);
}
