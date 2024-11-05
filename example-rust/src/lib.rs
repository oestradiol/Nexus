use nexus_api::{r#impl, Meta};
use tracing::info;

r#impl! {
    pub static META: Meta = Meta {
        name: env!("CARGO_PKG_NAME"),
        authors: env!("CARGO_PKG_AUTHORS"),
        version: env!("CARGO_PKG_VERSION"),
    };

    async fn main(&self) {
        info!("Hello, world!");
    }
}
