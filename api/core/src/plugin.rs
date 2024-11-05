use std::ffi::CStr;

pub use async_trait::async_trait;

#[repr(C)]
pub struct Meta {
    pub name: &'static CStr,
    pub authors: &'static CStr,
    pub version: &'static CStr,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    async fn main(&self);
}
