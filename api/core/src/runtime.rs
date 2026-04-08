use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::time::Duration;

/// Trait for accessing the runtime from plugins.
/// This avoids TLS issues by passing the runtime handle explicitly.
pub trait RuntimeHandle: Send + Sync + std::fmt::Debug {
    /// Spawn a new task on the runtime
    fn spawn(
        &self,
        future: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> tokio::task::JoinHandle<()>;

    /// Create a sleep future
    fn sleep(&self, duration: Duration) -> tokio::time::Sleep;

    /// Get the current time
    fn now(&self) -> tokio::time::Instant;
}

/// Wrapper around tokio's runtime handle
#[derive(Debug)]
pub struct TokioRuntimeHandle {
    handle: tokio::runtime::Handle,
}

impl TokioRuntimeHandle {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self { handle }
    }
}

impl RuntimeHandle for TokioRuntimeHandle {
    fn spawn(
        &self,
        future: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> tokio::task::JoinHandle<()> {
        self.handle.spawn(future)
    }

    fn sleep(&self, duration: Duration) -> tokio::time::Sleep {
        tokio::time::sleep(duration)
    }

    fn now(&self) -> tokio::time::Instant {
        tokio::time::Instant::now()
    }
}

/// Type alias for the runtime handle used in plugins
pub type RuntimeRef = Arc<dyn RuntimeHandle>;