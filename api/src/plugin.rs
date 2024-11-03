pub use async_trait::async_trait;

pub struct Meta {
    pub name: &'static str,
    pub authors: &'static str,
    pub version: &'static str,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    async fn start(&self);
}
