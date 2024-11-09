mod loader;
mod on_shutdown;

use std::path::PathBuf;

use loader::PluginInstance;
use nexus_utils::api::tokio;
use on_shutdown::with_graceful_shutdown;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    let discord_worker = {
        let path = PathBuf::from("./Logs");
        let discord_hook = Some("https://discord.com/api/webhooks/1302403792595320932/uRKBpnrO0QHW2av73HubxxOgIsOAUPNyrcRFvwwrvBYpxRZMvy-Ycj5I2jZ1AFbr2-OI".to_string());
        nexus_utils::init_logging(&path, "INFO".to_string(), None).await
    };

    tokio::spawn(async move {
        PluginInstance::new("./libnexus_metrics.so")
            .unwrap()
            .plugin
            .main()
            .await;
    });

    with_graceful_shutdown(discord_worker).await;
}
