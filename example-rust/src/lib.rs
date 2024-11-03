use std::sync::Arc;

use nexus_api::plugin::{async_trait, Meta, Plugin};
use nexus_api_macros::struct_c;
use tracing::{info, Subscriber};

struct_c! {
    #[allow(unsafe_code)]
    #[unsafe(no_mangle)]
    pub static META: Meta = Meta {
        name: env!("CARGO_PKG_NAME"),
        authors: env!("CARGO_PKG_AUTHORS"),
        version: env!("CARGO_PKG_VERSION"),
    };
}

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "Rust" fn new(
    logger: Arc<dyn Subscriber + Send + Sync>,
) -> Box<dyn Plugin> {
    tracing::subscriber::set_global_default(logger).unwrap();
    Box::new(Example)
}

pub struct Example;
#[async_trait]
impl Plugin for Example {
    async fn start(&self) {
        info!("Executing!");
    }
}
